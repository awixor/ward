use crate::config::Config;
use regex::Regex;
use globset::{Glob, GlobSet, GlobSetBuilder};
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
    exclude_globnet: GlobSet,
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

        // Build GlobSet for excludes
        let mut builder = GlobSetBuilder::new();
        for pattern in &config.exclude {
            if let Ok(glob) = Glob::new(pattern) {
                builder.add(glob);
            }
        }
        let exclude_globnet = builder.build().unwrap_or_else(|_| GlobSet::empty());

        Self { config, patterns, exclude_globnet }
    }

    pub fn scan_content(&self, path: &Path, content: &str) -> Result<Vec<Violation>> {
        if self.exclude_globnet.is_match(path) {
            return Ok(vec![]);
        }

        let mut violations = vec![];

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
                   let clean_word = word.trim_matches(|c| "()[]{}\";',".contains(c));
                   
                   if clean_word.len() > 20 {
                        // Skip if it looks like code function calls or paths
                        if clean_word.contains("::") || clean_word.contains("->") {
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
