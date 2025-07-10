use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;

/// Progress hook trait for monitoring download and processing progress
pub trait ProgressHook: Send + Sync {
    /// Called when download progress is updated
    ///
    /// # Arguments
    /// * `downloaded` - Number of bytes downloaded so far
    /// * `total` - Total size in bytes (if known)
    fn on_download_progress(&self, downloaded: u64, total: Option<u64>);

    /// Called when processing starts with a status message
    ///
    /// # Arguments  
    /// * `message` - Status message describing current operation
    fn on_processing_start(&self, message: &str);

    /// Called when processing progress is updated
    ///
    /// # Arguments
    /// * `current` - Current item being processed
    /// * `total` - Total items to process
    fn on_processing_progress(&self, current: usize, total: usize);
}

/// No-operation progress hook that ignores all progress updates
pub struct NoOpProgressHook;

impl ProgressHook for NoOpProgressHook {
    fn on_download_progress(&self, _downloaded: u64, _total: Option<u64>) {}
    fn on_processing_start(&self, _message: &str) {}
    fn on_processing_progress(&self, _current: usize, _total: usize) {}
}

/// Universal configuration for all Git providers
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// Custom HTTP headers to include in requests
    pub headers: HashMap<String, String>,

    /// Request timeout in seconds (None for default)
    pub timeout: Option<u64>,

    /// Maximum number of redirects to follow
    pub max_redirects: Option<u32>,

    /// User agent string to use for requests
    pub user_agent: Option<String>,

    /// Whether to accept invalid SSL certificates
    pub accept_invalid_certs: bool,

    /// Authentication credentials (varies by provider)
    pub credentials: HashMap<String, String>,

    /// Provider-specific settings
    pub provider_settings: HashMap<String, String>,

    /// Maximum file size to download in bytes
    pub max_file_size: Option<u64>,

    /// Whether to use compression for requests
    pub use_compression: bool,

    /// Custom proxy URL
    pub proxy: Option<String>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            headers: HashMap::new(),
            timeout: Some(300), // 5 minutes default
            max_redirects: Some(10),
            user_agent: Some("bytes-radar/1.0.0".to_string()),
            accept_invalid_certs: false,
            credentials: HashMap::new(),
            provider_settings: HashMap::new(),
            max_file_size: Some(100 * 1024 * 1024), // 100MB default
            use_compression: true,
            proxy: None,
        }
    }
}

impl ProviderConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a custom header
    ///
    /// # Arguments
    /// * `name` - Header name
    /// * `value` - Header value
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set request timeout in seconds
    ///
    /// # Arguments
    /// * `timeout` - Timeout in seconds
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set user agent string
    ///
    /// # Arguments
    /// * `user_agent` - User agent string
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set whether to accept invalid SSL certificates
    ///
    /// # Arguments
    /// * `accept` - Whether to accept invalid certificates
    pub fn with_accept_invalid_certs(mut self, accept: bool) -> Self {
        self.accept_invalid_certs = accept;
        self
    }

    /// Set authentication credentials
    ///
    /// # Arguments
    /// * `key` - Credential key (e.g., "token", "username")
    /// * `value` - Credential value
    pub fn with_credential(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.credentials.insert(key.into(), value.into());
        self
    }

    /// Set provider-specific setting
    ///
    /// # Arguments
    /// * `key` - Setting key
    /// * `value` - Setting value
    pub fn with_provider_setting(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.provider_settings.insert(key.into(), value.into());
        self
    }

    /// Set maximum file size in bytes
    ///
    /// # Arguments
    /// * `size` - Maximum file size in bytes
    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = Some(size);
        self
    }

    /// Set proxy URL
    ///
    /// # Arguments
    /// * `proxy` - Proxy URL
    pub fn with_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }
}

/// Parsed repository information from a URL
#[derive(Debug, Clone)]
pub struct ParsedRepository {
    /// Repository owner/organization
    pub owner: String,

    /// Repository name
    pub repo: String,

    /// Branch name or commit hash (if specified)
    pub branch_or_commit: Option<String>,

    /// Whether branch_or_commit is a commit hash
    pub is_commit: bool,

    /// Generated project name for display
    pub project_name: String,

    /// Host name (e.g., "github.com")
    pub host: Option<String>,
}

impl ParsedRepository {
    /// Create a new parsed repository with default main branch
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    pub fn new(owner: String, repo: String) -> Self {
        let project_name = format!("{}@main", repo);
        Self {
            owner,
            repo,
            branch_or_commit: None,
            is_commit: false,
            project_name,
            host: None,
        }
    }

    /// Set the branch and update project name
    ///
    /// # Arguments
    /// * `branch` - Branch name
    pub fn with_branch(mut self, branch: String) -> Self {
        self.project_name = format!("{}@{}", self.repo, branch);
        self.branch_or_commit = Some(branch);
        self.is_commit = false;
        self
    }

