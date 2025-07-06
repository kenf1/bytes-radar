use crate::core::{filter::IntelligentFilter, net::RemoteAnalyzer};
use wasm_bindgen::prelude::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AnalysisOptions {
    pub ignore_hidden: bool,
    pub ignore_gitignore: bool,
    pub max_file_size: i64,
    pub aggressive_filtering: Option<bool>,
    pub custom_filter: Option<IntelligentFilter>,
}

#[wasm_bindgen]
pub async fn analyze_url(url: String, options: JsValue) -> Result<JsValue, JsValue> {
    let opts: AnalysisOptions = serde_wasm_bindgen::from_value(options)?;

    web_sys::console::log_1(&format!("Starting analysis for URL: {}", url).into());

    let mut analyzer = RemoteAnalyzer::new();

    // Configure intelligent filtering
    if let Some(custom_filter) = opts.custom_filter {
        analyzer.set_filter(custom_filter);
    } else if let Some(aggressive) = opts.aggressive_filtering {
        analyzer.set_aggressive_filtering(aggressive);
    } else {
        // Use default filter but with backwards compatibility
        let mut filter = IntelligentFilter::default();
        filter.max_file_size = opts.max_file_size as u64;
        filter.ignore_hidden = opts.ignore_hidden;
        analyzer.set_filter(filter);
    }

    match analyzer.analyze_url(&url).await {
        Ok(analysis) => {
            web_sys::console::log_1(
                &format!(
                    "Analysis completed successfully for project: {}",
                    analysis.project_name
                )
                .into(),
            );

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
            for (lang_name, lang_analysis) in &analysis.language_analyses {
                languages.push(SimplifiedLanguageInfo {
                    language_name: lang_name.clone(),
                    file_count: lang_analysis.file_metrics.len(),
                    total_lines: lang_analysis.aggregate_metrics.total_lines,
                });
            }

            let debug_result = DebugResult {
                project_name: analysis.project_name.clone(),
                languages,
                global_metrics: analysis.global_metrics.clone(),
                debug_info: format!(
                    "Languages: {}, Files: {}",
                    analysis.language_analyses.len(),
                    analysis.global_metrics.file_count
                ),
            };

            Ok(serde_wasm_bindgen::to_value(&debug_result)?)
        }
        Err(e) => {
            web_sys::console::log_1(&format!("Analysis failed: {}", e).into());
            Err(JsValue::from_str(&format!("Analysis failed: {}", e)))
        }
    }
}
