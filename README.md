# bytes-radar 📊

[![CI](https://github.com/zmh-program/bytes-radar/workflows/CI/badge.svg)](https://github.com/zmh-program/bytes-radar/actions)
[![Crates.io](https://img.shields.io/crates/v/bytes-radar.svg)](https://crates.io/crates/bytes-radar)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

A hyper-fast code analysis tool for remote repositories with professional output and multi-platform support.

## ✨ Features

- **🚀 Lightning Fast**: Blazing fast analysis of remote repositories
- **🌐 Multi-Platform Support**: GitHub, GitLab, Bitbucket, Codeberg, and more
- **📊 Multiple Output Formats**: Table, JSON, CSV, and XML
- **🔧 Professional CLI**: Clean, tokei-inspired interface
- **📈 Progress Tracking**: Smart progress bars with download speed
- **🔒 Private Repositories**: Token-based authentication support
- **📱 Cross-Platform**: Linux, macOS, and Windows support

## 🛠️ Installation

### From Releases (Recommended)

Download the latest binary from [GitHub Releases](https://github.com/zmh-program/bytes-radar/releases):

```bash
# Linux/macOS
curl -L https://github.com/zmh-program/bytes-radar/releases/latest/download/bytes-radar-linux-x86_64 -o bytes-radar
chmod +x bytes-radar

# Windows
# Download bytes-radar-windows-x86_64.exe from releases
```

### From Cargo

```bash
cargo install bytes-radar
```

### From Source

```bash
git clone https://github.com/zmh-program/bytes-radar.git
cd bytes-radar
cargo build --release
```

## 🚀 Quick Start

```bash
# Analyze a GitHub repository
bytes-radar torvalds/linux

# Analyze with specific branch
bytes-radar microsoft/vscode@main

# Analyze with commit hash
bytes-radar rust-lang/rust@abc1234

# Multiple platforms
bytes-radar https://gitlab.com/user/repo
bytes-radar https://bitbucket.org/user/repo
bytes-radar https://codeberg.org/user/repo

# JSON output
bytes-radar -f json torvalds/linux

# With authentication token
bytes-radar --token ghp_xxx private/repo

# Quiet mode (minimal output)
bytes-radar --quiet user/repo
```

## 📊 Output Formats

### Table (Default)
```
================================================================================
 Project                                                  linux@main
 Total Files                                              75,823
 Total Lines                                              28,691,744
 Code Lines                                               22,453,891
 Comment Lines                                            3,891,234
 Blank Lines                                              2,346,619
 Languages                                                42
 Primary Language                                         C
 Code Ratio                                               78.3%
 Documentation                                            13.6%
================================================================================
 Language             Files        Lines     Code     Comments    Blanks   Share%
================================================================================
 C                   14,523   18,234,567   15,234   1,234,567   1,765,766    63.6%
 Assembly             2,341    3,456,789    2,891      234,567     321,331    12.0%
 ...
```

### JSON Output
```json
{
  "project_name": "linux@main",
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

## 🌐 Supported Platforms

| Platform | URL Format | Example |
|----------|------------|---------|
| **GitHub** | `user/repo` or full URL | `torvalds/linux` |
| **GitLab** | Full URL | `https://gitlab.com/user/repo` |
| **Bitbucket** | Full URL | `https://bitbucket.org/user/repo` |
| **Codeberg** | Full URL | `https://codeberg.org/user/repo` |
| **Direct** | tar.gz URL | `https://example.com/file.tar.gz` |

## 🔧 CLI Options

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

## 🏗️ Architecture

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

## 🤝 Contributing

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
cargo clippy --all-targets --all-features -- -D warnings
```

## 📈 Performance

bytes-radar is designed for speed:

- **Streaming Downloads**: Memory-efficient streaming with progress tracking
- **Parallel Processing**: Multi-threaded file analysis
- **Smart Caching**: Efficient dependency caching
- **Optimized Parsing**: Fast language detection and line counting

## 🐛 Bug Reports

Found a bug? Please report it on our [issue tracker](https://github.com/zmh-program/bytes-radar/issues) with:

- Command used
- Expected vs actual output
- Operating system and version
- bytes-radar version

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by [tokei](https://github.com/XAMPPRocky/tokei) for clean CLI design
- Built with ❤️ using [Rust](https://www.rust-lang.org/)

## 🔗 Links

- [Crates.io](https://crates.io/crates/bytes-radar)
- [Documentation](https://docs.rs/bytes-radar)
- [Changelog](CHANGELOG.md)
- [Contributing Guidelines](CONTRIBUTING.md)

---

**Made with ❤️ by [zmh-program](https://github.com/zmh-program)** 