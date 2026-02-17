# Project: Ward (Local-First Git Guard)

## 1. Overview & Goals

- **Mission:** Build a high-performance, **zero-server** CLI tool and Git hook to prevent developers from accidentally pushing sensitive data (Private Keys, Mnemonics, API Keys) to GitHub.
- **Privacy First:** All scanning happens in memory on the local machine. No data is ever sent to a cloud or external API.
- **Zero Friction:** Integration must be seamless and fast (target scan time: < 100ms) to ensure it doesn't interrupt the developer's flow.
- **Target Users:** Web3 developers (Foundry/Hardhat), AI engineers (OpenAI/Anthropic keys), and General DevOps.

## 2. Core Features (MVP)

- **Automatic Hook Installation:**
  - Command: `ward init` (distributed via `npx` wrapper).
  - **Behavior:**
    - Start by checking for an existing `.git/hooks/pre-commit`.
    - If none exists, create one with the Ward execution command.
    - If one exists, **prepend** the Ward command to the top (fail-fast execution).
    - Ensure existing scripts are preserved and run only if Ward passes.
- **Staged Files Only:** Uses `git diff --cached` to scan only the changes about to be committed.
- **Web3 Pattern Matching:** Specialized regex for:
  - Ethereum private keys (64-char hex)
  - BIP-39 mnemonics (12/24 words)
  - Solana keys (Base58)
- **The ".env" Guard:**
  - Hard-block if a `.env` file is staged.
  - Warn if a `.env` exists in the directory but is **not** in `.gitignore`.
- **Entropy-Based Detection:**
  - Identifies high-entropy strings that look like secrets but don't match specific regexes.
  - **Skip Option:** Configurable `skip_entropy_checks` in `ward.toml` for files like `*.lock`, `*.min.js` to avoid false positives.
- **Ignore System:** Support for `.wardignore` file to exclude specific files/patterns from scanning (standard glob patterns).
- **Binary Handling:** Relies on Git's native `is_binary` detection to skip binary files.
- **Manual Bypass:** Support standard `git commit --no-verify`.

## 3. Nice-to-have (Post-MVP)

- **Pre-push Guard:** A final safety check before data leaves the local machine.
- **Update Checker:**
  - **Explicit Command:** `ward check-updates` (no background checks or network calls during scan).
- **IDE Extension:** Real-time feedback in VS Code/Antigravity IDE.

## 4. Tech Stack & Constraints

- **Language:** **Rust** (execution speed, safety, zero-dependency binaries).
- **Distribution:**
  - **NPM Wrapper:** Primary distribution method (package name TBD, e.g., `@ward/cli` or similar).
  - **Installation:** Manual `npx <package> init` required (no `postinstall` script).
- **Git Integration:** Shell scripts calling the Rust binary.
- **Pattern Matching:** `regex` crate + `entropy` calculation logic.
- **Binary Size:** Optimized < 5MB.
- **Compatibility:** Linux, macOS, Windows (WSL).

## 5. Configuration

### Global vs. Local

- **Local Only:** Configuration lives strictly in the repo root (`ward.toml`).

### `ward.toml` Structure

```toml
# Example Configuration
exclude = ["*.lock", "public/images/", "target/"]
skip_entropy_checks = ["*.min.js", "package-lock.json"]
threshold = 3.8

[[rules]]
name = "Company Internal ID"
regex = "^ID-[0-9]{10}$"
```

## 6. UI/UX Guidelines (CLI)

- **Alerts:**
  - **RED:** Blocked commit (Secrets found).
  - **YELLOW:** Warnings (Configuration issues, .env not ignored).
  - **GREEN:** Clean commit.
- **Clarity:**
  - File path and Line number.
  - Rule triggered (e.g., "Detected: Ethereum Private Key").
  - Code snippet (masked for safety).
- **CI/CD Mode:**
  - Flag: `-s` (Silent/Strict).
  - Output: Standard stdout/stderr logs are sufficient (no JSON/SARIF for MVP).
  - Exit Code: 1 on failure.

## 7. Non-functional Requirements

- **Performance:** Scanning a standard commit < 50ms.
- **Mass-Add Handling:** **Fail Closed.** If too many files are staged (e.g., accidental `git add .` of `node_modules`), abort the commit to prevent hanging.
- **Updates:** No automatic network calls.
- **Regex Safety:** User-defined regexes are trusted without validation (for MVP).

## 8. Success Criteria / Verification

- [ ] Attempting to commit a 64-character hex string in a `.sol` file triggers a block.
- [ ] `ward init` correctly prepends hook to an existing `.git/hooks/pre-commit` file.
- [ ] Large binary files are skipped without error.
- [ ] Minified JS files are skipped by entropy check if configured.
- [ ] `git commit --no-verify` successfully bypasses the check.
