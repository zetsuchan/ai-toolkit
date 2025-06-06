# AI Toolkit - Unix Philosophy for AI Verification

A collection of Rust-based command-line tools that extend Unix philosophy for AI model verification and analysis.

## Tools

### Core Unix-Style Tools (Enhanced for AI)
- **ai-wc** - Enhanced word count with AI-specific metrics
- **ai-diff** / **semdiff** - Intelligent diff for AI outputs and semantic comparisons  
- **ai-grep** - Pattern matching for AI content and anomaly detection
- **ai-sort** - Sorting utilities for AI datasets and outputs
- **ai-uniq** - Deduplication for AI-generated content
- **ai-cut** - Extract specific fields from AI data formats
- **ai-join** - Merge AI datasets and verification results

### Novel Verification Tools
- **aiprobe** - Automatically infer what needs verification
- **ghostwriter** - Generate tests from examples and learn patterns
- **explain-failure** - Human-readable verification failure explanations
- **reality-check** - Validate AI output against real-world constraints
- **fact-density** / **citation-check** / **confidence-language** - Content quality analysis
- **bullshit-score** - Detect low-quality or fabricated content
- **extract-claims** / **build-knowledge-graph** / **find-contradictions** - Logical consistency

### Meta-Verification & Pipeline Tools
- **verifychain** - Composable verification pipeline builder
- **crowdverify** - Distributed verification across experts
- **failure-modes** - Analyze potential failure patterns
- **devil-advocate** - Generate skeptical verification suggestions
- **verify-logger** / **auto-verify** - Learning verification system

### Specialized Analysis Tools
- **chunker** - Semantic boundary splitting for large texts
- **netprobe** / **ratelimit-check** - Network and API analysis
- **syntax-check** / **import-validator** - Code validation
- **api-endpoint-tester** / **rate-limit-analyzer** - API verification

### Advanced uutils-Based Tools
- **factdiff** - Semantic fact comparison using embeddings
- **ai-tee** - Verification splitter for parallel analysis
- **tokentop** - Real-time token analysis (like htop for AI generation)
- **aistream** - Stream processing with backpressure and checkpoints
- **verifyd** - Learning verification daemon

### The Killer App: aicc (AI Compiler)
- **aicc** - Compile natural language to verified code output
- Multi-candidate generation with confidence scoring
- Automated verification passes (syntax, security, imports, tests)
- Language-agnostic with Python, Rust, JavaScript support

### Low-Level Debugging Tools
- **strace-ai** - Trace AI decision making and reasoning paths
- **valgrind-ai** - Memory and safety verification for AI-generated code

## Building

```bash
cargo build --release
```

## Installation

```bash
cargo install --path .
```

## Philosophy

Following Unix philosophy: each tool does one thing well, tools work together via pipes, and everything is a text stream.