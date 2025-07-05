pub fn expand_url(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }

    if url.contains('/') && !url.contains('.') {
        let parts: Vec<&str> = url.split('@').collect();
        let repo_part = parts[0];
        let branch_or_commit = parts.get(1);

        if let Some(branch) = branch_or_commit {
            if branch.len() >= 7 && branch.chars().all(|c| c.is_ascii_hexdigit()) {
                return format!("https://github.com/{}/commit/{}", repo_part, branch);
            } else {
                return format!("https://github.com/{}/tree/{}", repo_part, branch);
            }
        } else {
            return format!("https://github.com/{}", repo_part);
        }
    }

    url.to_string()
}

pub fn show_usage_examples() {
    println!("Error: URL argument is required");
    println!();
    println!("Usage: bytes-radar <URL>");
    println!();
    println!("Examples:");
    println!("  # GitHub repositories");
    println!("  bytes-radar user/repo                    # Default branch");
    println!("  bytes-radar user/repo@master             # Specific branch");
    println!("  bytes-radar user/repo@abc123             # Specific commit");
    println!("  bytes-radar https://github.com/user/repo # Full GitHub URL");
    println!();
    println!("  # Other platforms");
    println!("  bytes-radar https://gitlab.com/user/repo # GitLab");
    println!("  bytes-radar https://bitbucket.org/user/repo # Bitbucket");
    println!("  bytes-radar https://codeberg.org/user/repo # Codeberg");
    println!();
    println!("  # Output formats");
    println!("  bytes-radar -f json user/repo            # JSON output");
    println!("  bytes-radar -f csv user/repo             # CSV output");
    println!("  bytes-radar -f xml user/repo             # XML output");
    println!();
    println!("  # Authentication & options");
    println!("  bytes-radar --token ghp_xxx user/repo    # GitHub token");
    println!("  bytes-radar --timeout 600 user/repo      # Custom timeout");
    println!("  bytes-radar --quiet user/repo            # Minimal output");
    println!();
    println!("For more information try --help");
}
