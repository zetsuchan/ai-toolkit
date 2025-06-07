# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust workspace containing AI verification tools built with Unix philosophy principles. Each tool does one thing well, works with standard I/O, and composes through pipes. The project includes 40+ specialized command-line utilities for AI model verification, analysis, and code generation.

## Build Commands

```bash
# Build all tools in the workspace
cargo build --release

# Build a specific tool
cargo build --release -p ai-wc
cargo build --release -p aicc

# Install all tools to PATH
cargo install --path tools/ai-wc
cargo install --path tools/aicc
# (repeat for other tools as needed)

# Run tests for the workspace
cargo test

# Run tests for a specific tool
cargo test -p ai-wc

# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy lints
cargo clippy
```

## Architecture

### Workspace Structure
- **Root Cargo.toml**: Defines workspace with 40+ member tools
- **tools/**: Individual command-line utilities, each with own Cargo.toml
- **docs/**: Architecture documentation and examples

### Tool Categories
1. **Core Unix-style tools**: ai-wc, ai-diff, ai-grep, ai-sort, ai-uniq, ai-cut, ai-join
2. **Verification tools**: aiprobe, reality-check, fact-density, citation-check, bullshit-score
3. **Meta-verification**: verifychain, crowdverify, auto-verify, verify-logger
4. **Pipeline tools**: aistream, ai-tee, chunker
5. **The flagship**: aicc (AI Compiler) - compiles natural language to verified code

### Common Patterns
- All tools use clap for CLI argument parsing
- Standard workspace dependencies: clap, serde, tokio, anyhow, thiserror, regex
- Tools read from stdin if no files specified
- Text-based I/O for composability via pipes
- Rust editions 2021, shared version 0.1.0

### Key Implementation Examples
- **aicc**: Multi-candidate generation with confidence scoring and verification passes (syntax, security, imports, tests)
- **ai-wc**: Enhanced word count with AI-specific metrics (hallucination markers, fact density, repetition scores)
- **verifychain**: Composable verification pipeline builder (currently placeholder implementation)

### Development Patterns
- Use workspace dependencies for consistency
- Each tool is self-contained with its own main.rs
- Follow Unix philosophy: simple, composable, text-based interfaces
- Tools should work both standalone and in pipelines