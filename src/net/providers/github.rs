use crate::net::traits::{GitProvider, ParsedRepository, ProviderConfig};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct GitHubRepoInfo {
    default_branch: String,
}

pub struct GitHubProvider {
    token: Option<String>,
}

impl GitHubProvider {
    pub fn new() -> Self {
        Self { token: None }
    }
}

#[async_trait]
impl GitProvider for GitHubProvider {
    fn name(&self) -> &'static str {
        "github"
    }

    fn can_handle(&self, url: &str) -> bool {
        url.contains("github.com")
    }

    fn parse_url(&self, url: &str) -> Option<ParsedRepository> {
        if !self.can_handle(url) {
            return None;
        }

        let url = url.trim_end_matches('/');

        if url.contains("/tree/") {
            return self.parse_tree_url(url);
        }

        if url.contains("/commit/") {
            return self.parse_commit_url(url);
        }

        self.parse_basic_url(url)
    }

    fn build_download_urls(&self, parsed: &ParsedRepository) -> Vec<String> {
        let mut urls = Vec::new();

        if let Some(ref branch_or_commit) = parsed.branch_or_commit {
            if parsed.is_commit {
                urls.push(format!(
                    "https://github.com/{}/{}/archive/{}.tar.gz",
                    parsed.owner, parsed.repo, branch_or_commit
                ));
            } else {
                urls.push(format!(
                    "https://github.com/{}/{}/archive/refs/heads/{}.tar.gz",
                    parsed.owner, parsed.repo, branch_or_commit
                ));
                urls.push(format!(
                    "https://github.com/{}/{}/archive/refs/tags/{}.tar.gz",
                    parsed.owner, parsed.repo, branch_or_commit
                ));
            }
        }

        urls
    }

    async fn get_default_branch(
        &self,
        client: &Client,
        parsed: &ParsedRepository,
    ) -> Option<String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let api_url = format!(
                "https://api.github.com/repos/{}/{}",
                parsed.owner, parsed.repo
            );

            let mut request = client.get(&api_url);

            if let Some(ref token) = self.token {
                request = request.header("Authorization", format!("token {}", token));
            }

            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<GitHubRepoInfo>().await {
                            Ok(repo_info) => {
                                #[cfg(feature = "cli")]
                                log::debug!(
                                    "GitHub API: Found default branch '{}' for {}/{}",
                                    repo_info.default_branch,
                                    parsed.owner,
                                    parsed.repo
                                );
                                Some(repo_info.default_branch)
                            }
                            Err(_) => {
                                #[cfg(feature = "cli")]
                                log::debug!(
                                    "GitHub API: Failed to parse response for {}/{}",
                                    parsed.owner,
                                    parsed.repo
                                );
                                None
                            }
                        }
                    } else {
                        #[cfg(feature = "cli")]
                        log::debug!(
                            "GitHub API: Request failed with status {} for {}/{}",
                            response.status(),
                            parsed.owner,
                            parsed.repo
                        );
                        None
                    }
                }
                Err(_) => {
                    #[cfg(feature = "cli")]
                    log::debug!(
                        "GitHub API: Network error for {}/{}",
                        parsed.owner,
                        parsed.repo
                    );
                    None
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        None
    }

    fn apply_config(&mut self, config: &ProviderConfig) {
        self.token = config.credentials.get("token").cloned();
    }

    fn get_project_name(&self, url: &str) -> String {
        if let Some(parsed) = self.parse_url(url) {
            return parsed.project_name;
        }

        if let Some(filename) = url.split('/').last() {
            if filename.ends_with(".tar.gz") {
                return filename.trim_end_matches(".tar.gz").to_string();
            }
            if filename.ends_with(".tgz") {
                return filename.trim_end_matches(".tgz").to_string();
            }
            return filename.to_string();
        }

        "github-project".to_string()
    }
}

impl GitHubProvider {
    fn parse_tree_url(&self, url: &str) -> Option<ParsedRepository> {
        let parts: Vec<&str> = url.split('/').collect();
        if let Some(tree_pos) = parts.iter().position(|&x| x == "tree") {
            if tree_pos + 1 < parts.len() && tree_pos >= 2 {
                let owner = parts[tree_pos - 2].to_string();
                let repo = parts[tree_pos - 1].to_string();
                let branch = parts[tree_pos + 1].to_string();

                return Some(
                    ParsedRepository::new(owner, repo)
                        .with_branch(branch)
                        .with_host("github.com".to_string()),
                );
            }
        }
        None
    }

