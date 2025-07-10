pub mod providers;
pub mod stream;
pub mod traits;

use crate::core::{analysis::ProjectAnalysis, error::Result, filter::IntelligentFilter};
use providers::*;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use traits::{GitProvider, NoOpProgressHook};

pub use traits::{ParsedRepository, ProgressHook, ProviderConfig};

/// Remote repository analyzer with comprehensive configuration support
///
/// The RemoteAnalyzer supports multiple Git hosting providers and allows
/// extensive customization of HTTP requests, authentication, and processing behavior.
///
/// # Examples
///
/// ```rust
/// use bytes_radar::net::{RemoteAnalyzer, ProviderConfig};
///
/// // Basic usage
/// let mut analyzer = RemoteAnalyzer::new();
///
/// // With custom configuration
/// let config = ProviderConfig::new()
///     .with_timeout(120)
///     .with_header("X-Custom-Header", "value")
///     .with_credential("token", "your-token");
///
/// analyzer.set_global_config(config);
/// ```
pub struct RemoteAnalyzer {
    providers: Vec<Box<dyn GitProvider>>,
    global_config: ProviderConfig,
    filter: IntelligentFilter,
    progress_hook: Arc<dyn ProgressHook>,
    provider_configs: HashMap<String, ProviderConfig>,
}

impl RemoteAnalyzer {
    /// Create a new analyzer with default configuration
    pub fn new() -> Self {
        let mut analyzer = Self {
            providers: Vec::new(),
            global_config: ProviderConfig::default(),
            filter: IntelligentFilter::default(),
            progress_hook: Arc::new(NoOpProgressHook),
            provider_configs: HashMap::new(),
        };

        analyzer.register_default_providers();
        analyzer
    }

    /// Register all default Git providers
    fn register_default_providers(&mut self) {
        self.providers.push(Box::new(GitHubProvider::new()));
        self.providers.push(Box::new(GitLabProvider::new()));
        self.providers.push(Box::new(BitbucketProvider::new()));
        self.providers.push(Box::new(CodebergProvider::new()));
        self.providers.push(Box::new(GiteaProvider::new()));
        self.providers.push(Box::new(SourceForgeProvider::new()));
        self.providers.push(Box::new(AzureDevOpsProvider::new()));
        self.providers.push(Box::new(ArchiveProvider::new()));
    }

