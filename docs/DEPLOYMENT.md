# Deployment Guide

## Cloudflare Workers

[![Deploy to Cloudflare Workers](https://deploy.workers.cloudflare.com/button.svg)](https://deploy.workers.cloudflare.com/?url=https://github.com/zmh-program/bytes-radar)

> [!TIP]
> The Free Tier of Cloudflare Workers has a **20s request timeout limit**. Analysis of large repositories may fail due to this limitation. Consider upgrading to Cloudflare Workers Pro or using alternative methods for processing large repositories.

1. Fork the repository to your GitHub account
2. Connect it to your Cloudflare Workers account
3. Deploy the worker to your chosen environment

### Manual Deployment (Wrangler)

If you prefer to deploy manually:

1. Clone the repository

```bash
git clone https://github.com/zmh-program/bytes-radar.git
cd bytes-radar/worker
```

2. Install Wrangler CLI:

```bash
pnpm install -g wrangler
```

3. Authenticate with Cloudflare:

```bash
wrangler login
```

4. Deploy to staging environment:

```bash
wrangler deploy --env staging
```

5. Deploy to production:

```bash
wrangler deploy --env production
```

### Environment Configuration

The worker supports two environments:

- `staging`: For testing and development (bytes-radar-staging.workers.dev)
- `production`: For production use (bytes-radar-prod.workers.dev)

See `worker/wrangler.toml` for environment-specific configurations.

### Worker Configuration Options

The worker supports extensive configuration through the `AnalysisOptions` object:

```typescript
interface AnalysisOptions {
  // HTTP Configuration
  timeout?: number; // Request timeout in seconds (default: 300)
  max_redirects?: number; // Maximum number of redirects (default: 10)
  user_agent?: string; // Custom User-Agent (default: "bytes-radar/1.0.0")
  accept_invalid_certs: boolean; // Accept invalid SSL certificates (default: false)
  headers: Record<string, string>; // Custom HTTP headers
  credentials: Record<string, string>; // Authentication credentials
  provider_settings: Record<string, string>; // Provider-specific settings
  max_file_size?: number; // Maximum file size in bytes (default: 100MB)
  use_compression: boolean; // Use HTTP compression (default: true)
  proxy?: string; // Proxy URL

  // Analysis Configuration
  ignore_hidden: boolean; // Ignore hidden files (default: true)
  ignore_gitignore: boolean; // Respect .gitignore rules (default: true)
  aggressive_filtering?: boolean; // Use aggressive file filtering
  custom_filter?: IntelligentFilter; // Custom file filtering rules
}
```

## API Documentation

The Bytes Radar API provides code analysis capabilities through a simple HTTP interface.

### Base URL

```
https://bradar.zmh.me
```

### Endpoints

#### Analyze Repository

```http
GET /{repository_path}
```

Analyzes a repository and returns detailed statistics about its codebase.

##### Repository Path Formats

- GitHub repository: `owner/repo` or `owner/repo@branch`
- Full GitHub URL: `https://github.com/owner/repo`
- GitLab URL: `https://gitlab.com/owner/repo`
- Direct archive URL: `https://example.com/archive.tar.gz`

##### Query Parameters

All parameters from the `AnalysisOptions` interface are supported as query parameters:

- `ignore_hidden` (boolean, default: true) - Whether to ignore hidden files/directories
- `ignore_gitignore` (boolean, default: true) - Whether to respect .gitignore rules
- `max_file_size` (number, default: 104857600) - Maximum file size to analyze in bytes
- `timeout` (number, default: 300) - Request timeout in seconds
- `max_redirects` (number, default: 10) - Maximum number of redirects to follow
- `user_agent` (string) - Custom User-Agent string
- `accept_invalid_certs` (boolean, default: false) - Accept invalid SSL certificates
- `use_compression` (boolean, default: true) - Use HTTP compression
- `proxy` (string) - Proxy URL to use
- `aggressive_filtering` (boolean) - Enable aggressive file filtering

Custom headers, credentials, and provider settings can be set using the following format:

- `header.{name}` - Set custom HTTP header
- `credential.{name}` - Set provider credential
- `provider.{name}` - Set provider-specific setting

##### Example Request

```http
GET /zmh-program/bytes-radar?max_file_size=1048576&timeout=60&header.Authorization=token%20xyz
```

##### Example Response

```json
{
  "project_name": "bytes-radar@main",
  "summary": {
    "project_name": "bytes-radar@main",
    "total_files": 37,
    "total_lines": 10255,
    "total_code_lines": 8944,
    "total_comment_lines": 0,
    "total_blank_lines": 1311,
    "total_size_bytes": 303101,
    "language_count": 8,
    "primary_language": "Rust",
    "overall_complexity_ratio": 0.872,
    "overall_documentation_ratio": 0
  },
  "language_statistics": [
    {
      "language_name": "Rust",
      "file_count": 22,
      "total_lines": 3892,
      "code_lines": 3298,
      "comment_lines": 0,
      "blank_lines": 594,
      "total_size_bytes": 123310,
      "average_file_size": 176.91,
      "complexity_ratio": 0.847,
      "documentation_ratio": 0
    }
  ],
  "debug_info": {
    "total_languages": 8,
    "total_files": 37
  }
}
```

##### Error Response

```json
{
  "error": "Error message",
  "error_type": "NetworkError | AnalysisError",
  "url": "Original request URL"
}
```
