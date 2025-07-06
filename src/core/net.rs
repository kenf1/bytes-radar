use crate::core::{
    analysis::{FileMetrics, ProjectAnalysis},
    error::{AnalysisError, Result},
    registry::LanguageRegistry,
};
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use reqwest::Client;
use std::io::{Cursor, Read};
use tar::Archive;
use tokio::task;

static USER_AGENT: &str = "bytes-radar/1.0.0";

#[cfg(feature = "cli")]
use indicatif::ProgressBar;

pub struct RemoteAnalyzer {
    client: Client,
    github_token: Option<String>,
    timeout: u64,
    allow_insecure: bool,
    #[cfg(feature = "cli")]
    progress_bar: Option<ProgressBar>,
}

impl RemoteAnalyzer {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            github_token: None,
            timeout: 300,
            allow_insecure: false,
            #[cfg(feature = "cli")]
            progress_bar: None,
        }
    }

    #[cfg(feature = "cli")]
    pub fn set_progress_bar(&mut self, progress_bar: Option<ProgressBar>) {
        self.progress_bar = progress_bar;
    }

    pub fn set_github_token(&mut self, token: &str) {
        self.github_token = Some(token.to_string());
        self.rebuild_client();
    }

    pub fn set_timeout(&mut self, timeout: u64) {
        self.timeout = timeout;
        self.rebuild_client();
    }

    pub fn set_allow_insecure(&mut self, allow_insecure: bool) {
        self.allow_insecure = allow_insecure;
        self.rebuild_client();
    }

    fn rebuild_client(&mut self) {
        let mut builder = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(self.timeout));

        if self.allow_insecure {
            builder = builder.danger_accept_invalid_certs(true);
        }

        if let Some(token) = &self.github_token {
            let mut headers = reqwest::header::HeaderMap::new();
            let auth_value = format!("token {}", token);
            headers.insert(
                reqwest::header::AUTHORIZATION,
                auth_value.parse().expect("Invalid token format"),
            );
            builder = builder.default_headers(headers);
        }

        self.client = builder.build().expect("Failed to create HTTP client");
    }

    pub async fn analyze_url(&self, url: &str) -> Result<ProjectAnalysis> {
        let download_url = self.resolve_git_url(url).await?;
        let project_analysis = self.analyze_tarball_with_name(&download_url, url).await?;
        Ok(project_analysis)
    }

    async fn resolve_git_url(&self, url: &str) -> Result<String> {
        if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
            return Ok(url.to_string());
        }

        if url.starts_with("http://") || url.starts_with("https://") {
            if !url.contains("github.com")
                && !url.contains("gitlab.com")
                && !url.contains("gitlab.")
                && !url.contains("bitbucket.org")
                && !url.contains("codeberg.org")
            {
                if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
                    return Ok(url.to_string());
                } else {
                    return Ok(url.to_string());
                }
            }
        }

        let branches = ["main", "master", "develop", "dev"];

        if let Some(github_url) = self.parse_github_url_with_branch(url) {
            return Ok(github_url);
        }

        if let Some(gitlab_url) = self.parse_gitlab_url_with_branch(url) {
            return Ok(gitlab_url);
        }

        if let Some(bitbucket_url) = self.parse_bitbucket_url_with_branch(url) {
            return Ok(bitbucket_url);
        }

        if let Some(codeberg_url) = self.parse_codeberg_url_with_branch(url) {
            return Ok(codeberg_url);
        }

        for branch in &branches {
            if let Some(github_url) = self.parse_github_url(url, branch) {
                if self.check_url_exists(&github_url).await {
                    return Ok(github_url);
                }
            }

            if let Some(gitlab_url) = self.parse_gitlab_url(url, branch) {
                if self.check_url_exists(&gitlab_url).await {
                    return Ok(gitlab_url);
                }
            }

            if let Some(bitbucket_url) = self.parse_bitbucket_url(url, branch) {
                if self.check_url_exists(&bitbucket_url).await {
                    return Ok(bitbucket_url);
                }
            }

            if let Some(codeberg_url) = self.parse_codeberg_url(url, branch) {
                if self.check_url_exists(&codeberg_url).await {
                    return Ok(codeberg_url);
                }
            }
        }

        Err(AnalysisError::url_parsing(format!(
            "Unsupported URL format or no accessible branch found: {}. Please provide a direct tar.gz URL or a supported repository URL.",
            url
        )))
    }

    fn parse_github_url_with_branch(&self, url: &str) -> Option<String> {
        if url.contains("github.com") {
            if url.contains("/tree/") {
                let parts: Vec<&str> = url.split('/').collect();
                if let Some(tree_pos) = parts.iter().position(|&x| x == "tree") {
                    if tree_pos + 1 < parts.len() && tree_pos >= 2 {
                        let owner = parts[tree_pos - 2];
                        let repo = parts[tree_pos - 1];
                        let branch = parts[tree_pos + 1];
                        return Some(format!(
                            "https://github.com/{}/{}/archive/refs/heads/{}.tar.gz",
                            owner, repo, branch
                        ));
                    }
                }
            }

            if url.contains("/commit/") {
                return self.extract_github_commit_url(url);
            }
        }
        None
    }

    fn parse_gitlab_url_with_branch(&self, url: &str) -> Option<String> {
        if url.contains("gitlab.com") || url.contains("gitlab.") {
            if url.contains("/-/tree/") {
                let parts: Vec<&str> = url.split('/').collect();
                if let Some(tree_pos) = parts.iter().position(|&x| x == "tree") {
                    if tree_pos + 1 < parts.len() && tree_pos >= 3 {
                        let gitlab_pos = parts.iter().position(|&x| x.contains("gitlab")).unwrap();
                        let host = parts[gitlab_pos];
                        let owner = parts[gitlab_pos + 1];
                        let repo = parts[gitlab_pos + 2];
                        let branch = parts[tree_pos + 1];
                        return Some(format!(
                            "https://{}/{}{}/-/archive/{}/{}-{}.tar.gz",
                            host,
                            owner,
                            if parts.len() > gitlab_pos + 3 && parts[gitlab_pos + 3] != "-" {
                                format!("/{}", parts[gitlab_pos + 3..tree_pos - 1].join("/"))
                            } else {
                                String::new()
                            },
                            branch,
                            repo,
                            branch
                        ));
                    }
                }
            }
        }
        None
    }

    fn parse_bitbucket_url_with_branch(&self, url: &str) -> Option<String> {
        if url.contains("bitbucket.org") {
            if url.contains("/commits/") {
                let parts: Vec<&str> = url.split('/').collect();
                if let Some(commits_pos) = parts.iter().position(|&x| x == "commits") {
                    if commits_pos + 1 < parts.len() && commits_pos >= 2 {
                        let owner = parts[commits_pos - 2];
                        let repo = parts[commits_pos - 1];
                        let commit = parts[commits_pos + 1];
                        return Some(format!(
                            "https://bitbucket.org/{}/{}/get/{}.tar.gz",
                            owner, repo, commit
                        ));
                    }
                }
            }

            if url.contains("/branch/") {
                let parts: Vec<&str> = url.split('/').collect();
                if let Some(branch_pos) = parts.iter().position(|&x| x == "branch") {
                    if branch_pos + 1 < parts.len() && branch_pos >= 2 {
                        let owner = parts[branch_pos - 2];
                        let repo = parts[branch_pos - 1];
                        let branch = parts[branch_pos + 1];
                        return Some(format!(
                            "https://bitbucket.org/{}/{}/get/{}.tar.gz",
                            owner, repo, branch
                        ));
                    }
                }
            }
        }
        None
    }

    fn parse_codeberg_url_with_branch(&self, url: &str) -> Option<String> {
        if url.contains("codeberg.org") {
            if url.contains("/commit/") {
                let parts: Vec<&str> = url.split('/').collect();
                if let Some(commit_pos) = parts.iter().position(|&x| x == "commit") {
                    if commit_pos + 1 < parts.len() && commit_pos >= 2 {
                        let owner = parts[commit_pos - 2];
                        let repo = parts[commit_pos - 1];
                        let commit = parts[commit_pos + 1];
                        return Some(format!(
                            "https://codeberg.org/{}/{}/archive/{}.tar.gz",
                            owner, repo, commit
                        ));
                    }
                }
            }

            if url.contains("/src/branch/") {
                let parts: Vec<&str> = url.split('/').collect();
                if let Some(branch_pos) = parts.iter().position(|&x| x == "branch") {
                    if branch_pos + 1 < parts.len() && branch_pos >= 3 {
                        let owner = parts[branch_pos - 3];
                        let repo = parts[branch_pos - 2];
                        let branch = parts[branch_pos + 1];
                        return Some(format!(
                            "https://codeberg.org/{}/{}/archive/{}.tar.gz",
                            owner, repo, branch
                        ));
                    }
                }
            }
        }
        None
    }

    fn parse_bitbucket_url(&self, url: &str, branch: &str) -> Option<String> {
        if url.contains("bitbucket.org") {
            let parts: Vec<&str> = url.split('/').collect();
            if let Some(bitbucket_pos) = parts.iter().position(|&x| x == "bitbucket.org") {
                if bitbucket_pos + 2 < parts.len() {
                    let owner = parts[bitbucket_pos + 1];
                    let repo = parts[bitbucket_pos + 2];
                    return Some(format!(
                        "https://bitbucket.org/{}/{}/get/{}.tar.gz",
                        owner, repo, branch
                    ));
                }
            }
        }
        None
    }

    fn parse_codeberg_url(&self, url: &str, branch: &str) -> Option<String> {
        if url.contains("codeberg.org") {
            let parts: Vec<&str> = url.split('/').collect();
            if let Some(codeberg_pos) = parts.iter().position(|&x| x == "codeberg.org") {
                if codeberg_pos + 2 < parts.len() {
                    let owner = parts[codeberg_pos + 1];
                    let repo = parts[codeberg_pos + 2];
                    return Some(format!(
                        "https://codeberg.org/{}/{}/archive/{}.tar.gz",
                        owner, repo, branch
                    ));
                }
            }
        }
        None
    }

    async fn check_url_exists(&self, url: &str) -> bool {
        if let Ok(response) = self.client.head(url).send().await {
            response.status().is_success()
        } else {
            false
        }
    }

    fn parse_github_url(&self, url: &str, branch: &str) -> Option<String> {
        let url = url.trim_end_matches('/');

        if url.contains("github.com") {
            if let Some(commit_url) = self.extract_github_commit_url(url) {
                return Some(commit_url);
            }

            if let Some(repo_url) = self.extract_github_repo_url(url, branch) {
                return Some(repo_url);
            }
        }

        None
    }

    fn extract_github_commit_url(&self, url: &str) -> Option<String> {
        if url.contains("/commit/") {
            let parts: Vec<&str> = url.split('/').collect();
            if let Some(commit_pos) = parts.iter().position(|&x| x == "commit") {
                if commit_pos + 1 < parts.len() {
                    let owner = parts.get(parts.len() - 4)?;
                    let repo = parts.get(parts.len() - 3)?;
                    let commit = parts.get(commit_pos + 1)?;
                    return Some(format!(
                        "https://github.com/{}/{}/archive/{}.tar.gz",
                        owner, repo, commit
                    ));
                }
            }
        }
        None
    }

    fn extract_github_repo_url(&self, url: &str, branch: &str) -> Option<String> {
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() >= 2 && parts.contains(&"github.com") {
            if let Some(github_pos) = parts.iter().position(|&x| x == "github.com") {
                if github_pos + 2 < parts.len() {
                    let owner = parts[github_pos + 1];
                    let repo = parts[github_pos + 2];
                    return Some(format!(
                        "https://github.com/{}/{}/archive/refs/heads/{}.tar.gz",
                        owner, repo, branch
                    ));
                }
            }
        }
        None
    }

    fn parse_gitlab_url(&self, url: &str, branch: &str) -> Option<String> {
        let url = url.trim_end_matches('/');

        if url.contains("gitlab.com") || url.contains("gitlab.") {
            let parts: Vec<&str> = url.split('/').collect();
            if let Some(gitlab_pos) = parts.iter().position(|&x| x.contains("gitlab")) {
                if gitlab_pos + 2 < parts.len() {
                    let host = parts[gitlab_pos];
                    let owner = parts[gitlab_pos + 1];
                    let repo = parts[gitlab_pos + 2];
                    return Some(format!(
                        "https://{}/{}{}/-/archive/{}/{}-{}.tar.gz",
                        host,
                        owner,
                        if parts.len() > gitlab_pos + 3 {
                            format!("/{}", parts[gitlab_pos + 3..].join("/"))
                        } else {
                            String::new()
                        },
                        branch,
                        repo,
                        branch
                    ));
                }
            }
        }

        None
    }

    async fn analyze_tarball_with_name(
        &self,
        download_url: &str,
        original_url: &str,
    ) -> Result<ProjectAnalysis> {
        let project_name = self.extract_project_name_from_original(original_url);
        let mut project_analysis = ProjectAnalysis::new(project_name);

        let response = self
            .client
            .get(download_url)
            .send()
            .await
            .map_err(|e| AnalysisError::network(format!("Failed to fetch URL: {}", e)))?;

        if !response.status().is_success() {
            return Err(AnalysisError::network(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        let total_size = response.content_length();

        #[cfg(feature = "cli")]
        if let Some(pb) = &self.progress_bar {
            if let Some(size) = total_size {
                use indicatif::ProgressStyle;
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {decimal_bytes_per_sec} {binary_bytes}/{binary_total_bytes} ({eta}) {msg}")
                        .unwrap_or_else(|_| ProgressStyle::default_bar())
                        .progress_chars("#>-"),
                );
                pb.set_length(size);
                pb.set_message("Downloading and processing...");
            } else {
                pb.set_message("Downloading and processing...");
                pb.enable_steady_tick(std::time::Duration::from_millis(120));
            }
        }

        let stream = response.bytes_stream();
        let stream_reader = StreamReader::new(
            stream,
            #[cfg(feature = "cli")]
            self.progress_bar.clone(),
            total_size,
        );

        #[cfg(feature = "cli")]
        if let Some(pb) = &self.progress_bar {
            pb.set_message("Processing archive...");
        }

        self.process_tarball_stream(stream_reader, &mut project_analysis)
            .await?;
        Ok(project_analysis)
    }

    async fn process_tarball_stream(
        &self,
        stream_reader: StreamReader,
        project_analysis: &mut ProjectAnalysis,
    ) -> Result<()> {
        let metrics_result = task::spawn_blocking(move || {
            let decoder = GzDecoder::new(stream_reader);
            let mut archive = Archive::new(decoder);

            let entries = archive.entries().map_err(|e| {
                AnalysisError::archive(format!("Failed to read tar entries: {}", e))
            })?;

            let mut collected_metrics = Vec::new();

            for entry in entries {
                let entry = entry.map_err(|e| {
                    AnalysisError::archive(format!("Failed to read tar entry: {}", e))
                })?;

                if let Ok(metrics) = Self::process_tar_entry_sync(entry) {
                    collected_metrics.push(metrics);
                }
            }

            Ok::<Vec<FileMetrics>, AnalysisError>(collected_metrics)
        })
        .await
        .map_err(|e| AnalysisError::archive(format!("Task join error: {}", e)))??;

        for metrics in metrics_result {
            project_analysis.add_file_metrics(metrics)?;
        }

        Ok(())
    }

    fn process_tar_entry_sync<R: Read>(mut entry: tar::Entry<'_, R>) -> Result<FileMetrics> {
        let header = entry.header();
        let path = header
            .path()
            .map_err(|e| AnalysisError::archive(format!("Invalid path in tar entry: {}", e)))?;

        let file_path = path.to_string_lossy().to_string();

        if !header.entry_type().is_file() || header.size().unwrap_or(0) == 0 {
            return Err(AnalysisError::archive("Not a file or empty".to_string()));
        }

        let file_size = header.size().unwrap_or(0);
        let language = LanguageRegistry::detect_by_path(&file_path)
            .map(|l| l.name.clone())
            .unwrap_or_else(|| "Text".to_string());

        let mut content = String::new();
        if entry.read_to_string(&mut content).is_err() {
            return Err(AnalysisError::archive(
                "Failed to read file content".to_string(),
            ));
        }

        analyze_file_content(&file_path, &content, &language, file_size)
    }

    fn extract_project_name_from_original(&self, url: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") {
            let url = url.trim_end_matches('/');

            if url.contains("/tree/") {
                let parts: Vec<&str> = url.split('/').collect();
                if let Some(tree_pos) = parts.iter().position(|&x| x == "tree") {
                    if tree_pos > 1 {
                        let repo = parts[tree_pos - 1];
                        let branch = parts.get(tree_pos + 1).unwrap_or(&"unknown");
                        return format!("{}@{}", repo, branch);
                    }
                }
            }

            if url.contains("/commit/") {
                let parts: Vec<&str> = url.split('/').collect();
                if let Some(commit_pos) = parts.iter().position(|&x| x == "commit") {
                    if commit_pos > 1 {
                        let repo = parts[commit_pos - 1];
                        let commit = parts.get(commit_pos + 1).unwrap_or(&"unknown");
                        return format!("{}@{}", repo, &commit[..7.min(commit.len())]);
                    }
                }
            }

            let parts: Vec<&str> = url.split('/').collect();
            if parts.len() >= 2 {
                let repo = parts[parts.len() - 1];
                return format!("{}@main", repo);
            }
        } else if url.contains('/') && !url.contains('.') {
            let parts: Vec<&str> = url.split('@').collect();
            let repo_part = parts[0];
            let branch = parts.get(1).unwrap_or(&"main");

            if let Some(repo_name) = repo_part.split('/').last() {
                return format!("{}@{}", repo_name, branch);
            }
        }

        "remote-project".to_string()
    }

    #[allow(dead_code)]
    fn extract_project_name(&self, url: &str) -> String {
        let url_path = url.trim_end_matches('/');

        if let Some(filename) = url_path.split('/').last() {
            if filename.ends_with(".tar.gz") {
                return filename.trim_end_matches(".tar.gz").to_string();
            }
            if filename.ends_with(".tgz") {
                return filename.trim_end_matches(".tgz").to_string();
            }
            return filename.to_string();
        }

        "remote-project".to_string()
    }

    fn format_bytes_simple(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
        const THRESHOLD: f64 = 1024.0;

        if bytes == 0 {
            return "0 B".to_string();
        }

        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
            size /= THRESHOLD;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}

impl Default for RemoteAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

fn analyze_file_content(
    file_path: &str,
    content: &str,
    language: &str,
    file_size: u64,
) -> Result<FileMetrics> {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let mut code_lines = 0;
    let mut comment_lines = 0;
    let mut blank_lines = 0;

    let lang_def = LanguageRegistry::get_language(language);
    let empty_line_comments = vec![];
    let empty_multi_line_comments = vec![];
    let line_comments = lang_def
        .map(|l| &l.line_comments)
        .unwrap_or(&empty_line_comments);
    let multi_line_comments = lang_def
        .map(|l| &l.multi_line_comments)
        .unwrap_or(&empty_multi_line_comments);

    let mut in_multi_line_comment = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            blank_lines += 1;
            continue;
        }

        let mut is_comment = false;

        if !in_multi_line_comment {
            for comment_start in line_comments {
                if trimmed.starts_with(comment_start) {
                    is_comment = true;
                    break;
                }
            }

            for (start, end) in multi_line_comments {
                if trimmed.starts_with(start) {
                    is_comment = true;
                    if !trimmed.ends_with(end) {
                        in_multi_line_comment = true;
                    }
                    break;
                }
            }
        } else {
            is_comment = true;
            for (_, end) in multi_line_comments {
                if trimmed.ends_with(end) {
                    in_multi_line_comment = false;
                    break;
                }
            }
        }

        if is_comment {
            comment_lines += 1;
        } else {
            code_lines += 1;
        }
    }

    let metrics = FileMetrics::new(
        file_path,
        language.to_string(),
        total_lines,
        code_lines,
        comment_lines,
        blank_lines,
    )?
    .with_size_bytes(file_size);

    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_url_parsing() {
        let analyzer = RemoteAnalyzer::new();

        assert_eq!(
            analyzer.parse_github_url("https://github.com/user/repo", "main"),
            Some("https://github.com/user/repo/archive/refs/heads/main.tar.gz".to_string())
        );

        assert_eq!(
            analyzer.parse_github_url("https://github.com/user/repo/commit/abc123", "main"),
            Some("https://github.com/user/repo/archive/abc123.tar.gz".to_string())
        );
    }

    #[test]
    fn test_bitbucket_url_parsing() {
        let analyzer = RemoteAnalyzer::new();

        assert_eq!(
            analyzer.parse_bitbucket_url("https://bitbucket.org/user/repo", "main"),
            Some("https://bitbucket.org/user/repo/get/main.tar.gz".to_string())
        );
    }

    #[test]
    fn test_codeberg_url_parsing() {
        let analyzer = RemoteAnalyzer::new();

        assert_eq!(
            analyzer.parse_codeberg_url("https://codeberg.org/user/repo", "main"),
            Some("https://codeberg.org/user/repo/archive/main.tar.gz".to_string())
        );
    }

    #[test]
    fn test_extract_project_name() {
        let analyzer = RemoteAnalyzer::new();

        assert_eq!(
            analyzer.extract_project_name("https://example.com/project.tar.gz"),
            "project"
        );

        assert_eq!(
            analyzer.extract_project_name("https://github.com/user/repo/archive/main.tar.gz"),
            "main"
        );
    }
}

