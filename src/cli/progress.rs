use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub fn create_progress_bar(show_progress: bool) -> Option<ProgressBar> {
    if !show_progress {
        return None;
    }

    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("[{elapsed_precise}] {spinner:.green} {decimal_bytes_per_sec} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    pb.set_message("Preparing...");
    Some(pb)
}

pub fn show_completion_message(elapsed: Duration, quiet: bool) {
    if !quiet {
        println!("Analysis completed in {:.2}s", elapsed.as_secs_f64());
        println!();
    }
}

pub fn format_number(num: usize) -> String {
    let num_str = num.to_string();
    let mut result = String::new();
    let chars: Vec<char> = num_str.chars().collect();

    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }
    result
}

#[allow(dead_code)]
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    const THRESHOLD: f64 = 1024.0;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