    /// Set the commit hash and update project name
    ///
    /// # Arguments
    /// * `commit` - Commit hash
    pub fn with_commit(mut self, commit: String) -> Self {
        let short_commit = &commit[..7.min(commit.len())];
        self.project_name = format!("{}@{}", self.repo, short_commit);
        self.branch_or_commit = Some(commit);
        self.is_commit = true;
        self
    }

    /// Set the host name
    ///
    /// # Arguments
    /// * `host` - Host name
    pub fn with_host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }
}

/// Git provider trait for handling different repository hosting services
#[async_trait]
pub trait GitProvider: Send + Sync {
    /// Get the provider name (e.g., "github", "gitlab")
    fn name(&self) -> &'static str;

    /// Check if this provider can handle the given URL
    ///
    /// # Arguments
    /// * `url` - URL to check
    fn can_handle(&self, url: &str) -> bool;

    /// Parse a URL into repository information
    ///
    /// # Arguments
    /// * `url` - URL to parse
    fn parse_url(&self, url: &str) -> Option<ParsedRepository>;

    /// Build download URLs for the parsed repository
    ///
    /// # Arguments
    /// * `parsed` - Parsed repository information
    fn build_download_urls(&self, parsed: &ParsedRepository) -> Vec<String>;

    /// Get the default branch for a repository (if supported)
    ///
    /// # Arguments
    /// * `client` - HTTP client to use
    /// * `parsed` - Parsed repository information
    async fn get_default_branch(
        &self,
        client: &Client,
        parsed: &ParsedRepository,
    ) -> Option<String>;

    /// Apply configuration to this provider
    ///
    /// # Arguments
    /// * `config` - Configuration to apply
    fn apply_config(&mut self, config: &ProviderConfig);

    /// Get project name from URL
    ///
    /// # Arguments
    /// * `url` - URL to extract project name from
    fn get_project_name(&self, url: &str) -> String;

    /// Build HTTP client with provider-specific configuration
    ///
    /// # Arguments
    /// * `config` - Configuration to use
    fn build_client(
        &self,
        config: &ProviderConfig,
    ) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
        let mut builder = Client::builder();

        // Set user agent
        if let Some(ref user_agent) = config.user_agent {
            builder = builder.user_agent(user_agent);
        }

        // Set timeout (works on both wasm and native)
        if let Some(timeout) = config.timeout {
            #[cfg(not(target_arch = "wasm32"))]
            {
                builder = builder.timeout(Duration::from_secs(timeout));
            }
            #[cfg(target_arch = "wasm32")]
            {
                // On WASM, timeout is handled differently
                builder = builder.timeout(Duration::from_secs(timeout.min(300)));
                // Max 5 minutes for WASM
            }
        }

        // Set redirects
        if let Some(max_redirects) = config.max_redirects {
            builder = builder.redirect(reqwest::redirect::Policy::limited(max_redirects as usize));
        }

        // Set SSL verification
        #[cfg(not(target_arch = "wasm32"))]
        if config.accept_invalid_certs {
            builder = builder.danger_accept_invalid_certs(true);
        }

        // Set compression
        if !config.use_compression {
            builder = builder.no_gzip();
            #[cfg(not(target_arch = "wasm32"))]
            {
                builder = builder.no_brotli();
                builder = builder.no_deflate();
            }
        }

        // Set proxy (only on native)
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(ref proxy) = config.proxy {
            let proxy = reqwest::Proxy::all(proxy)?;
            builder = builder.proxy(proxy);
        }

        // Build default headers
        let mut headers = reqwest::header::HeaderMap::new();

        // Add custom headers
        for (name, value) in &config.headers {
            let header_name = reqwest::header::HeaderName::from_bytes(name.as_bytes())?;
            let header_value = reqwest::header::HeaderValue::from_str(value)?;
            headers.insert(header_name, header_value);
        }

        // Add provider-specific auth headers
        self.add_auth_headers(&mut headers, config)?;

        if !headers.is_empty() {
            builder = builder.default_headers(headers);
        }

        Ok(builder.build()?)
    }

    /// Add authentication headers specific to this provider
    ///
    /// # Arguments
    /// * `headers` - Header map to add to
    /// * `config` - Configuration containing credentials
    fn add_auth_headers(
        &self,
        _headers: &mut reqwest::header::HeaderMap,
        _config: &ProviderConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Default implementation does nothing
        // Providers can override this
        Ok(())
    }

    /// Validate configuration for this provider
    ///
    /// # Arguments
    /// * `config` - Configuration to validate
    fn validate_config(&self, config: &ProviderConfig) -> Result<(), String> {
        // Basic validation
        if let Some(timeout) = config.timeout {
            if timeout == 0 {
                return Err("Timeout cannot be zero".to_string());
            }
            if timeout > 3600 {
                return Err("Timeout cannot exceed 1 hour".to_string());
            }
        }

        if let Some(max_file_size) = config.max_file_size {
            if max_file_size > 1024 * 1024 * 1024 {
                return Err("Max file size cannot exceed 1GB".to_string());
            }
        }

        Ok(())
    }
}
