use super::ProgressHook;
use crate::core::{
    analysis::{FileMetrics, ProjectAnalysis},
    error::{AnalysisError, Result},
    filter::{FilterStats, IntelligentFilter},
    registry::LanguageRegistry,
};
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use std::io::{Cursor, Read};
use tar::Archive;
use tokio::sync::mpsc;

#[cfg(not(target_arch = "wasm32"))]
use tokio::task;

pub type ProgressCallback = Box<dyn Fn(u64, Option<u64>) + Send + Sync>;

pub struct StreamReader {
    receiver: mpsc::Receiver<std::io::Result<bytes::Bytes>>,
    current_chunk: Option<Cursor<bytes::Bytes>>,
    finished: bool,
}

impl StreamReader {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(
        stream: impl futures_util::Stream<Item = reqwest::Result<bytes::Bytes>> + Send + 'static,
        progress_callback: ProgressCallback,
        total_size: Option<u64>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut downloaded = 0u64;
            let mut stream = Box::pin(stream);

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        downloaded += chunk.len() as u64;
                        progress_callback(downloaded, total_size);

                        if tx.send(Ok(chunk)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Stream error: {}", e),
                            )))
                            .await;
                        break;
                    }
                }
            }
        });

        Self {
            receiver: rx,
            current_chunk: None,
            finished: false,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(
        stream: impl futures_util::Stream<Item = reqwest::Result<bytes::Bytes>> + 'static,
        progress_callback: ProgressCallback,
        total_size: Option<u64>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(32);

        wasm_bindgen_futures::spawn_local(async move {
            let mut downloaded = 0u64;
            let mut stream = Box::pin(stream);

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        downloaded += chunk.len() as u64;
                        progress_callback(downloaded, total_size);

                        if tx.send(Ok(chunk)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Stream error: {}", e),
                            )))
                            .await;
                        break;
                    }
                }
            }
        });

        Self {
            receiver: rx,
            current_chunk: None,
            finished: false,
        }
    }
}

impl Read for StreamReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(ref mut cursor) = self.current_chunk {
            let read = cursor.read(buf)?;
            if read > 0 {
                return Ok(read);
            }
            self.current_chunk = None;
        }

        if self.finished {
            return Ok(0);
        }

        match self.receiver.try_recv() {
            Ok(Ok(chunk)) => {
                self.current_chunk = Some(Cursor::new(chunk));
                if let Some(ref mut cursor) = self.current_chunk {
                    cursor.read(buf)
                } else {
                    Ok(0)
                }
            }
            Ok(Err(e)) => {
                self.finished = true;
                Err(e)
            }
            Err(mpsc::error::TryRecvError::Empty) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    match self.receiver.blocking_recv() {
                        Some(Ok(chunk)) => {
                            self.current_chunk = Some(Cursor::new(chunk));
                            if let Some(ref mut cursor) = self.current_chunk {
                                cursor.read(buf)
                            } else {
                                Ok(0)
                            }
                        }
                        Some(Err(e)) => {
                            self.finished = true;
                            Err(e)
                        }
                        None => {
                            self.finished = true;
                            Ok(0)
                        }
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::WouldBlock,
                        "Would block in WASM",
                    ))
                }
            }
            Err(mpsc::error::TryRecvError::Disconnected) => {
                self.finished = true;
                Ok(0)
            }
        }
    }
}

pub async fn process_tarball(
    bytes: bytes::Bytes,
    project_analysis: &mut ProjectAnalysis,
    filter: &IntelligentFilter,
    _progress_hook: &dyn ProgressHook,
) -> Result<()> {
    let decoder = GzDecoder::new(Cursor::new(bytes));
    let mut archive = Archive::new(decoder);

    let entries = archive
        .entries()
        .map_err(|e| AnalysisError::archive(format!("Failed to read tar entries: {}", e)))?;

    let mut stats = FilterStats::new();

    for entry in entries {
        let entry = entry
            .map_err(|e| AnalysisError::archive(format!("Failed to read tar entry: {}", e)))?;

        if let Ok(metrics) = process_tar_entry_sync(entry, filter, &mut stats) {
            project_analysis.add_file_metrics(metrics)?;
        }
    }

    #[cfg(feature = "cli")]
    log::info!(
        "Filter stats: processed {}/{} files ({:.1}% filtered), saved {}",
        stats.processed,
        stats.total_entries,
        stats.filter_ratio() * 100.0,
        stats.format_bytes_saved()
    );

    Ok(())
}

