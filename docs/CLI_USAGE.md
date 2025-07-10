# Bytes Radar CLI Usage Guide

`bradar` is a professional command-line tool for analyzing code statistics from remote repositories with hyper-fast performance.

## Table of Contents

- [Installation](#installation)
- [Basic Usage](#basic-usage)
- [Supported Platforms](#supported-platforms)
- [URL Formats](#url-formats)
- [Command-Line Options](#command-line-options)
- [Output Formats](#output-formats)
- [Advanced Usage Examples](#advanced-usage-examples)
- [Environment Variables](#environment-variables)
- [Performance Tuning](#performance-tuning)
- [Troubleshooting](#troubleshooting)

## Installation

### From Source

```bash
git clone https://github.com/zmh-program/bytes-radar.git
cd bytes-radar
cargo build --release
```

### Using Cargo

```bash
cargo install bytes-radar
```

## Basic Usage

The simplest way to analyze a repository:

```bash
bradar user/repo
```

This analyzes the default branch of `user/repo` on GitHub and displays results in a human-readable table format.

## Supported Platforms

- **GitHub** (github.com, GitHub Enterprise)
- **GitLab** (gitlab.com, self-hosted instances)
- **Bitbucket** (bitbucket.org)
- **Codeberg** (codeberg.org)
- **SourceForge** (sourceforge.net)
- **Gitea** instances
- **Azure DevOps**
- **Direct archive URLs** (tar.gz, tgz, zip)

## URL Formats

| Format             | Description                        | Example                              |
| ------------------ | ---------------------------------- | ------------------------------------ |
| `user/repo`        | GitHub repository (default branch) | `microsoft/vscode`                   |
| `user/repo@branch` | Specific branch                    | `torvalds/linux@master`              |
| `user/repo@commit` | Specific commit hash               | `rust-lang/rust@abc123`              |
| Full URL           | Complete repository URL            | `https://github.com/user/repo`       |
| Archive URL        | Direct archive link                | `https://example.com/project.tar.gz` |

## Command-Line Options

### Basic Information

```bash
bradar --help          # Show help information
bradar -v               # Show version information
```

### Repository Analysis

```bash
bradar [OPTIONS] <URL>
```

### Output Options

| Option          | Short | Description                                        | Default |
| --------------- | ----- | -------------------------------------------------- | ------- |
| `--format`      | `-f`  | Output format (table, json, csv, xml, yaml, toml)  | `table` |
| `--detailed`    |       | Show detailed file-by-file statistics              | `false` |
| `--quiet`       | `-q`  | Quiet mode - suppress progress and minimize output | `false` |
| `--no-progress` |       | Disable progress bar                               | `false` |
| `--no-color`    |       | Disable colored output                             | `false` |

### Authentication

| Option    | Description                                   |
| --------- | --------------------------------------------- |
| `--token` | Authentication token for private repositories |

### Network Options

| Option              | Description                                  | Default |
| ------------------- | -------------------------------------------- | ------- |
| `--timeout`         | Request timeout in seconds                   | `300`   |
| `--allow-insecure`  | Allow insecure HTTPS connections             | `false` |
| `--user-agent`      | Custom User-Agent string                     |         |
| `--retry-count`     | Number of retry attempts for failed requests | `3`     |
| `--max-redirects`   | Maximum number of redirects to follow        | `10`    |
| `--use-compression` | Enable HTTP compression                      | `true`  |
| `--proxy`           | Proxy URL for all requests                   |         |

### Provider Configuration

| Option               | Description                          | Example                            |
| -------------------- | ------------------------------------ | ---------------------------------- |
| `--provider-config`  | Provider-specific configuration file | `github-config.json`               |
| `--provider-setting` | Set provider-specific setting        | `github.api_version=2022-11-28`    |
| `--header`           | Add custom HTTP header               | `Accept=application/vnd.github.v3` |
| `--credential`       | Set provider credential              | `token=ghp_xxx`                    |

### Filtering Options

| Option                | Description                                         | Default  |
| --------------------- | --------------------------------------------------- | -------- |
| `--aggressive-filter` | Enable aggressive filtering for maximum performance | `false`  |
| `--max-file-size`     | Maximum file size to process in KB                  | `102400` |
| `--min-file-size`     | Minimum file size to process in bytes               | `1`      |
| `--include-tests`     | Include test directories in analysis                | `false`  |
| `--include-docs`      | Include documentation directories in analysis       | `false`  |
| `--include-hidden`    | Include hidden files and directories                | `false`  |
| `--exclude-pattern`   | Exclude files matching this pattern (glob)          |          |
| `--include-pattern`   | Only include files matching this pattern (glob)     |          |

### Language Options

| Option               | Description                             |
| -------------------- | --------------------------------------- |
| `--language`         | Only analyze files of specific language |
| `--exclude-language` | Exclude specific language from analysis |

### Analysis Options

| Option                | Description                                     | Default |
| --------------------- | ----------------------------------------------- | ------- |
| `--ignore-whitespace` | Ignore whitespace-only lines in code analysis   | `false` |
| `--count-generated`   | Include generated files in analysis             | `false` |
| `--max-line-length`   | Maximum line length to consider (0 = unlimited) | `0`     |

### Debug and Logging

| Option       | Short | Description                |
| ------------ | ----- | -------------------------- |
| `--debug`    | `-d`  | Enable debug output        |
| `--trace`    |       | Enable trace-level logging |
| `--log-file` |       | Write logs to file         |

### Advanced Options

| Option           | Description                            | Default |
| ---------------- | -------------------------------------- | ------- |
| `--threads`      | Number of worker threads (0 = auto)    | `0`     |
| `--memory-limit` | Memory limit in MB (0 = unlimited)     | `0`     |
| `--cache-dir`    | Directory for caching downloaded files |         |
| `--no-cache`     | Disable caching of downloaded files    | `false` |

### Experimental Features

| Option                     | Description                             |
| -------------------------- | --------------------------------------- |
| `--experimental-parallel`  | Enable experimental parallel processing |
| `--experimental-streaming` | Enable experimental streaming analysis  |

## Output Formats

### Table (Default)

Human-readable table format with colored output and progress indicators.

```bash
bradar microsoft/vscode
```

### JSON

Machine-readable JSON format for integration with other tools.

```bash
bradar --format json microsoft/vscode
```

### CSV

Comma-separated values format for spreadsheet analysis.

```bash
bradar --format csv microsoft/vscode
```

### XML

XML format for structured data processing.

```bash
bradar --format xml microsoft/vscode
```

### YAML

YAML format for configuration files and human-readable structured data.

```bash
bradar --format yaml microsoft/vscode
```

### TOML

TOML format for configuration management.

```bash
bradar --format toml microsoft/vscode
```

## Advanced Usage Examples

### Analyzing Private Repositories

```bash
# Using token parameter
bradar --token ghp_xxxxxxxxxxxxxxxxxxxx private-org/private-repo

# Using environment variable
export BRADAR_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx
bradar private-org/private-repo
```

### Performance Optimization

```bash
# For large repositories - aggressive filtering
bradar --aggressive-filter --max-file-size 2048 torvalds/linux

# Custom thread count and memory limit
bradar --threads 8 --memory-limit 4096 large-org/huge-repo

# Enable experimental features for better performance
bradar --experimental-parallel --experimental-streaming big-repo
```

### Detailed Analysis with Filtering

```bash
# Include tests and documentation
bradar --include-tests --include-docs --detailed microsoft/typescript

# Analyze only specific language
bradar --language rust rust-lang/rust

# Exclude multiple languages
bradar --exclude-language javascript --exclude-language css web-project

# Custom file patterns
bradar --include-pattern "*.rs" --exclude-pattern "*test*" rust-project
```

### Output Customization

```bash
# Quiet JSON output for scripting
bradar --quiet --format json user/repo | jq '.global_metrics'

# Detailed CSV for analysis
bradar --format csv --detailed user/repo > analysis.csv

# Debug mode with log file
bradar --debug --log-file analysis.log --trace user/repo
```

### Provider-Specific Configuration

```bash
# GitHub with custom API version
bradar --provider-setting github.api_version=2022-11-28 user/repo

# GitLab with custom instance
bradar --provider-setting gitlab.instance=https://gitlab.company.com user/repo

# Custom headers for enterprise instances
bradar --header "Authorization=Bearer token" --header "Accept=application/json" user/repo

# Multiple provider credentials
bradar --credential "github.token=ghp_xxx" --credential "gitlab.token=glpat_xxx" user/repo
```

### Network Configuration

```bash
# Custom timeout and compression
bradar --timeout 600 --use-compression false slow-server/repo

# Using a proxy
bradar --proxy http://proxy.company.com:8080 user/repo

# Custom redirect handling
bradar --max-redirects 5 user/repo

# Enterprise setup with custom headers
bradar --header "X-Custom-Auth=token" --allow-insecure https://git.company.com/repo
```

## Environment Variables

| Variable       | Description                  | Example                    |
| -------------- | ---------------------------- | -------------------------- |
| `BRADAR_TOKEN` | Default authentication token | `ghp_xxxxxxxxxxxxxxxxxxxx` |

## Performance Tuning

### For Large Repositories

1. **Use aggressive filtering**: `--aggressive-filter`
2. **Reduce max file size**: `--max-file-size 512`
3. **Increase timeout**: `--timeout 600`
4. **Use experimental features**: `--experimental-parallel`

### For Many Small Files

1. **Increase thread count**: `--threads 16`
2. **Use streaming**: `--experimental-streaming`
3. **Set memory limit**: `--memory-limit 8192`

### For Network-Limited Environments

1. **Enable caching**: `--cache-dir ~/.bradar-cache`
2. **Increase retry count**: `--retry-count 10`
3. **Extend timeout**: `--timeout 900`

## Troubleshooting

### Common Issues

#### Authentication Errors

```bash
# Make sure token has correct permissions
bradar --token ghp_xxxxxxxxxxxxxxxxxxxx --debug private-repo
```

#### Network Timeouts

```bash
# Increase timeout and retry count
bradar --timeout 600 --retry-count 5 slow-repo
```

#### Memory Issues

```bash
# Set memory limit and use aggressive filtering
bradar --memory-limit 2048 --aggressive-filter large-repo
```

#### Slow Analysis

```bash
# Use performance optimizations
bradar --aggressive-filter --experimental-parallel --threads 8 repo
```

### Debug Mode

Enable debug mode for detailed information:

```bash
bradar --debug --trace --log-file debug.log problematic-repo
```

### Getting Help

```bash
bradar --help                    # General help
bradar --version                 # Version information
```

## Integration Examples

### Shell Scripts

```bash
#!/bin/bash
# Analyze multiple repositories and save to JSON
repos=("user/repo1" "user/repo2" "user/repo3")
for repo in "${repos[@]}"; do
    bradar --format json --quiet "$repo" > "${repo//\//_}.json"
done
```

### CI/CD Integration

```yaml
# GitHub Actions example
- name: Analyze Code Statistics
  run: |
    bradar --format json --quiet ${{ github.repository }} > stats.json
    cat stats.json | jq '.global_metrics.total_lines'
```

### Monitoring Scripts

```bash
#!/bin/bash
# Monitor repository growth
bradar --format json --quiet user/repo | \
jq -r '.global_metrics | "\(.total_files) files, \(.total_lines) lines"'
```
