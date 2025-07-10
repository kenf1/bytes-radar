# Bytes Radar

[![CI](https://github.com/zmh-program/bytes-radar/workflows/CI/badge.svg)](https://github.com/zmh-program/bytes-radar/actions)
[![Crates.io](https://img.shields.io/crates/v/bytes-radar.svg)](https://crates.io/crates/bytes-radar)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

[![Deploy to Cloudflare Workers](https://deploy.workers.cloudflare.com/button.svg)](https://deploy.workers.cloudflare.com/?url=https://github.com/zmh-program/bytes-radar)

Hyper-fast **CLOC** _(\*count lines of code)_ tool for remote repositories.

![Banner](docs/banner.jpg)

## Features

- Efficient remote repository analysis with async streaming and in-memory decompression, optimized for low memory usage (always <12MB runtime mem)
- Unified URL parsing for GitHub, GitLab, Bitbucket, Codeberg, SourceForge, Gitea and Azure DevOps
- Rule-based language detection supporting 150+ programming languages with [Tokei's Language Rules](https://github.com/XAMPPRocky/tokei/blob/master/languages.json)
- Real-time progress tracking with download speed, ETA and adaptive terminal UI
- Multiple output formats (Table, JSON, CSV, XML, YAML, TOML) with schema validation
- OAuth token management for private repository access
- Native binaries for Linux, macOS and Windows
- Experimental parallel processing and streaming analysis

## Supported Platforms

| Platform           | URL Format                                | Example                                     |
| ------------------ | ----------------------------------------- | ------------------------------------------- |
| **GitHub**         | `user/repo`, `user/repo@branch`, full URL | `microsoft/vscode`, `torvalds/linux@master` |
| **GitLab**         | Full URL                                  | `https://gitlab.com/user/repo`              |
| **Bitbucket**      | Full URL                                  | `https://bitbucket.org/user/repo`           |
| **Codeberg**       | Full URL                                  | `https://codeberg.org/user/repo`            |
| **SourceForge**    | Full URL                                  | `https://sourceforge.net/user/repo`         |
| **Gitea**          | Full URL                                  | `https://gitea.example.com/user/repo`       |
| **Azure DevOps**   | Full URL                                  | `https://dev.azure.com/org/project`         |
| **Direct Archive** | tar.gz, tgz, zip URL                      | `https://example.com/archive.tar.gz`        |

## Installation

Download the latest binary from **[GitHub Releases](https://github.com/zmh-program/bytes-radar/releases)** or install via Cargo:

```bash
cargo install bytes-radar
```

## Usage

#### Basic Repo Analysis

```bash
bradar torvalds/linux # or https://github.com/torvalds/linux
```

#### Branch and Commit Targeting

Specify particular branches or commit hashes for analysis:

```bash
bradar microsoft/vscode@main # or https://github.com/microsoft/vscode/tree/main
bradar kubernetes/kubernetes@release-1.28 # or https://github.com/kubernetes/kubernetes/tree/release-1.28
```

#### Multi-Platform Repository Support

Analyze repositories from different Git hosting platforms:

```bash
bradar https://gitlab.com/gitlab-org/gitlab
bradar https://bitbucket.org/atlassian/stash
bradar https://codeberg.org/forgejo/forgejo
```

#### Output Format Configuration

Generate analysis results in structured data formats:

```bash
bradar -f json torvalds/linux
```

#### Private Repository Access

Authenticate with platform tokens for private repository analysis:

```bash
bradar --token ghp_xxxxxxxxxxxxxxxxxxxx private-org/confidential-repo
bradar --token glpat-xxxxxxxxxxxxxxxxxxxx https://gitlab.com/private-group/project
```

#### Performance and Output Control

Configure analysis behavior and output verbosity:

```bash
bradar --quiet --no-progress user/repo
bradar --timeout 600 --detailed large-org/massive-repo
```

## CLI Options

```bash
bradar [OPTIONS] <URL>

ARGUMENTS:
  <URL>  Repository URL to analyze (user/repo, user/repo@branch, or full URL)

OPTIONS:
  # Output Options
  -f, --format <FORMAT>        Output format [table|json|csv|xml|yaml|toml]
      --detailed              Show detailed file-by-file statistics
  -q, --quiet                Quiet mode - suppress progress and minimize output
      --no-progress          Disable progress bar
      --no-color             Disable colored output

  # Authentication
      --token <TOKEN>        Authentication token for private repositories

  # Network Options
      --timeout <SECONDS>    Request timeout in seconds [default: 300]
      --allow-insecure       Allow insecure HTTPS connections
      --user-agent <STRING>  Custom User-Agent string
      --retry-count <COUNT>  Number of retry attempts for failed requests [default: 3]

  # Filtering Options
      --aggressive-filter    Enable aggressive filtering for maximum performance
      --max-file-size <KB>  Maximum file size to process in KB [default: 1024]
      --min-file-size <BYTES> Minimum file size to process in bytes [default: 1]
      --include-tests       Include test directories in analysis
      --include-docs        Include documentation directories in analysis
      --include-hidden      Include hidden files and directories
      --exclude-pattern <PATTERN>  Exclude files matching this pattern (glob)
      --include-pattern <PATTERN>  Only include files matching this pattern (glob)

  # Language Options
      --language <LANG>     Only analyze files of specific language
      --exclude-language <LANG>  Exclude specific language from analysis

  # Analysis Options
      --ignore-whitespace   Ignore whitespace-only lines in code analysis
      --count-generated     Include generated files in analysis
      --max-line-length <LENGTH>  Maximum line length to consider (0 = unlimited) [default: 0]

  # Debug and Logging
  -d, --debug              Enable debug output
      --trace              Enable trace-level logging
      --log-file <FILE>    Write logs to file

  # Advanced Options
      --threads <COUNT>     Number of worker threads (0 = auto) [default: 0]
      --memory-limit <MB>   Memory limit in MB (0 = unlimited) [default: 0]
      --cache-dir <DIR>    Directory for caching downloaded files
      --no-cache           Disable caching of downloaded files

  # Experimental Features
      --experimental-parallel    Enable experimental parallel processing
      --experimental-streaming  Enable experimental streaming analysis

  # General
  -v, --version           Print version
  -h, --help             Print help
```

See [CLI USAGE GUIDE](docs/CLI_USAGE.md) for more detailed usage examples and advanced configurations.

## Deployment

### Cloudflare Workers

[![Deploy to Cloudflare Workers](https://deploy.workers.cloudflare.com/button.svg)](https://deploy.workers.cloudflare.com/?url=https://github.com/zmh-program/bytes-radar)

> [!TIP]
> The Free Tier of Cloudflare Workers has a **20s request timeout limit** (wall time). Analysis of large repositories may fail due to this limitation. Consider upgrading to Cloudflare Workers Pro or using alternative methods for processing large repositories.

For detailed deployment instructions and API documentation, see [DEPLOYMENT GUIDE](docs/DEPLOYMENT.md).

## Contributing

We welcome contributions! Please see [CONTRIBUTING GUIDE](docs/CONTRIBUTING.md) for guidelines.
