use anyhow::{Result, Context};
use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use colored::*;

const HOOK_SCRIPT: &str = r#"
# Ward - Local-First Git Guard
# This hook was automatically installed by Ward.
if command -v ward >/dev/null 2>&1; then
    ward scan
else
    # Fallback to local npx if global command not found
    if [ -f "node_modules/.bin/ward" ]; then
        ./node_modules/.bin/ward scan
    # Check if npx is available and try running via npx
    elif command -v npx >/dev/null 2>&1; then
        npx -y git-ward scan
    else
        echo "Ward not found in path or node_modules. Skipping scan."
    fi
fi
"#;

pub fn install_hook() -> Result<()> {
    // 1. Check if .git exists
    let git_dir = Path::new(".git");
    if !git_dir.exists() {
        println!("{}", "Error: Not a git repository. Run 'git init' first.".red());
        return Ok(());
    }

    let hooks_dir = git_dir.join("hooks");
    if !hooks_dir.exists() {
        fs::create_dir(&hooks_dir).context("Failed to create hooks directory")?;
    }

    let pre_commit_path = hooks_dir.join("pre-commit");
    
    // 2. Read existing hook or create new
    let mut hook_content = if pre_commit_path.exists() {
        fs::read_to_string(&pre_commit_path).context("Failed to read existing pre-commit hook")?
    } else {
        String::from("#!/bin/sh\n")
    };

    // 3. Simple prepend logic (naive for MVP, improves robust parsing later)
    if !hook_content.contains("ward scan") {
        // Insert after shebang if present
        if hook_content.starts_with("#!") {
            match hook_content.find('\n') {
                Some(idx) => {
                    hook_content.insert_str(idx + 1, HOOK_SCRIPT);
                }
                None => {
                    hook_content.push_str(HOOK_SCRIPT);
                }
            }
        } else {
             // No shebang? Prepend one + script
            hook_content = format!("#!/bin/sh\n{}{}", HOOK_SCRIPT, hook_content);
        }
    
        fs::write(&pre_commit_path, &hook_content).context("Failed to write pre-commit hook")?;
        
        // Make executable
        let mut perms = fs::metadata(&pre_commit_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&pre_commit_path, perms)?;

        println!("{}", "✓ Ward pre-commit hook installed successfully.".green());
    } else {
        println!("{}", "ℹ Ward hook already exists.".yellow());
    }

    Ok(())
}

pub fn get_staged_files() -> Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .args(&["diff", "--cached", "--name-only", "--diff-filter=ACM"])
        .output()
        .context("Failed to execute git diff")?;

    if !output.status.success() {
        // Handle case where no commit yet?
        return Ok(vec![]);
    }

    let output_str = String::from_utf8(output.stdout)?;
    let files = output_str
        .lines()
        .map(|s| PathBuf::from(s.trim()))
        .filter(|p| p.exists()) // Verify file exists on disk
        .collect();

    Ok(files)
}

pub fn get_staged_content(path: &Path) -> Result<String> {
    // Use git show :path/to/file to get content from index
    // Note: path must be relative to repo root, which get_staged_files returns
    let path_str = path.to_str().context("Invalid path encoding")?;
    
    let output = Command::new("git")
        .args(&["show", &format!(":{}", path_str)])
        .output()
        .context(format!("Failed to read staged content for {}", path_str))?;

    if !output.status.success() {
        // If file is deleted in index or error, we might get here. 
        // For pre-commit diff-filter=ACM, it should exist.
        // Fallback or error? Let's error for now to be safe.
        return Err(anyhow::anyhow!("git show failed for {}", path_str));
    }

    let content = String::from_utf8(output.stdout)
        .context("File content is not valid UTF-8 (binary?)")?;
        
    Ok(content)
}
