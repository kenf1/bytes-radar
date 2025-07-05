#[cfg(feature = "cli")]
use crate::core::*;
#[cfg(feature = "cli")]
use clap::Parser;
#[cfg(feature = "cli")]
use colored::Colorize;
#[cfg(feature = "cli")]
use std::time::Instant;

#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(name = "bytes-radar")]
#[command(about = "A tool for analyzing code statistics from remote repositories")]
#[command(version)]
pub struct Cli {
    #[arg(help = "URL to tar.gz archive, GitHub repo, GitLab repo, or commit")]
    pub url: Option<String>,

    #[arg(short, long, help = "Output format")]
    pub format: Option<OutputFormat>,

    #[arg(long, help = "Show detailed file-by-file statistics")]
    pub detailed: bool,

    #[arg(short = 'd', long = "debug", help = "Enable debug output")]
    pub debug: bool,
}

#[cfg(feature = "cli")]
#[derive(Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

#[cfg(feature = "cli")]
impl Default for OutputFormat {
    fn default() -> Self {
        Self::Table
    }
}

#[cfg(feature = "cli")]
pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    match cli.url {
        Some(url) => {
            analyze_remote_archive(&url, cli.format.unwrap_or_default(), cli.detailed).await
        }
        None => {
            println!("Error: URL argument is required");
            println!();
            println!("Usage: bytes-radar <URL>");
            println!();
            println!("Examples:");
            println!("  bytes-radar https://github.com/rust-lang/cargo");
            println!("  bytes-radar https://example.com/archive.tar.gz");
            println!("  bytes-radar -f json https://github.com/user/repo");
            println!();
            println!("For more information try --help");
            std::process::exit(1);
        }
    }
}

#[cfg(feature = "cli")]
async fn analyze_remote_archive(url: &str, format: OutputFormat, detailed: bool) -> Result<()> {
    println!("{}", "━".repeat(80).bright_black());
    println!(
        "{}",
        format!(" 🚀 Analyzing remote archive: {}", url)
            .bright_cyan()
            .bold()
    );
    println!("{}", "━".repeat(80).bright_black());

    let start_time = Instant::now();

    let analyzer = RemoteAnalyzer::new();
    let project_analysis = analyzer.analyze_url(url).await?;

    let elapsed = start_time.elapsed();

    match format {
        OutputFormat::Table => print_table_format(&project_analysis, detailed),
        OutputFormat::Json => print_json_format(&project_analysis)?,
        OutputFormat::Csv => print_csv_format(&project_analysis)?,
    }

    println!("{}", "━".repeat(80).bright_black());
    println!(
        " {} Analysis completed in {:.2}s",
        "✅".bright_green(),
        elapsed.as_secs_f64()
    );
    println!("{}", "━".repeat(80).bright_black());

    Ok(())
}

