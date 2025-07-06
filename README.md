# Bytes Radar

[![CI](https://github.com/zmh-program/bytes-radar/workflows/CI/badge.svg)](https://github.com/zmh-program/bytes-radar/actions)
[![Crates.io](https://img.shields.io/crates/v/bytes-radar.svg)](https://crates.io/crates/bytes-radar)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

A fast code analysis tool for remote repositories with multi-platform support.

## Features

- **Asynchronous Repository Processing**: Non-blocking HTTP client with async streaming request processing for efficient remote repository fetching and decompression, optimized for **low memory usage** and **serverless environments** (always `<32MiB` runtime memory usage for large files)
- **Multi-Platform URL Resolution**: Features intelligent URL parsing engine that normalizes different Git hosting platform APIs (GitHub, GitLab, Bitbucket, Codeberg) into unified archive endpoints with branch/commit resolution
- **Streaming Archive Analysis**: Processes tar.gz archives directly in memory using streaming decompression without temporary file extraction, reducing I/O overhead and memory footprint
- **Language Detection Engine**: Implements rule-based file extension and content analysis system supporting 150+ programming languages with configurable pattern matching and statistical computation (use tokei [languages map](https://github.com/XAMPPRocky/tokei/blob/master/languages.json))
- **Real-time Progress Monitoring**: Features bandwidth-aware progress tracking with download speed calculation, ETA estimation, and adaptive UI rendering for terminal environments
- **Structured Data Serialization**: Provides multiple output format engines (Table, JSON, CSV, XML) with schema validation and type-safe serialization for integration with external tools
- **Authentication Layer**: Implements OAuth token management with secure credential handling for accessing private repositories across different hosting platforms
- **Cross-Platform Binary Distribution**: Supports native compilation targets for Linux, macOS, and Windows with platform-specific optimizations and dependency management
- **WebAssembly Support**: Run bytes-radar directly in browsers or WASI environments with full feature parity

## Installation

### From Cargo (Recommended)

```bash
cargo install bytes-radar
```

### From npm (WASM)

```bash
npm install bytes-radar-wasm
```

### From Releases

Download the latest binary from [GitHub Releases](https://github.com/zmh-program/bytes-radar/releases)

### From Source

```bash
git clone https://github.com/zmh-program/bytes-radar.git
cd bytes-radar
cargo build --release

# For WASM build
wasm-pack build --target web --features wasm
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

## Usage Environments

### CLI

See the CLI Options section below for command-line usage.

### WebAssembly

bytes-radar can be used in both browser and WASI environments. For detailed WASM usage instructions and API reference, see [WASM Documentation](docs/wasm.md).

#### Browser Example

```javascript
import init, { analyze_repository } from 'bytes-radar';

async function main() {
    await init();
    const result = await analyze_repository('torvalds/linux');
    console.log(result);
}
```

#### WASI Example

```javascript
import { WASI } from '@wasmer/wasi';
const wasi = new WASI({
    args: ['bytes-radar', 'torvalds/linux']
});

const instance = await WebAssembly.instantiate(
    await WebAssembly.compile(
        await fs.readFile('bytes-radar.wasm')
    ),
    { wasi_snapshot_preview1: wasi.wasiImport }
);

wasi.start(instance);
```

## Output Formats

### Table (Default)
```shell
$ bytes-radar torvalds/linux
Analyzing: https://github.com/torvalds/linux
Analysis completed in 123.76s

================================================================================
 Project                                                  linux@main
 Total Files                                              89,639
 Total Lines                                              40,876,027
 Code Lines                                               32,848,710
 Comment Lines                                            2,877,885
 Blank Lines                                              5,149,432
 Languages                                                51
 Primary Language                                         C
 Code Ratio                                               80.4%
 Documentation                                            8.8%
================================================================================
 Language                Files        Lines     Code   Comments   Blanks   Share%
================================================================================
 C                      35,586   25,268,107 18,782,347  2,836,806 3,648,954    61.8%
 C Header               25,845   10,247,647 9,481,722          0  765,925    25.1%
 Device Tree             5,789    1,831,396 1,589,630          0  241,766     4.5%
 ReStructuredText        3,785      782,387  593,628          0  188,759     1.9%
 JSON                      961      572,657  572,655          0        2     1.4%
 Text                    5,100      566,733  499,590          0   67,143     1.4%
 YAML                    4,862      548,408  458,948          0   89,460     1.3%
 GNU Style Assembly      1,343      373,956  326,745          0   47,211     0.9%
 Shell                     960      189,965  155,974          0   33,991     0.5%
 Plain Text              1,298      128,205  105,235          0   22,970     0.3%
 Python                    293       89,285   69,449      5,770   14,066     0.2%
 Makefile                3,115       82,692   57,091     13,109   12,492     0.2%
 SVG                        82       53,409   53,316          0       93     0.1%
 Perl                       58       43,986   33,264      4,406    6,316     0.1%
 Rust                      158       39,561   19,032     16,697    3,832     0.1%
 XML                        24       22,193   20,971          0    1,222     0.1%
 PO File                     7        6,711    5,605          0    1,106     0.0%
 Happy                      10        6,078    5,352          0      726     0.0%
 Assembly                   11        5,361    4,427          0      934     0.0%
 Lex                        10        2,996    2,277        347      372     0.0%
 AWK                        12        2,611    1,777        487      347     0.0%
 C++                         7        2,267    1,932          0      335     0.0%
 Forge Config               15        1,352    1,065          0      287     0.0%
 Bazel                      78        1,303    1,097         21      185     0.0%
 Jinja2                    141        1,107      902        137       68     0.0%
 Unreal Script               5          672      574          0       98     0.0%
 ASN.1                      15          656      528          0      128     0.0%
 Markdown                    3          578      436          0      142     0.0%
 LD Script                  13          551      466          0       85     0.0%
 Autoconf                    6          449      387         29       33     0.0%
 Gherkin (Cucumber)          1          330      293          0       37     0.0%
 CSS                         3          295      241          0       54     0.0%
 SWIG                        1          252      181          0       71     0.0%
 TeX                         1          234      228          0        6     0.0%
 Alex                        2          222      180          0       42     0.0%
 XSL                        10          200      122         52       26     0.0%
 RPM Specfile                1          174      152          0       22     0.0%
 HEX                         2          173      173          0        0     0.0%
 Module-Definition           2          157      137          0       20     0.0%
 Snakemake                   4          143      114         15       14     0.0%
 Pacman's makepkg            1          131      102          0       29     0.0%
 C++ Header                  2          125      106          0       19     0.0%
 Objective-C                 1           89       72          0       17     0.0%
 TOML                        3           47       40          0        7     0.0%
 Vim Script                  1           42       39          0        3     0.0%
 HTML                        2           33       30          0        3     0.0%
 Automake                    3           31       23          3        5     0.0%
 Ruby                        1           29       25          0        4     0.0%
 Apache Velocity             1           15       15          0        0     0.0%
 INI                         2           13       11          0        2     0.0%
 Bitbake                     3           13        4          6        3     0.0%
================================================================================
 Total                  89,639   40,876,027 32,848,710  2,877,885 5,149,432   100.0%
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
