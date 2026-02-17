use clap::{Parser, Subcommand};
use colored::*;
use anyhow::Result;
use std::process::exit;

mod config;
mod git;
mod scanner;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Ward in the current repository
    Init,
    /// Scan staged files for secrets
    Scan,
    /// Check for updates (stub)
    CheckUpdates,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            println!("{}", "Initializing Ward...".blue());
            git::install_hook()?;
        }
        Commands::Scan => {
             // 1. Load Config
            let config = config::load_config().unwrap_or_else(|e| {
                println!("{}", format!("Warning: Failed to load config: {}", e).yellow());
                config::Config::default()
            });

            // 2. Get Staged Files
            let files = git::get_staged_files()?;
            if files.is_empty() {
                return Ok(());
            }

            // 3. Scan
            let scanner = scanner::Scanner::new(config);
            let mut all_violations = vec![];

            for file in files {
                match scanner.scan_file(&file) {
                    Ok(mut v) => all_violations.append(&mut v),
                    Err(e) => eprintln!("Error scanning {:?}: {}", file, e),
                }
            }

            // 4. Report
            if !all_violations.is_empty() {
                println!("\n{}", "Ward detected sensitive data in your commit:".red().bold());
                for v in all_violations {
                    let masked_snippet = if v.snippet.len() <= 8 {
                        "[REDACTED]".to_string()
                    } else {
                        format!("{}...{}", &v.snippet[..4], &v.snippet[v.snippet.len()-4..])
                    };

                    println!(
                        "  {} {}:{}: {}", 
                        "✖".red(), 
                        v.file.display(), 
                        v.line.to_string().cyan(), 
                        v.rule.yellow()
                    );
                    println!("    Code: {}", masked_snippet.dimmed());
                }
                println!("\n{}", "Commit blocked. Remove the secrets or use 'git commit --no-verify' to bypass.".red());
                exit(1);
            } else {
                 println!("{}", "✓ Ward scan clean".green());
            }
        }
        Commands::CheckUpdates => {
            println!("You are running the latest version of Ward.");
        }
    }

    Ok(())
}