    fn parse_commit_url(&self, url: &str) -> Option<ParsedRepository> {
        let parts: Vec<&str> = url.split('/').collect();
        if let Some(commit_pos) = parts.iter().position(|&x| x == "commit") {
            if commit_pos + 1 < parts.len() && commit_pos >= 2 {
                let owner = parts[commit_pos - 2].to_string();
                let repo = parts[commit_pos - 1].to_string();
                let commit = parts[commit_pos + 1].to_string();

                return Some(
                    ParsedRepository::new(owner, repo)
                        .with_commit(commit)
                        .with_host("github.com".to_string()),
                );
            }
        }
        None
    }

    fn parse_basic_url(&self, url: &str) -> Option<ParsedRepository> {
        let parts: Vec<&str> = url.split('/').collect();
        if let Some(github_pos) = parts.iter().position(|&x| x == "github.com") {
            if github_pos + 2 < parts.len() {
                let owner = parts[github_pos + 1].to_string();
                let repo = parts[github_pos + 2].to_string();

                return Some(
                    ParsedRepository::new(owner, repo).with_host("github.com".to_string()),
                );
            }
        }

        if let Some(stripped) = url.strip_prefix("https://github.com/") {
            let parts: Vec<&str> = stripped.split('/').collect();
            if parts.len() >= 2 {
                let owner = parts[0].to_string();
                let repo = parts[1].to_string();

                return Some(
                    ParsedRepository::new(owner, repo).with_host("github.com".to_string()),
                );
            }
        }

        None
    }
}

impl Default for GitHubProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let provider = GitHubProvider::new();
        assert!(provider.can_handle("https://github.com/user/repo"));
        assert!(provider.can_handle("https://github.com/user/repo/tree/main"));
        assert!(!provider.can_handle("https://gitlab.com/user/repo"));
    }

    #[test]
    fn test_parse_basic_url() {
        let provider = GitHubProvider::new();

        let parsed = provider.parse_url("https://github.com/user/repo").unwrap();
        assert_eq!(parsed.owner, "user");
        assert_eq!(parsed.repo, "repo");
        assert_eq!(parsed.project_name, "repo@main");
        assert_eq!(parsed.branch_or_commit, None);
        assert!(!parsed.is_commit);
        assert_eq!(parsed.host.as_ref().unwrap(), "github.com");
    }

    #[test]
    fn test_parse_tree_url() {
        let provider = GitHubProvider::new();

        let parsed = provider
            .parse_url("https://github.com/user/repo/tree/develop")
            .unwrap();
        assert_eq!(parsed.owner, "user");
        assert_eq!(parsed.repo, "repo");
        assert_eq!(parsed.project_name, "repo@develop");
        assert_eq!(parsed.branch_or_commit, Some("develop".to_string()));
        assert!(!parsed.is_commit);
    }

    #[test]
    fn test_parse_commit_url() {
        let provider = GitHubProvider::new();

        let parsed = provider
            .parse_url("https://github.com/user/repo/commit/abc1234567890")
            .unwrap();
        assert_eq!(parsed.owner, "user");
        assert_eq!(parsed.repo, "repo");
        assert_eq!(parsed.project_name, "repo@abc1234");
        assert_eq!(parsed.branch_or_commit, Some("abc1234567890".to_string()));
        assert!(parsed.is_commit);
    }

    #[test]
    fn test_build_download_urls() {
        let provider = GitHubProvider::new();

        let parsed = ParsedRepository::new("user".to_string(), "repo".to_string())
            .with_branch("main".to_string());

        let urls = provider.build_download_urls(&parsed);
        assert!(urls
            .contains(&"https://github.com/user/repo/archive/refs/heads/main.tar.gz".to_string()));
        assert!(urls
            .contains(&"https://github.com/user/repo/archive/refs/tags/main.tar.gz".to_string()));
    }
}
