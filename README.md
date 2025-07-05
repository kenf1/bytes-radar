# bytes-radar

[![CI](https://github.com/zmh-program/bytes-radar/workflows/CI/badge.svg)](https://github.com/zmh-program/bytes-radar/actions)
[![Crates.io](https://img.shields.io/crates/v/bytes-radar.svg)](https://crates.io/crates/bytes-radar)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

A fast code analysis tool for remote repositories with multi-platform support.

## Features

- **Asynchronous Repository Processing**: Implements non-blocking HTTP client with connection pooling and concurrent stream processing for efficient remote repository fetching and decompression
- **Multi-Platform URL Resolution**: Features intelligent URL parsing engine that normalizes different Git hosting platform APIs (GitHub, GitLab, Bitbucket, Codeberg) into unified archive endpoints with branch/commit resolution
- **Streaming Archive Analysis**: Processes tar.gz archives directly in memory using streaming decompression without temporary file extraction, reducing I/O overhead and memory footprint
- **Language Detection Engine**: Implements rule-based file extension and content analysis system supporting 150+ programming languages with configurable pattern matching and statistical computation
- **Real-time Progress Monitoring**: Features bandwidth-aware progress tracking with download speed calculation, ETA estimation, and adaptive UI rendering for terminal environments
- **Structured Data Serialization**: Provides multiple output format engines (Table, JSON, CSV, XML) with schema validation and type-safe serialization for integration with external tools
- **Authentication Layer**: Implements OAuth token management with secure credential handling for accessing private repositories across different hosting platforms
- **Cross-Platform Binary Distribution**: Supports native compilation targets for Linux, macOS, and Windows with platform-specific optimizations and dependency management

## Installation

### From Cargo (Recommended)

```bash
cargo install bytes-radar
```

### From Releases

Download the latest binary from [GitHub Releases](https://github.com/zmh-program/bytes-radar/releases)

### From Source

```bash
git clone https://github.com/zmh-program/bytes-radar.git
cd bytes-radar
cargo build --release
```

## Usage

```bash
bytes-radar [OPTIONS] <URL>
```

### Examples

#### Basic Repository Analysis

Analyze GitHub repositories using shorthand notation:

```bash
bytes-radar torvalds/linux
bytes-radar microsoft/typescript
bytes-radar rust-lang/cargo
```

#### Branch and Commit Targeting

Specify particular branches or commit hashes for analysis:

```bash
bytes-radar microsoft/vscode@main
bytes-radar kubernetes/kubernetes@release-1.28
bytes-radar rust-lang/rust@abc1234567
```

#### Multi-Platform Repository Support

Analyze repositories from different Git hosting platforms:

```bash
bytes-radar https://gitlab.com/gitlab-org/gitlab
bytes-radar https://bitbucket.org/atlassian/stash
bytes-radar https://codeberg.org/forgejo/forgejo
```

#### Output Format Configuration

Generate analysis results in structured data formats:

```bash
bytes-radar -f json torvalds/linux
bytes-radar -f csv microsoft/typescript
bytes-radar -f xml rust-lang/cargo
```

#### Private Repository Access

Authenticate with platform tokens for private repository analysis:

```bash
bytes-radar --token ghp_xxxxxxxxxxxxxxxxxxxx private-org/confidential-repo
bytes-radar --token glpat-xxxxxxxxxxxxxxxxxxxx https://gitlab.com/private-group/project
```

#### Performance and Output Control

Configure analysis behavior and output verbosity:

```bash
bytes-radar --quiet --no-progress user/repo
bytes-radar --timeout 600 --detailed large-org/massive-repo
```

## Output Formats

### Table (Default)
```shell
$ bytes-radar torvalds/linux
Analyzing: https://github.com/torvalds/linux
Analysis completed in 126.36s

================================================================================
 Project                                                  linux@main
 Total Files                                              89,639
 Total Lines                                              40,876,027
 Code Lines                                               31,293,116
 Comment Lines                                            4,433,479
 Blank Lines                                              5,149,432
 Languages                                                14
 Primary Language                                         C
 Code Ratio                                               76.6%
 Documentation                                            14.2%
================================================================================
 Language                Files        Lines     Code   Comments   Blanks   Share%
================================================================================
 C                      35,586   25,268,107 18,782,347  2,836,806 3,648,954    61.8%
 CHeader                25,845   10,247,647 7,953,679  1,528,043  765,925    25.1%
 Text                   20,954    3,917,052 3,324,410          0  592,642     9.6%
 Json                      961      572,657  572,655          0        2     1.4%
 Yaml                    4,862      548,408  436,698     22,250   89,460     1.3%
 Sh                        960      189,965  132,288     23,686   33,991     0.5%
 Python                    293       89,285   69,449      5,770   14,066     0.2%
 Rust                      158       39,561   19,032     16,697    3,832     0.1%
 Cpp                         7        2,267    1,836         96      335     0.0%
 Markdown                    3          578      436          0      142     0.0%
 Css                         3          295      172         69       54     0.0%
 CppHeader                   2          125       59         47       19     0.0%
 Toml                        3           47       28         12        7     0.0%
 Html                        2           33       27          3        3     0.0%
================================================================================
 Total                  89,639   40,876,027 31,293,116  4,433,479 5,149,432   100.0%
```

### JSON Output
```json
{
  "project_name": "linux@master",
  "summary": {
    "total_files": 75823,
    "total_lines": 28691744,
    "code_lines": 22453891,
    "comment_lines": 3891234,
    "blank_lines": 2346619
  },
  "language_statistics": [...]
}
```

## Supported Platforms

| Platform | URL Format | Example |
|----------|------------|---------|
| **GitHub** | `user/repo` or full URL | `torvalds/linux` |
| **GitLab** | Full URL | `https://gitlab.com/user/repo` |
| **Bitbucket** | Full URL | `https://bitbucket.org/user/repo` |
| **Codeberg** | Full URL | `https://codeberg.org/user/repo` |
| **Direct** | tar.gz URL | `https://example.com/file.tar.gz` |

## CLI Options

```bash
bytes-radar [OPTIONS] <URL>

ARGUMENTS:
  <URL>  URL to analyze: user/repo, user/repo@branch, or full URL

OPTIONS:
  -f, --format <FORMAT>        Output format [table|json|csv|xml]
      --detailed               Show detailed file-by-file statistics
  -d, --debug                  Enable debug output
      --token <TOKEN>          GitHub token for private repositories
      --timeout <SECONDS>      Request timeout in seconds [default: 300]
      --allow-insecure         Allow insecure HTTP connections
      --no-progress           Disable progress bar
      --quiet                 Quiet mode - minimal output
  -h, --help                  Print help
  -V, --version               Print version
```

## Architecture

bytes-radar is built with a modular architecture:

```
src/
├── cli/                 # Command-line interface
│   ├── args.rs         # Argument parsing
│   ├── output.rs       # Output formatting
│   ├── progress.rs     # Progress bar handling
│   └── url_parser.rs   # URL processing
├── core/               # Core analysis logic
│   ├── analysis.rs     # File analysis
│   ├── net.rs         # Network operations
│   ├── registry.rs    # Language detection
│   └── error.rs       # Error handling
└── lib.rs             # Library entry point
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/zmh-program/bytes-radar.git
cd bytes-radar

# Install dependencies
cargo build

# Run tests
cargo test --all-features

# Format code
cargo fmt

# Lint code
cargo clippy --all-targets --all-features
```