#[cfg(feature = "cli")]
fn print_table_format(project_analysis: &ProjectAnalysis, detailed: bool) {
    let summary = project_analysis.get_summary();
    let language_stats = project_analysis.get_language_statistics();

    println!();
    println!("{}", format!(" 📊 Project Summary").bright_white().bold());
    println!("{}", "─".repeat(80).bright_black());
    println!(
        " {:<20} {}",
        "Project:".bright_blue(),
        summary.project_name.bright_white()
    );
    println!(
        " {:<20} {}",
        "Total Files:".bright_blue(),
        summary.total_files.to_string().bright_yellow()
    );
    println!(
        " {:<20} {}",
        "Total Lines:".bright_blue(),
        summary.total_lines.to_string().bright_yellow()
    );
    println!(
        " {:<20} {}",
        "Code Lines:".bright_blue(),
        summary.total_code_lines.to_string().bright_green()
    );
    println!(
        " {:<20} {}",
        "Comment Lines:".bright_blue(),
        summary.total_comment_lines.to_string().bright_cyan()
    );
    println!(
        " {:<20} {}",
        "Blank Lines:".bright_blue(),
        summary.total_blank_lines.to_string().bright_white()
    );
    println!(
        " {:<20} {}",
        "Languages:".bright_blue(),
        summary.language_count.to_string().bright_magenta()
    );
    if let Some(ref primary) = summary.primary_language {
        println!(
            " {:<20} {}",
            "Primary Language:".bright_blue(),
            primary.bright_magenta()
        );
    }
    println!(
        " {:<20} {:.1}%",
        "Code Ratio:".bright_blue(),
        summary.overall_complexity_ratio * 100.0
    );
    println!(
        " {:<20} {:.1}%",
        "Documentation:".bright_blue(),
        summary.overall_documentation_ratio * 100.0
    );

    if !language_stats.is_empty() {
        println!();
        println!(
            "{}",
            format!(" 📈 Language Breakdown").bright_white().bold()
        );
        println!("{}", "─".repeat(80).bright_black());

        println!(
            " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8} {:>8}",
            "Language".bright_white().bold(),
            "Files".bright_white().bold(),
            "Lines".bright_white().bold(),
            "Code".bright_white().bold(),
            "Comments".bright_white().bold(),
            "Blanks".bright_white().bold(),
            "Code%".bright_white().bold(),
        );
        println!("{}", "─".repeat(80).bright_black());

        for stats in &language_stats {
            println!(
                " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8} {:>7.1}%",
                stats.language_name.bright_cyan(),
                stats.file_count.to_string().bright_white(),
                stats.total_lines.to_string().bright_yellow(),
                stats.code_lines.to_string().bright_green(),
                stats.comment_lines.to_string().bright_blue(),
                stats.blank_lines.to_string().bright_white(),
                stats.complexity_ratio * 100.0
            );
        }

        println!("{}", "─".repeat(80).bright_black());
        println!(
            " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8} {:>7.1}%",
            "Total".bright_green().bold(),
            summary.total_files.to_string().bright_green(),
            summary.total_lines.to_string().bright_green(),
            summary.total_code_lines.to_string().bright_green(),
            summary.total_comment_lines.to_string().bright_green(),
            summary.total_blank_lines.to_string().bright_green(),
            summary.overall_complexity_ratio * 100.0
        );
    }

    if detailed {
        println!();
        println!("{}", format!(" 📁 File Details").bright_white().bold());
        println!("{}", "─".repeat(80).bright_black());

        for (lang_name, analysis) in &project_analysis.language_analyses {
            if !analysis.file_metrics.is_empty() {
                println!();
                println!(
                    " {}",
                    format!("📄 {} Files", lang_name).bright_magenta().bold()
                );

                for file in &analysis.file_metrics {
                    println!(
                        "   {:<50} {:>6} lines ({} code, {} comments)",
                        file.file_path.bright_white(),
                        file.total_lines.to_string().bright_yellow(),
                        file.code_lines.to_string().bright_green(),
                        file.comment_lines.to_string().bright_blue()
                    );
                }
            }
        }
    }
}

#[cfg(feature = "cli")]
fn print_json_format(project_analysis: &ProjectAnalysis) -> Result<()> {
    let json = serde_json::to_string_pretty(project_analysis)
        .map_err(|e| AnalysisError::SerializationError { source: e })?;
    println!("{}", json);
    Ok(())
}

#[cfg(feature = "cli")]
fn print_csv_format(project_analysis: &ProjectAnalysis) -> Result<()> {
    let language_stats = project_analysis.get_language_statistics();

    println!("Language,Files,Lines,Code,Comments,Blanks,CodeRatio");
    for stats in language_stats {
        println!(
            "{},{},{},{},{},{},{:.2}",
            stats.language_name,
            stats.file_count,
            stats.total_lines,
            stats.code_lines,
            stats.comment_lines,
            stats.blank_lines,
            stats.complexity_ratio * 100.0
        );
    }

    Ok(())
}
