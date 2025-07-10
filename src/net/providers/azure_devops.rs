use crate::net::traits::{GitProvider, ParsedRepository, ProviderConfig};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

pub struct AzureDevOpsProvider {
    credentials: HashMap<String, String>,
}

impl AzureDevOpsProvider {
    pub fn new() -> Self {
        Self {
            credentials: HashMap::new(),
        }
    }
}

#[async_trait]
impl GitProvider for AzureDevOpsProvider {
    fn name(&self) -> &'static str {
        "azure_devops"
    }

    fn can_handle(&self, url: &str) -> bool {
        url.contains("dev.azure.com") || url.contains("visualstudio.com") || url.contains("_git/")
    }

    fn parse_url(&self, url: &str) -> Option<ParsedRepository> {
        if !self.can_handle(url) {
            return None;
        }

        let url = url.trim_end_matches('/');

        if url.contains("?version=GB") {
            return self.parse_branch_url(url);
        }

        if url.contains("?version=GC") {
            return self.parse_commit_url(url);
        }

        self.parse_basic_url(url)
    }

    fn build_download_urls(&self, parsed: &ParsedRepository) -> Vec<String> {
        let mut urls = Vec::new();

        if let Some(ref branch_or_commit) = parsed.branch_or_commit {
            let host = parsed.host.as_deref().unwrap_or("dev.azure.com");

            if parsed.is_commit {
                urls.push(format!(
                    "https://{}/{}/{}/_apis/git/repositories/{}/items?path=/&versionDescriptor.version={}&$format=zip",
                    host, parsed.owner, parsed.repo.split('/').next().unwrap_or(&parsed.repo), parsed.repo, branch_or_commit
                ));
            } else {
                urls.push(format!(
                    "https://{}/{}/{}/_apis/git/repositories/{}/items?path=/&versionDescriptor.versionType=branch&versionDescriptor.version={}&$format=zip",
                    host, parsed.owner, parsed.repo.split('/').next().unwrap_or(&parsed.repo), parsed.repo, branch_or_commit
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
        self.credentials = config.credentials.clone();
    }

    fn get_project_name(&self, url: &str) -> String {
        if let Some(parsed) = self.parse_url(url) {
            return parsed.project_name;
        }

        if let Some(filename) = url.split('/').last() {
            if filename.ends_with(".zip") {
                return filename.trim_end_matches(".zip").to_string();
            }
            return filename.to_string();
        }

        "azure-devops-project".to_string()
    }
}

impl AzureDevOpsProvider {
    fn parse_commit_url(&self, url: &str) -> Option<ParsedRepository> {
        if let Some(commit_start) = url.find("?version=GC") {
            let base_url = &url[..commit_start];
            let commit = &url[commit_start + "?version=GC".len()..];

            if let Some(parsed_base) = self.parse_basic_url(base_url) {
                return Some(parsed_base.with_commit(commit.to_string()));
            }
        }
        None
    }

    fn parse_branch_url(&self, url: &str) -> Option<ParsedRepository> {
        if let Some(branch_start) = url.find("?version=GB") {
            let base_url = &url[..branch_start];
            let branch = &url[branch_start + "?version=GB".len()..];

            if let Some(parsed_base) = self.parse_basic_url(base_url) {
                return Some(parsed_base.with_branch(branch.to_string()));
            }
        }
        None
    }

    fn parse_basic_url(&self, url: &str) -> Option<ParsedRepository> {
        if url.contains("dev.azure.com") {
            let parts: Vec<&str> = url.split('/').collect();
            if parts.len() >= 7 && parts.contains(&"_git") {
                let host = parts[2].to_string();
                let org = parts[3].to_string();
                let project = parts[4].to_string();
                let repo = parts[6].to_string();

                return Some(
                    ParsedRepository::new(format!("{}/{}", org, project), repo).with_host(host),
                );
            }
        } else if url.contains("visualstudio.com") {
            let parts: Vec<&str> = url.split('/').collect();
            if parts.len() >= 6 && parts.contains(&"_git") {
                let host = parts[2].to_string();
                let org = parts[2].split('.').next().unwrap_or("").to_string();
                let project = parts[4].to_string();
                let repo = parts[6].to_string();

                return Some(
                    ParsedRepository::new(format!("{}/{}", org, project), repo).with_host(host),
                );
            }
        }
        None
    }
}

impl Default for AzureDevOpsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let provider = AzureDevOpsProvider::new();
        assert!(provider.can_handle("https://dev.azure.com/org/project/_git/repo"));
        assert!(provider.can_handle("https://org.visualstudio.com/project/_git/repo"));
        assert!(!provider.can_handle("https://github.com/user/repo"));
    }

    #[test]
    fn test_parse_basic_url() {
        let provider = AzureDevOpsProvider::new();

        let parsed = provider
            .parse_url("https://dev.azure.com/myorg/myproject/_git/myrepo")
            .unwrap();
        assert_eq!(parsed.owner, "myorg/myproject");
        assert_eq!(parsed.repo, "myrepo");
        assert_eq!(parsed.project_name, "myrepo@main");
        assert_eq!(parsed.branch_or_commit, None);
        assert!(!parsed.is_commit);
        assert_eq!(parsed.host.as_ref().unwrap(), "dev.azure.com");
    }

    #[test]
    fn test_parse_branch_url() {
        let provider = AzureDevOpsProvider::new();

        let parsed = provider
            .parse_url("https://dev.azure.com/myorg/myproject/_git/myrepo?version=GBdevelop")
            .unwrap();
        assert_eq!(parsed.owner, "myorg/myproject");
        assert_eq!(parsed.repo, "myrepo");
        assert_eq!(parsed.project_name, "myrepo@develop");
        assert_eq!(parsed.branch_or_commit, Some("develop".to_string()));
        assert!(!parsed.is_commit);
    }
}
