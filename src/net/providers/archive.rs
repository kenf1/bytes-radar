use crate::net::traits::{GitProvider, ParsedRepository, ProviderConfig};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

pub struct ArchiveProvider {
    credentials: HashMap<String, String>,
}

impl ArchiveProvider {
    pub fn new() -> Self {
        Self {
            credentials: HashMap::new(),
        }
    }
}

#[async_trait]
impl GitProvider for ArchiveProvider {
    fn name(&self) -> &'static str {
        "archive"
    }

    fn can_handle(&self, url: &str) -> bool {
        url.ends_with(".tar.gz")
            || url.ends_with(".tgz")
            || url.ends_with(".tar.bz2")
            || url.ends_with(".tar.xz")
            || url.ends_with(".zip")
            || url.contains("/archive/")
            || url.contains("/tarball/")
            || url.contains("/zipball/")
    }

    fn parse_url(&self, url: &str) -> Option<ParsedRepository> {
        if !self.can_handle(url) {
            return None;
        }

        let filename = url.split('/').last()?;
        let name = self.extract_name_from_filename(filename);

        Some(
            ParsedRepository::new("archive".to_string(), name.clone())
                .with_host(self.extract_host_from_url(url)),
        )
    }

    fn build_download_urls(&self, _parsed: &ParsedRepository) -> Vec<String> {
        vec![]
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
        if let Some(filename) = url.split('/').last() {
            return self.extract_name_from_filename(filename);
        }

        "archive-project".to_string()
    }
}

impl ArchiveProvider {
    fn extract_name_from_filename(&self, filename: &str) -> String {
        let name = if filename.ends_with(".tar.gz") {
            filename.trim_end_matches(".tar.gz")
        } else if filename.ends_with(".tgz") {
            filename.trim_end_matches(".tgz")
        } else if filename.ends_with(".tar.bz2") {
            filename.trim_end_matches(".tar.bz2")
        } else if filename.ends_with(".tar.xz") {
            filename.trim_end_matches(".tar.xz")
        } else if filename.ends_with(".zip") {
            filename.trim_end_matches(".zip")
        } else {
            filename
        };

        name.to_string()
    }

    fn extract_host_from_url(&self, url: &str) -> String {
        if let Some(start) = url.find("://") {
            let after_protocol = &url[start + 3..];
            if let Some(end) = after_protocol.find('/') {
                return after_protocol[..end].to_string();
            }
            return after_protocol.to_string();
        }
        "unknown".to_string()
    }
}

impl Default for ArchiveProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let provider = ArchiveProvider::new();
        assert!(provider.can_handle("https://example.com/project.tar.gz"));
        assert!(provider.can_handle("https://example.com/project.tgz"));
        assert!(provider.can_handle("https://example.com/project.tar.bz2"));
        assert!(provider.can_handle("https://example.com/project.tar.xz"));
        assert!(provider.can_handle("https://example.com/project.zip"));
        assert!(provider.can_handle("https://example.com/archive/main.tar.gz"));
        assert!(provider.can_handle("https://example.com/tarball/main"));
        assert!(provider.can_handle("https://example.com/zipball/main"));
        assert!(!provider.can_handle("https://github.com/user/repo"));
    }

    #[test]
    fn test_parse_tar_gz_url() {
        let provider = ArchiveProvider::new();

        let parsed = provider
            .parse_url("https://example.com/myproject.tar.gz")
            .unwrap();
        assert_eq!(parsed.owner, "archive");
        assert_eq!(parsed.repo, "myproject");
        assert_eq!(parsed.project_name, "myproject@main");
        assert_eq!(parsed.branch_or_commit, None);
        assert!(!parsed.is_commit);
        assert_eq!(parsed.host.as_ref().unwrap(), "example.com");
    }

    #[test]
    fn test_parse_tgz_url() {
        let provider = ArchiveProvider::new();

        let parsed = provider
            .parse_url("https://cdn.example.com/releases/v1.0.0.tgz")
            .unwrap();
        assert_eq!(parsed.owner, "archive");
        assert_eq!(parsed.repo, "v1.0.0");
        assert_eq!(parsed.project_name, "v1.0.0@main");
        assert_eq!(parsed.host.as_ref().unwrap(), "cdn.example.com");
    }

    #[test]
    fn test_parse_zip_url() {
        let provider = ArchiveProvider::new();

        let parsed = provider
            .parse_url("https://releases.example.com/project-v2.0.zip")
            .unwrap();
        assert_eq!(parsed.owner, "archive");
        assert_eq!(parsed.repo, "project-v2.0");
        assert_eq!(parsed.project_name, "project-v2.0@main");
        assert_eq!(parsed.host.as_ref().unwrap(), "releases.example.com");
    }

    #[test]
    fn test_extract_name_from_filename() {
        let provider = ArchiveProvider::new();

        assert_eq!(
            provider.extract_name_from_filename("project.tar.gz"),
            "project"
        );
        assert_eq!(
            provider.extract_name_from_filename("mylib-v1.0.0.tgz"),
            "mylib-v1.0.0"
        );
        assert_eq!(
            provider.extract_name_from_filename("source.tar.bz2"),
            "source"
        );
        assert_eq!(
            provider.extract_name_from_filename("archive.tar.xz"),
            "archive"
        );
        assert_eq!(
            provider.extract_name_from_filename("release.zip"),
            "release"
        );
    }

    #[test]
    fn test_extract_host_from_url() {
        let provider = ArchiveProvider::new();

        assert_eq!(
            provider.extract_host_from_url("https://example.com/file.tar.gz"),
            "example.com"
        );
        assert_eq!(
            provider.extract_host_from_url("http://cdn.example.org/releases/v1.0.tgz"),
            "cdn.example.org"
        );
        assert_eq!(
            provider.extract_host_from_url("https://api.github.com/repos/user/repo/tarball/main"),
            "api.github.com"
        );
    }

    #[test]
    fn test_get_project_name() {
        let provider = ArchiveProvider::new();

        assert_eq!(
            provider.get_project_name("https://example.com/myproject.tar.gz"),
            "myproject"
        );
        assert_eq!(
            provider.get_project_name("https://releases.example.com/v1.2.3.tgz"),
            "v1.2.3"
        );
    }
}
