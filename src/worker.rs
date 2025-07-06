use wasm_bindgen::prelude::*;
use web_sys::{Request, RequestInit, RequestMode, Response};

use crate::core::{
    ProjectAnalysis,
    analysis::{FileMetrics, LanguageAnalysis},
};
use flate2::read::GzDecoder;
use std::io::{Cursor, Read};
use tar::Archive;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AnalysisOptions {
    pub ignore_hidden: bool,
    pub ignore_gitignore: bool,
    pub max_file_size: i64,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = fetch)]
    fn fetch_with_request(request: &Request) -> js_sys::Promise;
}

#[wasm_bindgen]
pub async fn analyze_url(url: String, options: JsValue) -> Result<JsValue, JsValue> {
    let opts: AnalysisOptions = serde_wasm_bindgen::from_value(options)?;

    web_sys::console::log_1(&format!("Starting real tar.gz analysis for URL: {}", url).into());

    let project_name = extract_project_name(&url);
    let mut result = ProjectAnalysis::new(project_name);

    match resolve_download_url(&url).await {
        Ok(download_urls) => {
            web_sys::console::log_1(
                &format!("Resolved {} download URLs", download_urls.len()).into(),
            );

            let mut last_error = String::new();
            for download_url in &download_urls {
                web_sys::console::log_1(&format!("Trying download URL: {}", download_url).into());

                match download_and_analyze_tar(download_url, &url, &opts).await {
                    Ok(analysis) => {
                        result = analysis;
                        web_sys::console::log_1(
                            &format!(
                                "Tar analysis completed successfully for project: {}",
                                result.project_name
                            )
                            .into(),
                        );
                        break;
                    }
                    Err(e) => {
                        web_sys::console::log_1(
                            &format!("URL {} failed: {}", download_url, e).into(),
                        );
                        last_error = e;
                        continue;
                    }
                }
            }

            if result.global_metrics.file_count == 0 {
                web_sys::console::log_1(
                    &format!("All download URLs failed. Last error: {}", last_error).into(),
                );
                return Err(JsValue::from_str(&last_error));
            }
        }
        Err(e) => {
            web_sys::console::log_1(&format!("URL resolution failed: {}", e).into());
            return Err(JsValue::from_str(&e));
        }
    }

    #[derive(serde::Serialize)]
    struct SimplifiedLanguageInfo {
        language_name: String,
        file_count: usize,
        total_lines: usize,
    }

    #[derive(serde::Serialize)]
    struct DebugResult {
        project_name: String,
        languages: Vec<SimplifiedLanguageInfo>,
        global_metrics: crate::core::analysis::AggregateMetrics,
        debug_info: String,
    }

    let mut languages = Vec::new();
    for (lang_name, lang_analysis) in &result.language_analyses {
        languages.push(SimplifiedLanguageInfo {
            language_name: lang_name.clone(),
            file_count: lang_analysis.file_metrics.len(),
            total_lines: lang_analysis.aggregate_metrics.total_lines,
        });
    }

    let debug_result = DebugResult {
        project_name: result.project_name.clone(),
        languages,
        global_metrics: result.global_metrics.clone(),
        debug_info: format!(
            "Languages: {}, Files: {}",
            result.language_analyses.len(),
            result.global_metrics.file_count
        ),
    };

    Ok(serde_wasm_bindgen::to_value(&debug_result)?)
}

async fn resolve_download_url(url: &str) -> Result<Vec<String>, String> {
    if url.contains("github.com") {
        if let Some(download_urls) = parse_github_url(url) {
            return Ok(download_urls);
        }
    }

    if url.contains("gitlab.") {
        if let Some(download_url) = parse_gitlab_url(url) {
            return Ok(vec![download_url]);
        }
    }

    if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
        return Ok(vec![url.to_string()]);
    }

    Err(format!("Unsupported URL format: {}", url))
}

fn parse_github_url(url: &str) -> Option<Vec<String>> {
    let parts: Vec<&str> = url.split('/').collect();
    let mut urls = Vec::new();

    if url.contains("/tree/") {
        if let Some(tree_pos) = parts.iter().position(|&x| x == "tree") {
            if tree_pos + 1 < parts.len() && tree_pos >= 2 {
                let owner = parts[tree_pos - 2];
                let repo = parts[tree_pos - 1];
                let branch = parts[tree_pos + 1];
                urls.push(format!(
                    "https://github.com/{}/{}/archive/refs/heads/{}.tar.gz",
                    owner, repo, branch
                ));
                return Some(urls);
            }
        }
    }

    if let Some(github_pos) = parts.iter().position(|&x| x == "github.com") {
        if github_pos + 2 < parts.len() {
            let owner = parts[github_pos + 1];
            let repo = parts[github_pos + 2];
            urls.push(format!(
                "https://github.com/{}/{}/archive/refs/heads/main.tar.gz",
                owner, repo
            ));
            urls.push(format!(
                "https://github.com/{}/{}/archive/refs/heads/master.tar.gz",
                owner, repo
            ));
            return Some(urls);
        }
    }

    None
}

