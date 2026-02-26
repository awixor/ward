use crate::config::Config;
use regex::Regex;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
// use std::fs; // Removed unused import
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug)]
pub struct Violation {
    pub file: PathBuf,
    pub line: usize,
    pub rule: String,
    pub snippet: String,
}

pub struct Scanner {
    config: Config,
    patterns: Vec<(String, Regex)>,
    exclude_globnet: Gitignore,
}

fn shannon_entropy(s: &str) -> f32 {
    let mut map = HashMap::new();
    let len = s.len() as f32;

    for c in s.chars() {
        *map.entry(c).or_insert(0.0) += 1.0;
    }

    map.values().fold(0.0, |acc, &count| {
        let p = count / len;
        acc - p * p.log2()
    })
}

impl Scanner {
    pub fn new(config: Config) -> Self {
        let mut patterns = vec![
            ("Ethereum Private Key".to_string(), Regex::new(r"(?i)0x[a-fA-F0-9]{64}").unwrap()),
            ("BIP-39 Mnemonic".to_string(), Regex::new(r"(?i)(\b[a-z]{3,}\b\s){11,}\b[a-z]{3,}\b").unwrap()),
            ("Generic API Key".to_string(), Regex::new(r#"(?i)(api_key|access_token|secret_key)[\s:=]+['""]?[a-zA-Z0-9_\-]{20,}"#).unwrap()),
        ];
        
        // Add custom rules
        for rule in &config.rules {
            if let Ok(re) = Regex::new(&rule.regex) {
                patterns.push((rule.name.clone(), re));
            }
        }

        // Build Gitignore for excludes
        let mut builder = GitignoreBuilder::new("");
        for pattern in &config.exclude {
            let _ = builder.add_line(None, pattern);
        }
        let exclude_globnet = builder.build().unwrap_or_else(|_| Gitignore::empty());

        Self { config, patterns, exclude_globnet }
    }

    pub fn scan_content(&self, path: &Path, content: &str) -> Result<Vec<Violation>> {
        if self.exclude_globnet.matched_path_or_any_parents(path, false).is_ignore() {
            return Ok(vec![]);
        }

        let mut violations = vec![];

        // 0. File Name Check
        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            // Block .env and .env.* (e.g. .env.local, .env.production)
            // But allow .env.example, .env.sample (common safe patterns)
            if filename.starts_with(".env") {
                let is_safe_example = filename.ends_with(".example") || filename.ends_with(".sample");
                
                if !is_safe_example {
                     violations.push(Violation {
                        file: path.to_path_buf(),
                        line: 1,
                        rule: format!("Critical: {} detected", filename),
                        snippet: "Do not commit .env files. Use .env.example instead.".to_string(),
                    });
                    return Ok(violations);
                }
            }
        }

        for (i, line) in content.lines().enumerate() {
            let line_idx = i + 1;

            // 1. Regex Check
            for (name, re) in &self.patterns {
                if re.is_match(line) {
                     violations.push(Violation {
                        file: path.to_path_buf(),
                        line: line_idx,
                        rule: name.clone(),
                        snippet: line.trim().chars().take(50).collect(),
                    });
                    continue; // Detected, move to next line
                }
            }

            // 2. Entropy Check
            // Verify if we should skip this file for entropy
            let skip_entropy = self.config.skip_entropy_checks.iter().any(|pattern| {
                 // Simple contains check for MVP, glob matching TODO
                 path.to_string_lossy().contains(pattern.trim_start_matches('*')) 
            });

            if !skip_entropy {
               // Only check words/tokens > 20 chars
               for word in line.split_whitespace() {
                   // Clean common wrapping punctuation from the word
                   let clean_word = word.trim_matches(|c| "()[]{}\";',`".contains(c));
                   
                   if clean_word.len() > 20 {
                        // Skip if it looks like code function calls, paths, or template literals
                        if clean_word.contains("::") || 
                           clean_word.contains("->") || 
                           clean_word.contains("=>") || 
                           clean_word.contains("${") || 
                           clean_word.contains("</") {
                            continue;
                        }

                        let entropy = shannon_entropy(clean_word);
                        if entropy > self.config.threshold {
                             violations.push(Violation {
                                file: path.to_path_buf(),
                                line: line_idx,
                                rule: format!("High Entropy ({:.2})", entropy),
                                snippet: clean_word.chars().take(20).collect(), // truncated
                            });
                        }
                   }
               }
            }
        }

        Ok(violations)
    }
}
