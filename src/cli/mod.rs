#[cfg(feature = "cli")]
mod args;
#[cfg(feature = "cli")]
mod output;
#[cfg(feature = "cli")]
mod progress;
#[cfg(feature = "cli")]
mod url_parser;

#[cfg(feature = "cli")]
use crate::core::*;
#[cfg(feature = "cli")]
use crate::net::RemoteAnalyzer;
#[cfg(feature = "cli")]
use clap::Parser;
#[cfg(feature = "cli")]
use std::time::Instant;

#[cfg(feature = "cli")]
pub use args::{Cli, OutputFormat};

#[cfg(feature = "cli")]
pub async fn run() -> Result<()> {
    let cli = args::Cli::parse();

    init_logging(&cli)?;

    let mut cli = cli;
    if let Ok(token) = std::env::var("BRADAR_TOKEN") {
        if cli.token.is_none() {
            cli.token = Some(token);
        }
    }

    match &cli.url {
        Some(url) => analyze_remote_archive(url, &cli).await,
        None => {
            url_parser::show_usage_examples();
            std::process::exit(1);
        }
    }
}

#[cfg(feature = "cli")]
fn init_logging(cli: &Cli) -> Result<()> {
    let log_level = if cli.trace {
        "trace"
    } else if cli.debug {
        "debug"
    } else {
        "warn"
    };

    let mut builder =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level));

    if let Some(log_file) = &cli.log_file {
        use std::fs::OpenOptions;

        let target = Box::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)
                .map_err(|e| crate::core::error::AnalysisError::file_read(log_file, e))?,
        );
        builder.target(env_logger::Target::Pipe(target));
    }

    builder.init();
    Ok(())
}

#[cfg(feature = "cli")]
async fn analyze_remote_archive(url: &str, cli: &Cli) -> Result<()> {
    let should_show_progress =
        !cli.no_progress && matches!(cli.format, OutputFormat::Table) && !cli.quiet;

    let processed_url = url_parser::expand_url(url);

    if should_show_progress && !cli.quiet {
        println!("Analyzing: {}", processed_url);
    }

    let start_time = Instant::now();
    let progress_bar = progress::create_progress_bar(should_show_progress);

    let mut analyzer = RemoteAnalyzer::new();

    if let Some(token) = &cli.token {
        let mut credentials = std::collections::HashMap::new();
        credentials.insert("token".to_string(), token.clone());
        analyzer.set_provider_credentials("github", credentials);
    }

    analyzer.set_timeout(cli.timeout);
    analyzer.set_allow_insecure(cli.allow_insecure);

    if let Some(pb) = progress_bar.clone() {
        analyzer.set_progress_hook(progress::ProgressBarHook::new(pb));
    }

    configure_analyzer_filters(&mut analyzer, cli)?;

    let project_analysis = analyzer.analyze_url(&processed_url).await?;

    let elapsed = start_time.elapsed();

    if let Some(pb) = &progress_bar {
        pb.finish_and_clear();
    }

    progress::show_completion_message(elapsed, cli.quiet);

    output_results(&project_analysis, cli)?;

    Ok(())
}

#[cfg(feature = "cli")]
fn configure_analyzer_filters(analyzer: &mut RemoteAnalyzer, cli: &Cli) -> Result<()> {
    if cli.aggressive_filter {
        analyzer.set_aggressive_filtering(true);
    } else {
        let filter = filter::IntelligentFilter {
            max_file_size: cli.max_file_size * 1024,
            ignore_test_dirs: !cli.include_tests,
            ignore_docs_dirs: !cli.include_docs,
            ..filter::IntelligentFilter::default()
        };

        analyzer.set_filter(filter);
    }

    Ok(())
}

#[cfg(feature = "cli")]
fn output_results(project_analysis: &analysis::ProjectAnalysis, cli: &Cli) -> Result<()> {
    match cli.format {
        OutputFormat::Table => {
            output::print_table_format(project_analysis, cli.detailed, cli.quiet);
        }
        OutputFormat::Json => output::print_json_format(project_analysis)?,
        OutputFormat::Csv => output::print_csv_format(project_analysis)?,
        OutputFormat::Xml => output::print_xml_format(project_analysis)?,
        OutputFormat::Yaml => output::print_yaml_format(project_analysis)?,
        OutputFormat::Toml => output::print_toml_format(project_analysis)?,
    }

    Ok(())
}
