use crate::net::traits::{GitProvider, ParsedRepository, ProviderConfig};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

pub struct SourceForgeProvider {
    credentials: HashMap<String, String>,
}

impl SourceForgeProvider {
    pub fn new() -> Self {
        Self {
            credentials: HashMap::new(),
        }
    }
}

#[async_trait]
impl GitProvider for SourceForgeProvider {
    fn name(&self) -> &'static str {
        "sourceforge"
    }

    fn can_handle(&self, url: &str) -> bool {
        url.contains("sourceforge.net") || url.contains("sf.net")
    }

    fn parse_url(&self, url: &str) -> Option<ParsedRepository> {
        if !self.can_handle(url) {
            return None;
        }

        let url = url.trim_end_matches('/');

        if url.contains("/ci/") && url.contains("/tree/") {
            return self.parse_tree_url(url);
        }

        if url.contains("/ci/") {
            return self.parse_commit_url(url);
        }

        self.parse_basic_url(url)
    }

    fn build_download_urls(&self, parsed: &ParsedRepository) -> Vec<String> {
        let mut urls = Vec::new();

        if let Some(ref branch_or_commit) = parsed.branch_or_commit {
            urls.push(format!(
                "https://sourceforge.net/p/{}/code/ci/{}/tarball",
                parsed.repo, branch_or_commit
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

        if let Some(filename) = url.split('/').last() {
            if filename.ends_with(".tar.gz") {
                return filename.trim_end_matches(".tar.gz").to_string();
            }
            if filename.ends_with(".tgz") {
                return filename.trim_end_matches(".tgz").to_string();
            }
            return filename.to_string();
        }

        "sourceforge-project".to_string()
    }
}

impl SourceForgeProvider {
    fn parse_commit_url(&self, url: &str) -> Option<ParsedRepository> {
        let parts: Vec<&str> = url.split('/').collect();
        if let Some(ci_pos) = parts.iter().position(|&x| x == "ci") {
            if ci_pos + 1 < parts.len() && ci_pos >= 3 {
                let project = parts[ci_pos - 1].to_string();
                let commit = parts[ci_pos + 1].to_string();

                return Some(
                    ParsedRepository::new("sourceforge".to_string(), project)
                        .with_commit(commit)
                        .with_host("sourceforge.net".to_string()),
                );
            }
        }
        None
    }

    fn parse_tree_url(&self, url: &str) -> Option<ParsedRepository> {
        let parts: Vec<&str> = url.split('/').collect();
        if let Some(tree_pos) = parts.iter().position(|&x| x == "tree") {
            if tree_pos + 1 < parts.len() {
                if let Some(ci_pos) = parts.iter().position(|&x| x == "ci") {
                    if ci_pos >= 3 {
                        let project = parts[ci_pos - 1].to_string();
                        let branch = parts[tree_pos + 1].to_string();

                        return Some(
                            ParsedRepository::new("sourceforge".to_string(), project)
                                .with_branch(branch)
                                .with_host("sourceforge.net".to_string()),
                        );
                    }
                }
            }
        }
        None
    }

    fn parse_basic_url(&self, url: &str) -> Option<ParsedRepository> {
        if url.contains("/p/") && url.contains("/code") {
            let parts: Vec<&str> = url.split('/').collect();
            if let Some(p_pos) = parts.iter().position(|&x| x == "p") {
                if p_pos + 1 < parts.len() {
                    let project = parts[p_pos + 1].to_string();
                    return Some(
                        ParsedRepository::new("sourceforge".to_string(), project)
                            .with_host("sourceforge.net".to_string()),
                    );
                }
            }
        }
        None
    }
}

impl Default for SourceForgeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let provider = SourceForgeProvider::new();
        assert!(provider.can_handle("https://sourceforge.net/p/project/code/"));
        assert!(provider.can_handle("https://sf.net/p/project/code/"));
        assert!(!provider.can_handle("https://github.com/user/repo"));
    }

    #[test]
    fn test_parse_basic_url() {
        let provider = SourceForgeProvider::new();

        let parsed = provider
            .parse_url("https://sourceforge.net/p/myproject/code/")
            .unwrap();
        assert_eq!(parsed.owner, "sourceforge");
        assert_eq!(parsed.repo, "myproject");
        assert_eq!(parsed.project_name, "myproject@main");
        assert_eq!(parsed.branch_or_commit, None);
        assert!(!parsed.is_commit);
        assert_eq!(parsed.host.as_ref().unwrap(), "sourceforge.net");
    }

    #[test]
    fn test_build_download_urls() {
        let provider = SourceForgeProvider::new();

        let parsed = ParsedRepository::new("sourceforge".to_string(), "project".to_string())
            .with_branch("master".to_string());

        let urls = provider.build_download_urls(&parsed);
        assert!(
            urls.contains(&"https://sourceforge.net/p/project/code/ci/master/tarball".to_string())
        );
    }
}
