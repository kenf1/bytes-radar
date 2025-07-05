#[cfg(feature = "cli")]
use crate::core::*;
#[cfg(feature = "cli")]
use colored::Colorize;

#[cfg(feature = "cli")]
pub fn run() -> Result<()> {
    env_logger::init();

    print_example_analysis()?;

    Ok(())
}

#[cfg(feature = "cli")]
fn print_example_analysis() -> Result<()> {
    println!("{}", "━".repeat(80).bright_black());

    let mut project = ProjectAnalysis::new("Sample Project");

    // Add sample file analysis with correct API
    let rust_file = FileMetrics::new("src/main.rs", "Rust".to_string(), 5, 3, 1, 1)?;
    project.add_file_metrics(rust_file)?;

    let js_file = FileMetrics::new("src/app.js", "JavaScript".to_string(), 3, 2, 1, 0)?;
    project.add_file_metrics(js_file)?;

    // Get summary to access totals
    let summary = project.get_summary();

    // Print table header
    println!(
        "{}",
        format!(
            " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8}",
            "Language", "Files", "Lines", "Code", "Comments", "Blanks"
        )
        .bright_white()
    );
    println!("{}", "━".repeat(80).bright_black());

    // Print sample languages
    println!(
        " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8}",
        "Rust".bright_cyan(),
        "1",
        "5",
        "3",
        "1",
        "1"
    );
    println!(
        " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8}",
        "JavaScript".bright_cyan(),
        "1",
        "3",
        "2",
        "1",
        "0"
    );

    // Print separator and total
    println!("{}", "━".repeat(80).bright_black());
    println!(
        " {:<20} {:>8} {:>12} {:>8} {:>10} {:>8}",
        "Total".bright_green().bold(),
        summary.total_files.to_string().bright_green(),
        summary.total_lines.to_string().bright_green(),
        summary.total_code_lines.to_string().bright_green(),
        summary.total_comment_lines.to_string().bright_green(),
        summary.total_blank_lines.to_string().bright_green()
    );
    println!("{}", "━".repeat(80).bright_black());
    Ok(())
}