use tokio::sync::mpsc;

struct StreamReader {
    receiver: mpsc::Receiver<std::io::Result<bytes::Bytes>>,
    current_chunk: Option<Cursor<bytes::Bytes>>,
    finished: bool,
}

impl StreamReader {
    fn new(
        stream: impl futures_util::Stream<Item = reqwest::Result<bytes::Bytes>> + Send + 'static,
        #[cfg(feature = "cli")] progress_bar: Option<ProgressBar>,
        total_size: Option<u64>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut downloaded = 0u64;
            let mut stream = Box::pin(stream);

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        downloaded += chunk.len() as u64;

                        #[cfg(feature = "cli")]
                        if let Some(pb) = &progress_bar {
                            if let Some(_total) = total_size {
                                pb.set_position(downloaded);
                            } else {
                                let formatted = RemoteAnalyzer::format_bytes_simple(downloaded);
                                pb.set_message(format!("Downloaded {}...", formatted));
                            }
                        }

                        if tx.send(Ok(chunk)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Stream error: {}", e),
                            )))
                            .await;
                        break;
                    }
                }
            }
        });

        Self {
            receiver: rx,
            current_chunk: None,
            finished: false,
        }
    }
}

impl Read for StreamReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(ref mut cursor) = self.current_chunk {
            let read = cursor.read(buf)?;
            if read > 0 {
                return Ok(read);
            }
            self.current_chunk = None;
        }

        if self.finished {
            return Ok(0);
        }

        match self.receiver.try_recv() {
            Ok(Ok(chunk)) => {
                self.current_chunk = Some(Cursor::new(chunk));
                if let Some(ref mut cursor) = self.current_chunk {
                    cursor.read(buf)
                } else {
                    Ok(0)
                }
            }
            Ok(Err(e)) => {
                self.finished = true;
                Err(e)
            }
            Err(mpsc::error::TryRecvError::Empty) => match self.receiver.blocking_recv() {
                Some(Ok(chunk)) => {
                    self.current_chunk = Some(Cursor::new(chunk));
                    if let Some(ref mut cursor) = self.current_chunk {
                        cursor.read(buf)
                    } else {
                        Ok(0)
                    }
                }
                Some(Err(e)) => {
                    self.finished = true;
                    Err(e)
                }
                None => {
                    self.finished = true;
                    Ok(0)
                }
            },
            Err(mpsc::error::TryRecvError::Disconnected) => {
                self.finished = true;
                Ok(0)
            }
        }
    }
}