fn parse_gitlab_url(url: &str) -> Option<String> {
    let parts: Vec<&str> = url.split('/').collect();

    if let Some(gitlab_pos) = parts.iter().position(|&x| x.contains("gitlab")) {
        if gitlab_pos + 2 < parts.len() {
            let host = parts[gitlab_pos];
            let owner = parts[gitlab_pos + 1];
            let repo = parts[gitlab_pos + 2];
            return Some(format!(
                "https://{}/{}{}/-/archive/main/{}-main.tar.gz",
                host,
                owner,
                "/".to_string() + repo,
                repo
            ));
        }
    }

    None
}

async fn download_and_analyze_tar(
    download_url: &str,
    original_url: &str,
    opts: &AnalysisOptions,
) -> Result<ProjectAnalysis, String> {
    let req_init = RequestInit::new();
    req_init.set_method("GET");
    req_init.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(download_url, &req_init)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;

    let headers = request.headers();
    headers
        .set("User-Agent", "bytes-radar/1.0")
        .map_err(|e| format!("Failed to set User-Agent: {:?}", e))?;
    headers
        .set("Accept", "application/octet-stream")
        .map_err(|e| format!("Failed to set Accept: {:?}", e))?;

    web_sys::console::log_1(&format!("Fetching: {}", download_url).into());

    let resp_promise = fetch_with_request(&request);
    let resp: Response = wasm_bindgen_futures::JsFuture::from(resp_promise)
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?
        .into();

    web_sys::console::log_1(&format!("Response status: {}", resp.status()).into());

    if !resp.ok() {
        return Err(format!(
            "HTTP request failed with status: {}. URL: {}",
            resp.status(),
            download_url
        ));
    }

    let array_buffer = wasm_bindgen_futures::JsFuture::from(
        resp.array_buffer()
            .map_err(|e| format!("Failed to get array buffer: {:?}", e))?,
    )
    .await
    .map_err(|e| format!("Failed to read array buffer: {:?}", e))?;

    let bytes = js_sys::Uint8Array::new(&array_buffer).to_vec();

    web_sys::console::log_1(
        &format!(
            "Downloaded {} bytes, starting tar.gz extraction",
            bytes.len()
        )
        .into(),
    );

    let project_name = extract_project_name(original_url);
    let mut analysis = ProjectAnalysis::new(project_name);

    match process_tarball_sync(&bytes, &mut analysis, opts) {
        Ok(_) => {
            web_sys::console::log_1(
                &format!(
                    "Successfully processed {} files",
                    analysis.global_metrics.file_count
                )
                .into(),
            );
            web_sys::console::log_1(
                &format!(
                    "Language analyses count: {}",
                    analysis.language_analyses.len()
                )
                .into(),
            );
            Ok(analysis)
        }
        Err(e) => {
            web_sys::console::log_1(&format!("Tar processing failed: {}", e).into());
            Err(format!("Tar processing failed: {}", e))
        }
    }
}

fn process_tarball_sync(
    bytes: &[u8],
    project_analysis: &mut ProjectAnalysis,
    opts: &AnalysisOptions,
) -> Result<(), String> {
    web_sys::console::log_1(&format!("Starting tar processing with {} bytes", bytes.len()).into());

    let cursor = Cursor::new(bytes);

    let decoder = GzDecoder::new(cursor);

    let mut archive = Archive::new(decoder);

    web_sys::console::log_1(&"Created tar archive, reading entries...".into());

    let entries = archive
        .entries()
        .map_err(|e| format!("Failed to read tar entries: {}", e))?;

    let mut processed_count = 0;
    let mut total_entries = 0;
    for entry in entries {
        total_entries += 1;
        if total_entries % 1000 == 0 {
            web_sys::console::log_1(&format!("Processing entry {}", total_entries).into());
        }

        let entry = entry.map_err(|e| format!("Failed to read tar entry: {}", e))?;

        match process_tar_entry_sync(entry, opts) {
            Ok(Some(metrics)) => {
                let language = metrics.language.clone();
                let mut lang_analysis = LanguageAnalysis::new(language.clone());

                if let Err(_) = lang_analysis.add_file_metrics(metrics.clone()) {
                    project_analysis
                        .language_analyses
                        .insert(language.clone(), LanguageAnalysis::new(language));
                } else {
                    project_analysis
                        .language_analyses
                        .insert(language, lang_analysis);
                }

                project_analysis.global_metrics.incorporate(&metrics);

                processed_count += 1;
            }
            Ok(None) => {}
            Err(e) => {
                web_sys::console::log_1(&format!("Warning: Failed to process file: {}", e).into());
            }
        }
    }

    Ok(())
}

