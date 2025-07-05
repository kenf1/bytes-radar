use super::progress::format_number;
use crate::core::{analysis::ProjectAnalysis, error::Result};
use colored::Colorize;
use serde_xml_rs;

pub fn print_table_format(project_analysis: &ProjectAnalysis, detailed: bool, quiet: bool) {
    let summary = project_analysis.get_summary();
    let language_stats = project_analysis.get_language_statistics();

    if !quiet {
        println!("{}", "=".repeat(80));
    }

    println!(" {:<56} {}", "Project", summary.project_name);
    println!(
        " {:<56} {}",
        "Total Files",
        format_number(summary.total_files)
    );
    println!(
        " {:<56} {}",
        "Total Lines",
        format_number(summary.total_lines)
    );
    println!(
        " {:<56} {}",
        "Code Lines",
        format_number(summary.total_code_lines)
    );
    println!(
        " {:<56} {}",
        "Comment Lines",
        format_number(summary.total_comment_lines)
    );
    println!(
        " {:<56} {}",
        "Blank Lines",
        format_number(summary.total_blank_lines)
    );
    println!(
        " {:<56} {}",
        "Languages",
        format_number(summary.language_count)
    );
    if let Some(ref primary) = summary.primary_language {
        println!(" {:<56} {}", "Primary Language", primary);
    }
    println!(
        " {:<56} {:.1}%",
        "Code Ratio",
        summary.overall_complexity_ratio * 100.0
    );
    println!(
        " {:<56} {:.1}%",
        "Documentation",
        summary.overall_documentation_ratio * 100.0
    );

    if !language_stats.is_empty() && !quiet {
        println!("{}", "=".repeat(80));

        println!(
            " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8} {:>8}",
            "Language".bold(),
            "Files",
            "Lines",
            "Code",
            "Comments",
            "Blanks",
            "Share%"
        );
        println!("{}", "=".repeat(80));

        for stats in &language_stats {
            let share_percentage = if summary.total_lines > 0 {
                (stats.total_lines as f64 / summary.total_lines as f64) * 100.0
            } else {
                0.0
            };

            println!(
                " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8} {:>7.1}%",
                stats.language_name,
                format_number(stats.file_count),
                format_number(stats.total_lines),
                format_number(stats.code_lines),
                format_number(stats.comment_lines),
                format_number(stats.blank_lines),
                share_percentage
            );
        }

        println!("{}", "=".repeat(80));
        println!(
            " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8} {:>7.1}%",
            "Total".bold(),
            format_number(summary.total_files),
            format_number(summary.total_lines),
            format_number(summary.total_code_lines),
            format_number(summary.total_comment_lines),
            format_number(summary.total_blank_lines),
            100.0
        );
    }

    if detailed && !quiet {
        println!("{}", "=".repeat(80));

        for (lang_name, analysis) in &project_analysis.language_analyses {
            if !analysis.file_metrics.is_empty() {
                println!();
                println!("{} Files", lang_name.bold());

                for file in &analysis.file_metrics {
                    println!(
                        "   {:<50} {:>6} lines ({} code, {} comments)",
                        file.file_path,
                        format_number(file.total_lines),
                        format_number(file.code_lines),
                        format_number(file.comment_lines)
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
