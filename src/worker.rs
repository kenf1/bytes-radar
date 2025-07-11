use crate::core::error::AnalysisError;
use crate::net::ProviderConfig;
use crate::{core::filter::IntelligentFilter, net::RemoteAnalyzer};
use instant::Instant;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn console(message: &str) {
    web_sys::console::log_1(&message.into());
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AnalysisOptions {
    pub timeout: Option<u64>,
    pub max_redirects: Option<u32>,
    pub user_agent: Option<String>,
    pub accept_invalid_certs: bool,
    pub headers: HashMap<String, String>,
    pub credentials: HashMap<String, String>,
    pub provider_settings: HashMap<String, String>,
    pub max_file_size: Option<u64>,
    pub use_compression: bool,
    pub proxy: Option<String>,
    pub ignore_hidden: bool,
    pub aggressive_filtering: Option<bool>,
    pub custom_filter: Option<IntelligentFilter>,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            timeout: Some(300),
            max_redirects: Some(10),
            user_agent: Some("bytes-radar/1.0.0".to_string()),
            accept_invalid_certs: false,
            headers: HashMap::new(),
            credentials: HashMap::new(),
            provider_settings: HashMap::new(),
            max_file_size: Some(100 * 1024 * 1024),
            use_compression: true,
            proxy: None,
            ignore_hidden: true,
            aggressive_filtering: None,
            custom_filter: None,
        }
    }
}

#[derive(serde::Serialize, Clone)]
struct WASMDebugInfo {
    total_languages: usize,
    total_files: usize,
    version: String,
    spend_time: f64,
}

#[derive(serde::Serialize)]
struct WASMAnalysisResult {
    project_name: String,
    summary: crate::core::analysis::ProjectSummary,
    language_statistics: Vec<crate::core::analysis::LanguageStatistics>,
    debug_info: WASMDebugInfo,
}

#[derive(serde::Serialize)]
struct WASMErrorResult {
    error: String,
    error_type: String,
    url: String,
    debug_info: WASMDebugInfo,
}

impl AnalysisOptions {
    fn to_provider_config(&self) -> ProviderConfig {
        let mut config = ProviderConfig::new()
            .with_timeout(self.timeout.unwrap_or(300))
            .with_user_agent(
                self.user_agent
                    .clone()
                    .unwrap_or_else(|| "bytes-radar/1.0.0".to_string()),
            )
            .with_accept_invalid_certs(self.accept_invalid_certs);

        if let Some(max_redirects) = self.max_redirects {
            config = config.with_provider_setting("max_redirects", max_redirects.to_string());
        }

        if let Some(max_file_size) = self.max_file_size {
            config = config.with_max_file_size(max_file_size);
        }

        if let Some(proxy) = &self.proxy {
            config = config.with_proxy(proxy);
        }

        for (key, value) in &self.headers {
            config = config.with_header(key, value);
        }

        for (key, value) in &self.credentials {
            config = config.with_credential(key, value);
        }

        for (key, value) in &self.provider_settings {
            config = config.with_provider_setting(key, value);
        }

        config
    }

    fn to_intelligent_filter(&self) -> IntelligentFilter {
        IntelligentFilter {
            max_file_size: self.max_file_size.unwrap_or(100 * 1024 * 1024),
            ignore_hidden: self.ignore_hidden,
            ..IntelligentFilter::default()
        }
    }
}

fn create_wasm_result(
    analysis: &crate::core::analysis::ProjectAnalysis,
    spend_time: f64,
) -> WASMAnalysisResult {
    WASMAnalysisResult {
        project_name: analysis.project_name.clone(),
        summary: analysis.get_summary(),
        language_statistics: analysis.get_language_statistics(),
        debug_info: WASMDebugInfo {
            total_languages: analysis.language_analyses.len(),
            total_files: analysis.global_metrics.file_count,
            version: VERSION.to_string(),
            spend_time,
        },
    }
}

fn create_error_result(error: AnalysisError, url: String, spend_time: f64) -> WASMErrorResult {
    let error_type = match error {
        AnalysisError::NetworkError { .. } => "network_error",
        _ => "analysis_error",
    };

    WASMErrorResult {
        error: format!("{}", error),
        error_type: error_type.to_string(),
        url,
        debug_info: WASMDebugInfo {
            total_languages: 0,
            total_files: 0,
            version: VERSION.to_string(),
            spend_time,
        },
    }
}

#[wasm_bindgen]
pub async fn analyze_url(url: String, options: JsValue) -> Result<JsValue, JsValue> {
    let opts: AnalysisOptions = serde_wasm_bindgen::from_value(options)?;
    console(&format!("Starting analysis for URL: {}", url));

    let mut analyzer = RemoteAnalyzer::new();
    analyzer.set_global_config(opts.to_provider_config());

    match &opts.custom_filter {
        Some(filter) => analyzer.set_filter(filter.clone()),
        None => match opts.aggressive_filtering {
            Some(aggressive) => analyzer.set_aggressive_filtering(aggressive),
            None => analyzer.set_filter(opts.to_intelligent_filter()),
        },
    }

    let start_time = Instant::now();
    let result = match analyzer.analyze_url(&url).await {
        Ok(analysis) => {
            let result = create_wasm_result(&analysis, start_time.elapsed().as_secs_f64());
            console(&format!(
                "Analysis completed successfully for project: {} ({} files, {} languages)",
                analysis.project_name,
                analysis.global_metrics.file_count,
                analysis.language_analyses.len()
            ));
            serde_wasm_bindgen::to_value(&result)?
        }
        Err(e) => {
            console(&format!("Analysis failed: {}", e));
            console(&format!("Error details - URL: {}, Error: {:?}", url, e));
            let error_result = create_error_result(e, url, start_time.elapsed().as_secs_f64());
            serde_wasm_bindgen::to_value(&error_result)?
        }
    };

    Ok(result)
}
