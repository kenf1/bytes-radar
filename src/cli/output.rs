use super::progress::format_number;
use crate::core::{analysis::ProjectAnalysis, error::Result};
use colored::Colorize;
use serde_xml_rs;

pub fn print_table_format(project_analysis: &ProjectAnalysis, detailed: bool, quiet: bool) {
    let summary = project_analysis.get_summary();
    let language_stats = project_analysis.get_language_statistics();

    if !quiet {
        println!();
        println!("{}", "PROJECT SUMMARY".bright_white().bold());
        println!("{}", "─".repeat(80).bright_black());
    }

    println!(
        " {:<20} {}",
        "Project:".bright_blue(),
        summary.project_name.bright_white()
    );
    println!(
        " {:<20} {}",
        "Total Files:".bright_blue(),
        format_number(summary.total_files).bright_yellow()
    );
    println!(
        " {:<20} {}",
        "Total Lines:".bright_blue(),
        format_number(summary.total_lines).bright_yellow()
    );
    println!(
        " {:<20} {}",
        "Code Lines:".bright_blue(),
        format_number(summary.total_code_lines).bright_green()
    );
    println!(
        " {:<20} {}",
        "Comment Lines:".bright_blue(),
        format_number(summary.total_comment_lines).bright_cyan()
    );
    println!(
        " {:<20} {}",
        "Blank Lines:".bright_blue(),
        format_number(summary.total_blank_lines).bright_white()
    );
    println!(
        " {:<20} {}",
        "Languages:".bright_blue(),
        format_number(summary.language_count).bright_magenta()
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

    if !language_stats.is_empty() && !quiet {
        println!();
        println!("{}", "LANGUAGE BREAKDOWN".bright_white().bold());
        println!("{}", "─".repeat(80).bright_black());

        println!(
            " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8} {:>8}",
            "Language".bright_white().bold(),
            "Files".bright_white().bold(),
            "Lines".bright_white().bold(),
            "Code".bright_white().bold(),
            "Comments".bright_white().bold(),
            "Blanks".bright_white().bold(),
            "Share%".bright_white().bold(),
        );
        println!("{}", "─".repeat(80).bright_black());

        for stats in &language_stats {
            let share_percentage = if summary.total_lines > 0 {
                (stats.total_lines as f64 / summary.total_lines as f64) * 100.0
            } else {
                0.0
            };

            println!(
                " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8} {:>7.1}%",
                stats.language_name.bright_cyan(),
                format_number(stats.file_count).bright_white(),
                format_number(stats.total_lines).bright_yellow(),
                format_number(stats.code_lines).bright_green(),
                format_number(stats.comment_lines).bright_blue(),
                format_number(stats.blank_lines).bright_white(),
                share_percentage
            );
        }

        println!("{}", "─".repeat(80).bright_black());
        println!(
            " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8} {:>7.1}%",
            "Total".bright_green().bold(),
            format_number(summary.total_files).bright_green(),
            format_number(summary.total_lines).bright_green(),
            format_number(summary.total_code_lines).bright_green(),
            format_number(summary.total_comment_lines).bright_green(),
            format_number(summary.total_blank_lines).bright_green(),
            100.0
        );
    }

    if detailed && !quiet {
        println!();
        println!("{}", "FILE DETAILS".bright_white().bold());
        println!("{}", "─".repeat(80).bright_black());

        for (lang_name, analysis) in &project_analysis.language_analyses {
            if !analysis.file_metrics.is_empty() {
                println!();
                println!("{} Files", lang_name.bright_magenta().bold());

                for file in &analysis.file_metrics {
                    println!(
                        "   {:<50} {:>6} lines ({} code, {} comments)",
                        file.file_path.bright_white(),
                        format_number(file.total_lines).bright_yellow(),
                        format_number(file.code_lines).bright_green(),
                        format_number(file.comment_lines).bright_blue()
                    );
                }
            }
        }
    }
}

pub fn print_json_format(project_analysis: &ProjectAnalysis) -> Result<()> {
    let json = serde_json::to_string_pretty(project_analysis)?;
    println!("{}", json);
    Ok(())
}

pub fn print_csv_format(project_analysis: &ProjectAnalysis) -> Result<()> {
    let language_stats = project_analysis.get_language_statistics();
    let summary = project_analysis.get_summary();

    println!("Language,Files,Lines,Code,Comments,Blanks,SharePercent");
    for stats in language_stats {
        let share_percentage = if summary.total_lines > 0 {
            (stats.total_lines as f64 / summary.total_lines as f64) * 100.0
        } else {
            0.0
        };

        println!(
            "\"{}\",{},{},{},{},{},{:.2}",
            stats.language_name,
            format_number(stats.file_count),
            format_number(stats.total_lines),
            format_number(stats.code_lines),
            format_number(stats.comment_lines),
            format_number(stats.blank_lines),
            share_percentage
        );
    }

    Ok(())
}

pub fn print_xml_format(project_analysis: &ProjectAnalysis) -> Result<()> {
    #[derive(serde::Serialize)]
    struct XmlProjectAnalysis {
        project_name: String,
        summary: XmlSummary,
        language_statistics: Vec<XmlLanguageStats>,
    }

    #[derive(serde::Serialize)]
    struct XmlSummary {
        total_files: usize,
        total_lines: usize,
        total_code_lines: usize,
        total_comment_lines: usize,
        total_blank_lines: usize,
        language_count: usize,
        primary_language: Option<String>,
        overall_complexity_ratio: f64,
        overall_documentation_ratio: f64,
    }

    #[derive(serde::Serialize)]
    struct XmlLanguageStats {
        language_name: String,
        file_count: usize,
        total_lines: usize,
        code_lines: usize,
        comment_lines: usize,
        blank_lines: usize,
        complexity_ratio: f64,
    }

    let summary = project_analysis.get_summary();
    let language_stats = project_analysis.get_language_statistics();

    let xml_data = XmlProjectAnalysis {
        project_name: summary.project_name.clone(),
        summary: XmlSummary {
            total_files: summary.total_files,
            total_lines: summary.total_lines,
            total_code_lines: summary.total_code_lines,
            total_comment_lines: summary.total_comment_lines,
            total_blank_lines: summary.total_blank_lines,
            language_count: summary.language_count,
            primary_language: summary.primary_language.clone(),
            overall_complexity_ratio: summary.overall_complexity_ratio,
            overall_documentation_ratio: summary.overall_documentation_ratio,
        },
        language_statistics: language_stats
            .iter()
            .map(|stats| XmlLanguageStats {
                language_name: stats.language_name.clone(),
                file_count: stats.file_count,
                total_lines: stats.total_lines,
                code_lines: stats.code_lines,
                comment_lines: stats.comment_lines,
                blank_lines: stats.blank_lines,
                complexity_ratio: stats.complexity_ratio,
            })
            .collect(),
    };

    let xml = serde_xml_rs::to_string(&xml_data)
        .map_err(|e| crate::core::error::AnalysisError::xml_serialization(e.to_string()))?;

    println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    println!("{}", xml);
    Ok(())
}