    /// Set a progress hook for monitoring operations
    ///
    /// # Arguments
    /// * `hook` - Progress hook implementation
    ///
    /// # Examples
    /// ```rust
    /// use bytes_radar::net::{RemoteAnalyzer, ProgressHook};
    ///
    /// struct MyHook;
    /// impl ProgressHook for MyHook {
    ///     fn on_download_progress(&self, downloaded: u64, total: Option<u64>) {
    ///         println!("Downloaded: {} bytes", downloaded);
    ///     }
    ///     fn on_processing_start(&self, message: &str) {
    ///         println!("Processing: {}", message);
    ///     }
    ///     fn on_processing_progress(&self, current: usize, total: usize) {
    ///         println!("Progress: {}/{}", current, total);
    ///     }
    /// }
    ///
    /// let mut analyzer = RemoteAnalyzer::new();
    /// analyzer.set_progress_hook(MyHook);
    /// ```
    pub fn set_progress_hook<H: ProgressHook + 'static>(&mut self, hook: H) {
        self.progress_hook = Arc::new(hook);
    }

    /// Set global configuration that applies to all providers
    ///
    /// # Arguments
    /// * `config` - Global configuration
    ///
    /// # Examples
    /// ```rust
    /// use bytes_radar::net::{RemoteAnalyzer, ProviderConfig};
    ///
    /// let config = ProviderConfig::new()
    ///     .with_timeout(300)
    ///     .with_user_agent("my-app/1.0.0")
    ///     .with_header("X-API-Key", "secret");
    ///
    /// let mut analyzer = RemoteAnalyzer::new();
    /// analyzer.set_global_config(config);
    /// ```
    pub fn set_global_config(&mut self, config: ProviderConfig) {
        self.global_config = config;
        self.apply_config_to_providers();
    }

    /// Set configuration for a specific provider
    ///
    /// # Arguments
    /// * `provider_name` - Name of the provider (e.g., "github", "gitlab")
    /// * `config` - Provider-specific configuration
    ///
    /// # Examples
    /// ```rust
    /// use bytes_radar::net::{RemoteAnalyzer, ProviderConfig};
    ///
    /// let github_config = ProviderConfig::new()
    ///     .with_credential("token", "github-token")
    ///     .with_header("Accept", "application/vnd.github.v3+json");
    ///
    /// let mut analyzer = RemoteAnalyzer::new();
    /// analyzer.set_provider_config("github", github_config);
    /// ```
    pub fn set_provider_config(&mut self, provider_name: &str, config: ProviderConfig) {
        self.provider_configs
            .insert(provider_name.to_string(), config);
        self.apply_config_to_providers();
    }

    /// Apply configurations to all providers
    fn apply_config_to_providers(&mut self) {
        for provider in &mut self.providers {
            let provider_name = provider.name();

            // Start with global config
            let mut config = self.global_config.clone();

            // Override with provider-specific config if exists
            if let Some(provider_config) = self.provider_configs.get(provider_name) {
                // Merge configurations (provider-specific takes precedence)
                config.headers.extend(provider_config.headers.clone());
                config
                    .credentials
                    .extend(provider_config.credentials.clone());
                config
                    .provider_settings
                    .extend(provider_config.provider_settings.clone());

                if provider_config.timeout.is_some() {
                    config.timeout = provider_config.timeout;
                }
                if provider_config.max_redirects.is_some() {
                    config.max_redirects = provider_config.max_redirects;
                }
                if provider_config.user_agent.is_some() {
                    config.user_agent = provider_config.user_agent.clone();
                }
                if provider_config.max_file_size.is_some() {
                    config.max_file_size = provider_config.max_file_size;
                }
                if provider_config.proxy.is_some() {
                    config.proxy = provider_config.proxy.clone();
                }

                config.accept_invalid_certs = provider_config.accept_invalid_certs;
                config.use_compression = provider_config.use_compression;
            }

            provider.apply_config(&config);
        }
    }

    /// Set file filtering configuration
    ///
    /// # Arguments
    /// * `filter` - File filter configuration
    pub fn set_filter(&mut self, filter: IntelligentFilter) {
        self.filter = filter;
    }

    /// Enable or disable aggressive file filtering
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable aggressive filtering
    pub fn set_aggressive_filtering(&mut self, enabled: bool) {
        if enabled {
            self.filter = IntelligentFilter::aggressive();
        } else {
            self.filter = IntelligentFilter::default();
        }
    }

    // Legacy methods for backward compatibility

    /// Set timeout for all providers (legacy method)
    ///
    /// # Arguments
    /// * `timeout` - Timeout in seconds
    pub fn set_timeout(&mut self, timeout: u64) {
        self.global_config.timeout = Some(timeout);
        self.apply_config_to_providers();
    }

    /// Set whether to accept invalid SSL certificates (legacy method)
    ///
    /// # Arguments
    /// * `allow_insecure` - Whether to accept invalid certificates
    pub fn set_allow_insecure(&mut self, allow_insecure: bool) {
        self.global_config.accept_invalid_certs = allow_insecure;
        self.apply_config_to_providers();
    }

    /// Set credentials for a specific provider (legacy method)
    ///
    /// # Arguments
    /// * `provider_name` - Name of the provider
    /// * `credentials` - Credentials map
    pub fn set_provider_credentials(
        &mut self,
        provider_name: &str,
        credentials: HashMap<String, String>,
    ) {
        let config = self
            .provider_configs
            .entry(provider_name.to_string())
            .or_insert_with(ProviderConfig::default);

        config.credentials.extend(credentials);
        self.apply_config_to_providers();
    }

    /// Analyze a repository from its URL
    ///
    /// # Arguments
    /// * `url` - Repository URL or shorthand notation
    ///
    /// # Examples
    /// ```rust,no_run
    /// use bytes_radar::net::RemoteAnalyzer;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let analyzer = RemoteAnalyzer::new();
    ///
    ///     // Full URLs
    ///     let analysis = analyzer.analyze_url("https://github.com/user/repo").await?;
    ///
    ///     // Shorthand notation
    ///     let analysis = analyzer.analyze_url("user/repo@main").await?;
    ///
    ///     // Direct archive
    ///     let analysis = analyzer.analyze_url("https://example.com/project.tar.gz").await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn analyze_url(&self, url: &str) -> Result<ProjectAnalysis> {
        let expanded_url = self.expand_url(url);

        // Try direct archive first for better performance
        if expanded_url.ends_with(".tar.gz") || expanded_url.ends_with(".tgz") {
            return self.analyze_direct_tarball(&expanded_url).await;
        }

        // Try each provider
        for provider in &self.providers {
            if provider.can_handle(&expanded_url) {
                if let Some(parsed) = provider.parse_url(&expanded_url) {
                    match self.analyze_with_provider(provider.as_ref(), &parsed).await {
                        Ok(analysis) => return Ok(analysis),
                        Err(e) => {
                            #[cfg(feature = "cli")]
                            log::debug!(
                                "Provider {} failed for {}: {}",
                                provider.name(),
                                expanded_url,
                                e
                            );
                            continue;
                        }
                    }
                }
            }
        }

        Err(crate::core::error::AnalysisError::url_parsing(format!(
            "Unsupported URL format: {}. Supported formats include GitHub, GitLab, Bitbucket, Codeberg, Gitea, SourceForge, Azure DevOps, and direct archive URLs.",
            expanded_url
        )))
    }

    /// Analyze using a specific provider
    async fn analyze_with_provider(
        &self,
        provider: &dyn GitProvider,
        parsed: &ParsedRepository,
    ) -> Result<ProjectAnalysis> {
        let mut download_urls = provider.build_download_urls(parsed);

        // If no URLs and no specific branch/commit, try common branches
        if download_urls.is_empty() && parsed.branch_or_commit.is_none() {
            let mut branches = vec![
                "main".to_string(),
                "master".to_string(),
                "develop".to_string(),
                "dev".to_string(),
            ];

            // Try to get default branch from API
            #[cfg(not(target_arch = "wasm32"))]
            {
                let config = self.get_effective_config(provider.name());
                if let Ok(client) = provider.build_client(&config) {
                    if let Some(default_branch) = provider.get_default_branch(&client, parsed).await
                    {
                        branches.insert(0, default_branch);
                        branches.dedup();
                    }
                }
            }

            // Generate URLs for each branch
            for branch in branches {
                let mut branch_parsed = parsed.clone();
                branch_parsed.branch_or_commit = Some(branch);
                download_urls.extend(provider.build_download_urls(&branch_parsed));
            }
        }

        // Try each download URL
        for download_url in download_urls {
            match self
                .analyze_direct_tarball_with_name(&download_url, &parsed.project_name)
                .await
            {
                Ok(analysis) => return Ok(analysis),
                Err(e) => {
                    #[cfg(feature = "cli")]
                    log::debug!("Failed to download from {}: {}", download_url, e);
                    continue;
                }
            }
        }

        Err(crate::core::error::AnalysisError::network(
            "All download URLs failed".to_string(),
        ))
    }

    /// Get effective configuration for a provider
    fn get_effective_config(&self, provider_name: &str) -> ProviderConfig {
        let mut config = self.global_config.clone();

        if let Some(provider_config) = self.provider_configs.get(provider_name) {
            // Merge configurations
            config.headers.extend(provider_config.headers.clone());
            config
                .credentials
                .extend(provider_config.credentials.clone());
            config
                .provider_settings
                .extend(provider_config.provider_settings.clone());

            if provider_config.timeout.is_some() {
                config.timeout = provider_config.timeout;
            }
            if provider_config.max_redirects.is_some() {
                config.max_redirects = provider_config.max_redirects;
            }
            if provider_config.user_agent.is_some() {
                config.user_agent = provider_config.user_agent.clone();
            }
            if provider_config.max_file_size.is_some() {
                config.max_file_size = provider_config.max_file_size;
            }
            if provider_config.proxy.is_some() {
                config.proxy = provider_config.proxy.clone();
            }

            config.accept_invalid_certs = provider_config.accept_invalid_certs;
            config.use_compression = provider_config.use_compression;
        }

        config
    }

    /// Analyze a direct archive URL
    async fn analyze_direct_tarball(&self, url: &str) -> Result<ProjectAnalysis> {
        let project_name = self.extract_project_name_from_url(url);
        self.analyze_direct_tarball_with_name(url, &project_name)
            .await
    }

    /// Analyze a direct archive URL with custom project name
    async fn analyze_direct_tarball_with_name(
        &self,
        url: &str,
        project_name: &str,
    ) -> Result<ProjectAnalysis> {
        let mut project_analysis = ProjectAnalysis::new(project_name);

        // Use global config to build client for direct downloads
        let client = self.build_global_client()?;

        let response = client.get(url).send().await.map_err(|e| {
            crate::core::error::AnalysisError::network(format!("Failed to fetch URL: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(crate::core::error::AnalysisError::network(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        let total_size = response.content_length();
        self.progress_hook.on_download_progress(0, total_size);

        let stream = response.bytes_stream();
        let progress_hook = Arc::clone(&self.progress_hook);
        let stream_reader = stream::StreamReader::new(
            stream,
            Box::new(move |downloaded, total| {
                progress_hook.on_download_progress(downloaded, total);
                log::debug!(
                    "Downloaded: {} bytes of {} total",
                    downloaded,
                    total
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                );
            }),
            total_size,
        );

        self.progress_hook.on_processing_start("Processing...");
        stream::process_tarball_stream(
            stream_reader,
            &mut project_analysis,
            &self.filter,
            self.progress_hook.as_ref(),
        )
        .await?;

        Ok(project_analysis)
    }

    /// Build HTTP client using global configuration
    fn build_global_client(&self) -> Result<Client> {
        // Use archive provider to build client (it has good defaults)
        let archive_provider = ArchiveProvider::new();
        archive_provider
            .build_client(&self.global_config)
            .map_err(|e| {
                crate::core::error::AnalysisError::network(format!(
                    "Failed to build HTTP client: {}",
                    e
                ))
            })
    }

    /// Expand shorthand URLs to full URLs
    fn expand_url(&self, url: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") {
            return url.to_string();
        }

        // Handle shorthand notation like "user/repo@branch"
        if url.contains('/') && !url.starts_with("http://") && !url.starts_with("https://") {
            let parts: Vec<&str> = url.split('@').collect();
            let repo_part = parts[0];
            let branch_or_commit = parts.get(1);

            let path_parts: Vec<&str> = repo_part.split('/').collect();
            if path_parts.len() == 2 {
                if let Some(branch) = branch_or_commit {
                    // Check if it looks like a commit hash
                    if branch.len() >= 7 && branch.chars().all(|c| c.is_ascii_hexdigit()) {
                        return format!("https://github.com/{}/commit/{}", repo_part, branch);
                    } else {
                        return format!("https://github.com/{}/tree/{}", repo_part, branch);
                    }
                } else {
                    return format!("https://github.com/{}", repo_part);
                }
            }
        }

        url.to_string()
    }

    /// Extract project name from a direct URL
    fn extract_project_name_from_url(&self, url: &str) -> String {
        let url_path = url.trim_end_matches('/');

        if let Some(filename) = url_path.split('/').next_back() {
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
}

impl Default for RemoteAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
