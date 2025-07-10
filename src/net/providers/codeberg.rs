use crate::net::traits::{GitProvider, ParsedRepository, ProviderConfig};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

pub struct CodebergProvider {
    credentials: HashMap<String, String>,
}

impl CodebergProvider {
    pub fn new() -> Self {
        Self {
            credentials: HashMap::new(),
        }
    }
}

#[async_trait]
impl GitProvider for CodebergProvider {
    fn name(&self) -> &'static str {
        "codeberg"
    }

    fn can_handle(&self, url: &str) -> bool {
        url.contains("codeberg.org")
    }

    fn parse_url(&self, url: &str) -> Option<ParsedRepository> {
        if !self.can_handle(url) {
            return None;
        }

        let url = url.trim_end_matches('/');

        if url.contains("/commit/") {
            return self.parse_commit_url(url);
        }

        if url.contains("/src/branch/") {
            return self.parse_branch_url(url);
        }

        self.parse_basic_url(url)
    }

    fn build_download_urls(&self, parsed: &ParsedRepository) -> Vec<String> {
        let mut urls = Vec::new();

        if let Some(ref branch_or_commit) = parsed.branch_or_commit {
            urls.push(format!(
                "https://codeberg.org/{}/{}/archive/{}.tar.gz",
                parsed.owner, parsed.repo, branch_or_commit
            ));
        }

        urls
    }

    async fn get_default_branch(
        &self,
        _client: &Client,
        _parsed: &ParsedRepository,
    ) -> Option<String> {
        None
    }

    fn apply_config(&mut self, config: &ProviderConfig) {
        self.credentials = config.credentials.clone();
    }

    fn get_project_name(&self, url: &str) -> String {
        if let Some(parsed) = self.parse_url(url) {
            return parsed.project_name;
        }

        if let Some(filename) = url.split('/').next_back() {
            if filename.ends_with(".tar.gz") {
                return filename.trim_end_matches(".tar.gz").to_string();
            }
            if filename.ends_with(".tgz") {
                return filename.trim_end_matches(".tgz").to_string();
            }
            return filename.to_string();
        }

        "codeberg-project".to_string()
    }
}

impl CodebergProvider {
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
                        .with_host("codeberg.org".to_string()),
                );
            }
        }
        None
    }

    fn parse_branch_url(&self, url: &str) -> Option<ParsedRepository> {
        let parts: Vec<&str> = url.split('/').collect();
        if let Some(branch_pos) = parts.iter().position(|&x| x == "branch") {
            if branch_pos + 1 < parts.len() && branch_pos >= 3 {
                let owner = parts[branch_pos - 3].to_string();
                let repo = parts[branch_pos - 2].to_string();
                let branch = parts[branch_pos + 1].to_string();

                return Some(
                    ParsedRepository::new(owner, repo)
                        .with_branch(branch)
                        .with_host("codeberg.org".to_string()),
                );
            }
        }
        None
    }

    fn parse_basic_url(&self, url: &str) -> Option<ParsedRepository> {
        let parts: Vec<&str> = url.split('/').collect();
        if let Some(codeberg_pos) = parts.iter().position(|&x| x == "codeberg.org") {
            if codeberg_pos + 2 < parts.len() {
                let owner = parts[codeberg_pos + 1].to_string();
                let repo = parts[codeberg_pos + 2].to_string();

                return Some(
                    ParsedRepository::new(owner, repo).with_host("codeberg.org".to_string()),
                );
            }
        }
        None
    }
}

impl Default for CodebergProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let provider = CodebergProvider::new();
        assert!(provider.can_handle("https://codeberg.org/user/repo"));
        assert!(provider.can_handle("https://codeberg.org/user/repo/commit/abc123"));
        assert!(!provider.can_handle("https://github.com/user/repo"));
    }

    #[test]
    fn test_parse_basic_url() {
        let provider = CodebergProvider::new();

        let parsed = provider
            .parse_url("https://codeberg.org/user/repo")
            .unwrap();
        assert_eq!(parsed.owner, "user");
        assert_eq!(parsed.repo, "repo");
        assert_eq!(parsed.project_name, "repo@main");
        assert_eq!(parsed.branch_or_commit, None);
        assert!(!parsed.is_commit);
        assert_eq!(parsed.host.as_ref().unwrap(), "codeberg.org");
    }

    #[test]
    fn test_parse_branch_url() {
        let provider = CodebergProvider::new();

        let parsed = provider
            .parse_url("https://codeberg.org/user/repo/src/branch/develop")
            .unwrap();
        assert_eq!(parsed.owner, "user");
        assert_eq!(parsed.repo, "repo");
        assert_eq!(parsed.project_name, "repo@develop");
        assert_eq!(parsed.branch_or_commit, Some("develop".to_string()));
        assert!(!parsed.is_commit);
    }

    #[test]
    fn test_parse_commit_url() {
        let provider = CodebergProvider::new();

        let parsed = provider
            .parse_url("https://codeberg.org/user/repo/commit/abc1234567890")
            .unwrap();
        assert_eq!(parsed.owner, "user");
        assert_eq!(parsed.repo, "repo");
        assert_eq!(parsed.project_name, "repo@abc1234");
        assert_eq!(parsed.branch_or_commit, Some("abc1234567890".to_string()));
        assert!(parsed.is_commit);
    }

    #[test]
    fn test_build_download_urls() {
        let provider = CodebergProvider::new();

        let parsed = ParsedRepository::new("user".to_string(), "repo".to_string())
            .with_branch("main".to_string());

        let urls = provider.build_download_urls(&parsed);
        assert!(urls.contains(&"https://codeberg.org/user/repo/archive/main.tar.gz".to_string()));
    }
}
