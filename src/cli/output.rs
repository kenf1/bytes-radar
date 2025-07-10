use super::progress::format_number;
use crate::core::{analysis::ProjectAnalysis, error::Result};
use colored::Colorize;

fn get_percentage_color(percentage: f64) -> colored::ColoredString {
    let percentage_str = format!("{:.1}%", percentage);
    if percentage >= 50.0 {
        percentage_str.bright_green()
    } else if percentage >= 10.0 {
        percentage_str.yellow()
    } else if percentage >= 1.0 {
        percentage_str.white()
    } else {
        percentage_str.dimmed()
    }
}

fn color_number(num: usize) -> colored::ColoredString {
    format_number(num).bright_white()
}

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
        color_number(summary.total_files)
    );
    println!(
        " {:<56} {}",
        "Total Lines",
        color_number(summary.total_lines)
    );
    println!(
        " {:<56} {}",
        "Code Lines",
        color_number(summary.total_code_lines)
    );
    println!(
        " {:<56} {}",
        "Comment Lines",
        color_number(summary.total_comment_lines)
    );
    println!(
        " {:<56} {}",
        "Blank Lines",
        color_number(summary.total_blank_lines)
    );
    println!(
        " {:<56} {}",
        "Languages",
        color_number(summary.language_count)
    );
    if let Some(ref primary) = summary.primary_language {
        println!(" {:<56} {}", "Primary Language", primary);
    }
    println!(
        " {:<56} {}",
        "Code Ratio",
        format!("{:.1}%", summary.overall_complexity_ratio * 100.0).bold()
    );
    println!(
        " {:<56} {}",
        "Documentation",
        format!("{:.1}%", summary.overall_documentation_ratio * 100.0).bold()
    );

    if !language_stats.is_empty() && !quiet {
        println!("{}", "=".repeat(80));

        println!(
            " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8} {:>7}",
            "Language".bold(),
            "Files",
            "Lines",
            "Code",
            "Comments",
            "Blanks",
            "%"
        );
        println!("{}", "=".repeat(80));

        for stats in &language_stats {
            let share_percentage = if summary.total_lines > 0 {
                (stats.total_lines as f64 / summary.total_lines as f64) * 100.0
            } else {
                0.0
            };

            println!(
                " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8} {:>7}",
                stats.language_name,
                color_number(stats.file_count),
                color_number(stats.total_lines),
                color_number(stats.code_lines),
                color_number(stats.comment_lines),
                color_number(stats.blank_lines),
                get_percentage_color(share_percentage)
            );
        }

        println!("{}", "=".repeat(80));
        println!(
            " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8} {:>7}",
            "Total".bold(),
            color_number(summary.total_files),
            color_number(summary.total_lines),
            color_number(summary.total_code_lines),
            color_number(summary.total_comment_lines),
            color_number(summary.total_blank_lines),
            "%"
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
                        color_number(file.total_lines),
                        color_number(file.code_lines),
                        color_number(file.comment_lines)
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
    let summary = project_analysis.get_summary();
    let language_stats = project_analysis.get_language_statistics();

    println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    println!("<project_analysis>");

    println!(
        "  <project_name>{}</project_name>",
        xml_escape(&summary.project_name)
    );

    println!("  <summary>");
    println!("    <total_files>{}</total_files>", summary.total_files);
    println!("    <total_lines>{}</total_lines>", summary.total_lines);
    println!(
        "    <total_code_lines>{}</total_code_lines>",
        summary.total_code_lines
    );
    println!(
        "    <total_comment_lines>{}</total_comment_lines>",
        summary.total_comment_lines
    );
    println!(
        "    <total_blank_lines>{}</total_blank_lines>",
        summary.total_blank_lines
    );
    println!(
        "    <language_count>{}</language_count>",
        summary.language_count
    );

    if let Some(ref primary_lang) = summary.primary_language {
        println!(
            "    <primary_language>{}</primary_language>",
            xml_escape(primary_lang)
        );
    }

    println!(
        "    <overall_complexity_ratio>{:.6}</overall_complexity_ratio>",
        summary.overall_complexity_ratio
    );
    println!(
        "    <overall_documentation_ratio>{:.6}</overall_documentation_ratio>",
        summary.overall_documentation_ratio
    );
    println!("  </summary>");

    println!("  <language_statistics>");
    for stats in language_stats {
        println!("    <language>");
        println!("      <n>{}</n>", xml_escape(&stats.language_name));
        println!("      <file_count>{}</file_count>", stats.file_count);
        println!("      <total_lines>{}</total_lines>", stats.total_lines);
        println!("      <code_lines>{}</code_lines>", stats.code_lines);
        println!(
            "      <comment_lines>{}</comment_lines>",
            stats.comment_lines
        );
        println!("      <blank_lines>{}</blank_lines>", stats.blank_lines);
        println!(
            "      <complexity_ratio>{:.6}</complexity_ratio>",
            stats.complexity_ratio
        );
        println!("    </language>");
    }
    println!("  </language_statistics>");

    println!("</project_analysis>");
    Ok(())
}

pub fn print_yaml_format(project_analysis: &ProjectAnalysis) -> Result<()> {
    let yaml = serde_yaml::to_string(project_analysis)
        .map_err(|e| crate::core::error::AnalysisError::invalid_statistics(e.to_string()))?;
    println!("{}", yaml);
    Ok(())
}

pub fn print_toml_format(project_analysis: &ProjectAnalysis) -> Result<()> {
    let toml = toml::to_string_pretty(project_analysis)
        .map_err(|e| crate::core::error::AnalysisError::invalid_statistics(e.to_string()))?;
    println!("{}", toml);
    Ok(())
}
fn xml_escape(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;")
}
