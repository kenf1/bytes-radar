use crate::net::traits::{GitProvider, ParsedRepository, ProviderConfig};
use async_trait::async_trait;
use reqwest::Client;

pub struct GitLabProvider {
    token: Option<String>,
}

impl GitLabProvider {
    pub fn new() -> Self {
        Self { token: None }
    }
}

#[async_trait]
impl GitProvider for GitLabProvider {
    fn name(&self) -> &'static str {
        "gitlab"
    }

    fn can_handle(&self, url: &str) -> bool {
        url.contains("gitlab.com") || url.contains("gitlab.")
    }

    fn parse_url(&self, url: &str) -> Option<ParsedRepository> {
        if !self.can_handle(url) {
            return None;
        }

        let url = url.trim_end_matches('/');

        if url.contains("/-/tree/") {
            return self.parse_tree_url(url);
        }

        if url.contains("/-/commit/") {
            return self.parse_commit_url(url);
        }

        self.parse_basic_url(url)
    }

    fn build_download_urls(&self, parsed: &ParsedRepository) -> Vec<String> {
        let mut urls = Vec::new();

        if let Some(ref branch_or_commit) = parsed.branch_or_commit {
            let host = parsed.host.as_deref().unwrap_or("gitlab.com");

            if parsed.is_commit {
                urls.push(format!(
                    "https://{}/{}/-/archive/{}/{}-{}.tar.gz",
                    host,
                    self.build_project_path(&parsed.owner, &parsed.repo),
                    branch_or_commit,
                    parsed.repo,
                    branch_or_commit
                ));
            } else {
                urls.push(format!(
                    "https://{}/{}/-/archive/{}/{}-{}.tar.gz",
                    host,
                    self.build_project_path(&parsed.owner, &parsed.repo),
                    branch_or_commit,
                    parsed.repo,
                    branch_or_commit
                ));
            }
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
        self.token = config
            .credentials
            .get("token")
            .cloned()
            .or_else(|| config.credentials.get("private_token").cloned());
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

        "gitlab-project".to_string()
    }
}

impl GitLabProvider {
    fn parse_tree_url(&self, url: &str) -> Option<ParsedRepository> {
        let parts: Vec<&str> = url.split('/').collect();
        if let Some(tree_pos) = parts.iter().position(|&x| x == "tree") {
            if tree_pos + 1 < parts.len() && tree_pos >= 3 {
                let gitlab_pos = parts.iter().position(|&x| x.contains("gitlab"))?;
                let host = parts[gitlab_pos].to_string();
                let owner = parts[gitlab_pos + 1].to_string();
                let repo = parts[gitlab_pos + 2].to_string();
                let branch = parts[tree_pos + 1].to_string();

                return Some(
                    ParsedRepository::new(owner, repo)
                        .with_branch(branch)
                        .with_host(host),
                );
            }
        }
        None
    }

    fn parse_commit_url(&self, url: &str) -> Option<ParsedRepository> {
        let parts: Vec<&str> = url.split('/').collect();
        if let Some(commit_pos) = parts.iter().position(|&x| x == "commit") {
            if commit_pos + 1 < parts.len() && commit_pos >= 3 {
                let gitlab_pos = parts.iter().position(|&x| x.contains("gitlab"))?;
                let host = parts[gitlab_pos].to_string();
                let owner = parts[gitlab_pos + 1].to_string();
                let repo = parts[gitlab_pos + 2].to_string();
                let commit = parts[commit_pos + 1].to_string();

                return Some(
                    ParsedRepository::new(owner, repo)
                        .with_commit(commit)
                        .with_host(host),
                );
            }
        }
        None
    }

    fn parse_basic_url(&self, url: &str) -> Option<ParsedRepository> {
        let parts: Vec<&str> = url.split('/').collect();
        if let Some(gitlab_pos) = parts.iter().position(|&x| x.contains("gitlab")) {
            if gitlab_pos + 2 < parts.len() {
                let host = parts[gitlab_pos].to_string();
                let owner = parts[gitlab_pos + 1].to_string();
                let repo = parts[gitlab_pos + 2].to_string();

                return Some(ParsedRepository::new(owner, repo).with_host(host));
            }
        }

        None
    }

    fn build_project_path(&self, owner: &str, repo: &str) -> String {
        if owner.contains("/") {
            format!("{}/{}", owner, repo)
        } else {
            format!("{}/{}", owner, repo)
        }
    }
}

impl Default for GitLabProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let provider = GitLabProvider::new();
        assert!(provider.can_handle("https://gitlab.com/user/repo"));
        assert!(provider.can_handle("https://gitlab.example.com/user/repo"));
        assert!(!provider.can_handle("https://github.com/user/repo"));
    }

    #[test]
    fn test_parse_basic_url() {
        let provider = GitLabProvider::new();

        let parsed = provider.parse_url("https://gitlab.com/user/repo").unwrap();
        assert_eq!(parsed.owner, "user");
        assert_eq!(parsed.repo, "repo");
        assert_eq!(parsed.project_name, "repo@main");
        assert_eq!(parsed.branch_or_commit, None);
        assert!(!parsed.is_commit);
        assert_eq!(parsed.host.as_ref().unwrap(), "gitlab.com");
    }

    #[test]
    fn test_parse_tree_url() {
        let provider = GitLabProvider::new();

        let parsed = provider
            .parse_url("https://gitlab.com/user/repo/-/tree/develop")
            .unwrap();
        assert_eq!(parsed.owner, "user");
        assert_eq!(parsed.repo, "repo");
        assert_eq!(parsed.project_name, "repo@develop");
        assert_eq!(parsed.branch_or_commit, Some("develop".to_string()));
        assert!(!parsed.is_commit);
    }

    #[test]
    fn test_build_download_urls() {
        let provider = GitLabProvider::new();

        let parsed = ParsedRepository::new("user".to_string(), "repo".to_string())
            .with_branch("main".to_string())
            .with_host("gitlab.com".to_string());

        let urls = provider.build_download_urls(&parsed);
        assert!(urls
            .contains(&"https://gitlab.com/user/repo/-/archive/main/repo-main.tar.gz".to_string()));
    }

    #[test]
    fn test_self_hosted_gitlab() {
        let provider = GitLabProvider::new();

        let parsed = provider
            .parse_url("https://gitlab.company.com/team/project")
            .unwrap();
        assert_eq!(parsed.owner, "team");
        assert_eq!(parsed.repo, "project");
        assert_eq!(parsed.host.as_ref().unwrap(), "gitlab.company.com");
    }
}