fn process_tar_entry_sync<R: Read>(
    mut entry: tar::Entry<'_, R>,
    opts: &AnalysisOptions,
) -> Result<Option<FileMetrics>, String> {
    let header = entry.header();

    let path = entry
        .path()
        .map_err(|e| format!("Failed to get entry path: {}", e))?;
    let file_path = path.to_string_lossy().to_string();

    if header.entry_type().is_dir() {
        return Ok(None);
    }

    if opts.ignore_hidden {
        if let Some(filename) = path.file_name() {
            if filename.to_string_lossy().starts_with('.') {
                return Ok(None);
            }
        }
    }

    let file_size = header
        .size()
        .map_err(|e| format!("Failed to get file size: {}", e))?;

    if opts.max_file_size > 0 && file_size > opts.max_file_size as u64 {
        return Ok(None);
    }

    let mut content = String::new();
    match entry.read_to_string(&mut content) {
        Ok(_) => {
            let language = detect_language(&file_path);
            analyze_file_content(&file_path, &content, &language, file_size).map(Some)
        }
        Err(_) => Ok(None),
    }
}

fn detect_language(file_path: &str) -> String {
    let path = std::path::Path::new(file_path);

    if let Some(extension) = path.extension() {
        match extension.to_string_lossy().to_lowercase().as_str() {
            "rs" => "Rust".to_string(),
            "js" | "mjs" => "JavaScript".to_string(),
            "ts" => "TypeScript".to_string(),
            "py" => "Python".to_string(),
            "java" => "Java".to_string(),
            "cpp" | "cc" | "cxx" => "C++".to_string(),
            "c" => "C".to_string(),
            "h" | "hpp" => "C/C++ Header".to_string(),
            "go" => "Go".to_string(),
            "php" => "PHP".to_string(),
            "rb" => "Ruby".to_string(),
            "cs" => "C#".to_string(),
            "html" | "htm" => "HTML".to_string(),
            "css" => "CSS".to_string(),
            "json" => "JSON".to_string(),
            "xml" => "XML".to_string(),
            "yml" | "yaml" => "YAML".to_string(),
            "md" => "Markdown".to_string(),
            "txt" => "Text".to_string(),
            _ => "Unknown".to_string(),
        }
    } else {
        "Unknown".to_string()
    }
}

fn analyze_file_content(
    file_path: &str,
    content: &str,
    language: &str,
    file_size: u64,
) -> Result<FileMetrics, String> {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let mut code_lines = 0usize;
    let mut comment_lines = 0usize;
    let mut blank_lines = 0usize;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blank_lines += 1;
        } else if is_comment_line(trimmed, language) {
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
    )
    .map_err(|e| format!("Failed to create file metrics: {}", e))?;

    Ok(metrics.with_size_bytes(file_size))
}

fn is_comment_line(line: &str, language: &str) -> bool {
    match language {
        "Rust" | "JavaScript" | "TypeScript" | "C" | "C++" | "Java" | "C#" | "Go" | "PHP" => {
            line.starts_with("//") || line.starts_with("/*") || line.starts_with("*")
        }
        "Python" | "Ruby" => line.starts_with("#"),
        "HTML" | "XML" => line.starts_with("<!--"),
        "CSS" => line.starts_with("/*"),
        _ => false,
    }
}

fn extract_project_name(url: &str) -> String {
    if url.contains("github.com") {
        let parts: Vec<&str> = url.split('/').collect();
        if let Some(repo_name) = parts.last() {
            if !repo_name.is_empty() && *repo_name != "/" {
                return repo_name.to_string();
            }
        }
        if parts.len() >= 2 {
            return parts[parts.len() - 1].to_string();
        }
    }

    if url.contains("gitlab.") || url.contains("bitbucket.") || url.contains("codeberg.") {
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() >= 2 {
            return parts[parts.len() - 1].to_string();
        }
    }

    "remote-project".to_string()
}

fn extract_project_name_from_url(url: &str) -> String {
    if let Some(last_slash) = url.rfind('/') {
        let filename = &url[last_slash + 1..];
        if let Some(dot_pos) = filename.find('.') {
            return filename[..dot_pos].to_string();
        }
    }
    "downloaded-project".to_string()
}
