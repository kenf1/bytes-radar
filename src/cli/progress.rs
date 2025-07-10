use crate::net::traits::ProgressHook;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub fn create_progress_bar(show_progress: bool) -> Option<ProgressBar> {
    if !show_progress {
        return None;
    }

    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("[{elapsed_precise}] {spinner:.green} {msg} ({decimal_bytes_per_sec})")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    pb.set_message("Preparing...");
    pb.enable_steady_tick(Duration::from_millis(120));
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

pub struct ProgressBarHook {
    progress_bar: ProgressBar,
}

impl ProgressBarHook {
    pub fn new(progress_bar: ProgressBar) -> Self {
        Self { progress_bar }
    }
}

impl ProgressHook for ProgressBarHook {
    fn on_download_progress(&self, downloaded: u64, total: Option<u64>) {
        if let Some(total_size) = total {
            self.progress_bar.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {decimal_bytes_per_sec} {binary_bytes}/{binary_total_bytes} ({eta}) {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_bar())
                    .progress_chars("#>-"),
            );
            self.progress_bar.set_length(total_size);
            self.progress_bar.set_position(downloaded);
        } else {
            self.progress_bar.set_style(
                ProgressStyle::default_spinner()
                    .template(
                        "[{elapsed_precise}] {spinner:.green} {msg} ({decimal_bytes_per_sec})",
                    )
                    .unwrap_or_else(|_| ProgressStyle::default_spinner()),
            );
            self.progress_bar.set_position(downloaded);
            let formatted = format_bytes(downloaded);
            self.progress_bar
                .set_message(format!("Downloaded {}...", formatted));
        }
    }

    fn on_processing_start(&self, message: &str) {
        self.progress_bar.set_message(message.to_string());
    }

    fn on_processing_progress(&self, current: usize, total: usize) {
        if total > 0 {
            let percentage = (current * 100) / total;
            self.progress_bar.set_message(format!(
                "Processing files: {}% ({}/{})",
                percentage, current, total
            ));
        }
    }
}
