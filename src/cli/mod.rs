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
use clap::Parser;
#[cfg(feature = "cli")]
use std::time::Instant;

#[cfg(feature = "cli")]
pub use args::{Cli, OutputFormat};

#[cfg(feature = "cli")]
pub async fn run() -> Result<()> {
    let cli = args::Cli::parse();

    if cli.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
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
async fn analyze_remote_archive(url: &str, cli: &Cli) -> Result<()> {
    let format = cli.format.as_ref().unwrap_or(&OutputFormat::Table);
    let should_show_progress =
        !cli.no_progress && matches!(format, OutputFormat::Table) && !cli.quiet;

    let processed_url = url_parser::expand_url(url);

    if should_show_progress && !cli.quiet {
        println!("Analyzing: {}", processed_url);
        println!("{}", "─".repeat(80));
    }

    let start_time = Instant::now();
    let progress_bar = progress::create_progress_bar(should_show_progress);

    let mut analyzer = RemoteAnalyzer::new();

    if let Some(token) = &cli.token {
        analyzer.set_github_token(token);
    }

    analyzer.set_timeout(cli.timeout);
    analyzer.set_allow_insecure(cli.allow_insecure);
    analyzer.set_progress_bar(progress_bar.clone());

    let project_analysis = analyzer.analyze_url(&processed_url).await?;

    let elapsed = start_time.elapsed();

    if let Some(pb) = &progress_bar {
        pb.finish_and_clear();
    }

    progress::show_completion_message(elapsed, cli.quiet);

    match format {
        OutputFormat::Table => {
            output::print_table_format(&project_analysis, cli.detailed, cli.quiet);
        }
        OutputFormat::Json => output::print_json_format(&project_analysis)?,
        OutputFormat::Csv => output::print_csv_format(&project_analysis)?,
        OutputFormat::Xml => output::print_xml_format(&project_analysis)?,
    }

    Ok(())
}
