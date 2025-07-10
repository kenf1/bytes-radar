use clap::{ArgGroup, Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "bradar")]
#[command(about = "A professional tool for analyzing code statistics from remote repositories")]
#[command(version)]
#[command(long_about = "
bradar - Bytes Radar: A hyper-fast code analysis tool for remote repositories.

SUPPORTED PLATFORMS:
  • GitHub (github.com, GitHub Enterprise)
  • GitLab (gitlab.com, self-hosted instances)
  • Bitbucket (bitbucket.org)
  • Codeberg (codeberg.org)
  • SourceForge (sourceforge.net)
  • Gitea instances
  • Azure DevOps
  • Direct archive URLs (tar.gz, tgz, zip)

URL FORMATS:
  user/repo                           # GitHub repo (default branch)
  user/repo@branch                    # Specific branch
  user/repo@commit-hash               # Specific commit
  https://github.com/user/repo        # Full GitHub URL
  https://gitlab.com/user/repo        # GitLab URL
  https://example.com/archive.tar.gz  # Direct archive URL

EXAMPLES:
  bradar microsoft/vscode
  bradar torvalds/linux@master
  bradar https://github.com/rust-lang/rust
  bradar --format json --detailed user/repo
  bradar --token ghp_xxx --include-tests private/repo
  bradar --aggressive-filter --max-file-size 2048 large/repo
")]
#[command(arg_required_else_help = true)]
#[command(disable_version_flag = true)]
#[command(group(
    ArgGroup::new("auth")
        .args(&["token"])
        .multiple(false)
))]
#[command(group(
    ArgGroup::new("output_control")
        .args(&["quiet", "debug"])
        .multiple(false)
))]
#[non_exhaustive]
pub struct Cli {
    #[arg(help = "Repository URL to analyze (user/repo, user/repo@branch, or full URL)")]
    pub url: Option<String>,

    // Version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version, help = "Print version information")]
    version: (),

    // Output Options
    #[arg(
        short = 'f',
        long = "format",
        help = "Output format",
        value_enum,
        default_value = "table"
    )]
    pub format: OutputFormat,

    #[arg(long = "detailed", help = "Show detailed file-by-file statistics")]
    pub detailed: bool,

    #[arg(
        short = 'q',
        long = "quiet",
        help = "Quiet mode - suppress progress and minimize output"
    )]
    pub quiet: bool,

    #[arg(long = "no-progress", help = "Disable progress bar")]
    pub no_progress: bool,

    #[arg(long = "no-color", help = "Disable colored output")]
    pub no_color: bool,

    // Authentication
    #[arg(long = "token", help = "Authentication token for private repositories")]
    pub token: Option<String>,

    // Network Options
    #[arg(
        long = "timeout",
        help = "Request timeout in seconds",
        default_value = "300",
        value_name = "SECONDS"
    )]
    pub timeout: u64,

    #[arg(long = "allow-insecure", help = "Allow insecure HTTPS connections")]
    pub allow_insecure: bool,

    #[arg(
        long = "user-agent",
        help = "Custom User-Agent string",
        value_name = "STRING"
    )]
    pub user_agent: Option<String>,

    #[arg(
        long = "retry-count",
        help = "Number of retry attempts for failed requests",
        default_value = "3",
        value_name = "COUNT"
    )]
    pub retry_count: u32,

    // Filtering Options
    #[arg(
        long = "aggressive-filter",
        help = "Enable aggressive filtering for maximum performance"
    )]
    pub aggressive_filter: bool,

    #[arg(
        long = "max-file-size",
        help = "Maximum file size to process in KB",
        default_value = "1024",
        value_name = "KB"
    )]
    pub max_file_size: u64,

    #[arg(long = "include-tests", help = "Include test directories in analysis")]
    pub include_tests: bool,

    #[arg(
        long = "include-docs",
        help = "Include documentation directories in analysis"
    )]
    pub include_docs: bool,

    #[arg(long = "include-hidden", help = "Include hidden files and directories")]
    pub include_hidden: bool,

    #[arg(
        long = "exclude-pattern",
        help = "Exclude files matching this pattern (glob)",
        value_name = "PATTERN"
    )]
    pub exclude_pattern: Option<String>,

    #[arg(
        long = "include-pattern",
        help = "Only include files matching this pattern (glob)",
        value_name = "PATTERN"
    )]
    pub include_pattern: Option<String>,

    #[arg(
        long = "min-file-size",
        help = "Minimum file size to process in bytes",
        default_value = "1",
        value_name = "BYTES"
    )]
    pub min_file_size: u64,

    // Language Options
    #[arg(
        long = "language",
        help = "Only analyze files of specific language",
        value_name = "LANG"
    )]
    pub language: Option<String>,

    #[arg(
        long = "exclude-language",
        help = "Exclude specific language from analysis",
        value_name = "LANG"
    )]
    pub exclude_language: Vec<String>,

    // Analysis Options
    #[arg(
        long = "ignore-whitespace",
        help = "Ignore whitespace-only lines in code analysis"
    )]
    pub ignore_whitespace: bool,

    #[arg(long = "count-generated", help = "Include generated files in analysis")]
    pub count_generated: bool,

    #[arg(
        long = "max-line-length",
        help = "Maximum line length to consider (0 = unlimited)",
        default_value = "0",
        value_name = "LENGTH"
    )]
    pub max_line_length: usize,

    // Debug and Logging
    #[arg(short = 'd', long = "debug", help = "Enable debug output")]
    pub debug: bool,

    #[arg(long = "trace", help = "Enable trace-level logging")]
    pub trace: bool,

    #[arg(long = "log-file", help = "Write logs to file", value_name = "FILE")]
    pub log_file: Option<String>,

    // Advanced Options
    #[arg(
        long = "threads",
        help = "Number of worker threads (0 = auto)",
        default_value = "0",
        value_name = "COUNT"
    )]
    pub threads: usize,

    #[arg(
        long = "memory-limit",
        help = "Memory limit in MB (0 = unlimited)",
        default_value = "0",
        value_name = "MB"
    )]
    pub memory_limit: usize,

    #[arg(
        long = "cache-dir",
        help = "Directory for caching downloaded files",
        value_name = "DIR"
    )]
    pub cache_dir: Option<String>,

    #[arg(long = "no-cache", help = "Disable caching of downloaded files")]
    pub no_cache: bool,

    // Experimental Features
    #[arg(
        long = "experimental-parallel",
        help = "Enable experimental parallel processing"
    )]
    pub experimental_parallel: bool,

    #[arg(
        long = "experimental-streaming",
        help = "Enable experimental streaming analysis"
    )]
    pub experimental_streaming: bool,
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    #[value(name = "table", help = "Human-readable table format")]
    Table,
    #[value(name = "json", help = "JSON format")]
    Json,
    #[value(name = "csv", help = "CSV format")]
    Csv,
    #[value(name = "xml", help = "XML format")]
    Xml,
    #[value(name = "yaml", help = "YAML format")]
    Yaml,
    #[value(name = "toml", help = "TOML format")]
    Toml,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Table
    }
}
