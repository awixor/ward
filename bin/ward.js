#!/usr/bin/env node

const { spawn } = require("child_process");
const path = require("path");

// Point to the package's Cargo.toml so we can run from anywhere
const projectRoot = path.join(__dirname, "..");
const manifestPath = path.join(projectRoot, "Cargo.toml");

const args = process.argv.slice(2);
const command = "cargo";
// Use --release for production speed
const runArgs = [
  "run",
  "--release",
  "--quiet",
  "--manifest-path",
  manifestPath,
  "--",
  ...args,
];

const child = spawn(command, runArgs, {
  stdio: "inherit",
});

child.on("exit", (code) => {
  process.exit(code);
});
