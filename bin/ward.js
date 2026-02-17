#!/usr/bin/env node

const { spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

// For local development, use cargo run
// In production, this would point to the downloaded binary
const args = process.argv.slice(2);
const command = "cargo";
const runArgs = ["run", "--quiet", "--", ...args];

const child = spawn(command, runArgs, {
  stdio: "inherit",
  shell: true,
});

child.on("exit", (code) => {
  process.exit(code);
});