pub async fn process_tarball_stream(
    stream_reader: StreamReader,
    project_analysis: &mut ProjectAnalysis,
    filter: &IntelligentFilter,
    _progress_hook: &dyn ProgressHook,
) -> Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let filter = filter.clone();
        let metrics_result = task::spawn_blocking(move || {
            let decoder = GzDecoder::new(stream_reader);
            let mut archive = Archive::new(decoder);

            let entries = archive.entries().map_err(|e| {
                AnalysisError::archive(format!("Failed to read tar entries: {}", e))
            })?;

            let mut collected_metrics = Vec::new();
            let mut stats = FilterStats::new();

            for entry in entries {
                let entry = entry.map_err(|e| {
                    AnalysisError::archive(format!("Failed to read tar entry: {}", e))
                })?;

                if let Ok(metrics) = process_tar_entry_sync(entry, &filter, &mut stats) {
                    collected_metrics.push(metrics);
                }
            }

            #[cfg(feature = "cli")]
            log::info!(
                "Filter stats: processed {}/{} files ({:.1}% filtered), saved {}",
                stats.processed,
                stats.total_entries,
                stats.filter_ratio() * 100.0,
                stats.format_bytes_saved()
            );

            Ok::<Vec<FileMetrics>, AnalysisError>(collected_metrics)
        })
        .await
        .map_err(|e| AnalysisError::archive(format!("Task join error: {}", e)))??;

        for metrics in metrics_result {
            project_analysis.add_file_metrics(metrics)?;
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        let decoder = GzDecoder::new(stream_reader);
        let mut archive = Archive::new(decoder);

        let entries = archive
            .entries()
            .map_err(|e| AnalysisError::archive(format!("Failed to read tar entries: {}", e)))?;

        let mut stats = FilterStats::new();

        for entry in entries {
            let entry = entry
                .map_err(|e| AnalysisError::archive(format!("Failed to read tar entry: {}", e)))?;

            if let Ok(metrics) = process_tar_entry_sync(entry, filter, &mut stats) {
                project_analysis.add_file_metrics(metrics)?;
            }
        }
    }

    Ok(())
}

fn process_tar_entry_sync<R: Read>(
    mut entry: tar::Entry<'_, R>,
    filter: &IntelligentFilter,
    stats: &mut FilterStats,
) -> Result<FileMetrics> {
    let header = entry.header();
    let path = header
        .path()
        .map_err(|e| AnalysisError::archive(format!("Invalid path in tar entry: {}", e)))?;

    let file_path = path.to_string_lossy().to_string();

    if !header.entry_type().is_file() || header.size().unwrap_or(0) == 0 {
        return Err(AnalysisError::archive("Not a file or empty".to_string()));
    }

    let file_size = header.size().unwrap_or(0);

    let should_process = filter.should_process_file(&file_path, file_size);
    stats.record_entry(file_size, !should_process);

    if !should_process {
        return Err(AnalysisError::archive("File filtered out".to_string()));
    }

    let language = LanguageRegistry::detect_by_path(&file_path)
        .map(|l| l.name.clone())
        .unwrap_or_else(|| "Text".to_string());

    let mut content = String::new();
    if entry.read_to_string(&mut content).is_err() {
        return Err(AnalysisError::archive(
            "Failed to read file content".to_string(),
        ));
    }

    analyze_file_content(&file_path, &content, &language, file_size)
}

fn analyze_file_content(
    file_path: &str,
    content: &str,
    language: &str,
    file_size: u64,
) -> Result<FileMetrics> {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let mut code_lines = 0;
    let mut comment_lines = 0;
    let mut blank_lines = 0;

    let lang_def = LanguageRegistry::get_language(language);
    let empty_line_comments = vec![];
    let empty_multi_line_comments = vec![];
    let line_comments = lang_def
        .map(|l| &l.line_comments)
        .unwrap_or(&empty_line_comments);
    let multi_line_comments = lang_def
        .map(|l| &l.multi_line_comments)
        .unwrap_or(&empty_multi_line_comments);

    let mut in_multi_line_comment = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            blank_lines += 1;
            continue;
        }

        let mut is_comment = false;

        if !in_multi_line_comment {
            for comment_start in line_comments {
                if trimmed.starts_with(comment_start) {
                    is_comment = true;
                    break;
                }
            }

            for (start, end) in multi_line_comments {
                if trimmed.starts_with(start) {
                    is_comment = true;
                    if !trimmed.ends_with(end) {
                        in_multi_line_comment = true;
                    }
                    break;
                }
            }
        } else {
            is_comment = true;
            for (_, end) in multi_line_comments {
                if trimmed.ends_with(end) {
                    in_multi_line_comment = false;
                    break;
                }
            }
        }

        if is_comment {
            comment_lines += 1;
        } else {
            code_lines += 1;
        }
    }

    let metrics = FileMetrics::new(
        file_path,
        language.to_string(),
        total_lines,
        code_lines,
        comment_lines,
        blank_lines,
    )?
    .with_size_bytes(file_size);

    Ok(metrics)
}
