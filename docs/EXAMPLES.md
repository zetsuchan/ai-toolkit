# Usage Examples

## Enhanced Core Tools

### ai-wc - Advanced Word Count
```bash
# Basic usage with AI metrics
$ echo "I think this might be correct, probably" | ai-wc --ai-metrics
Lines: 1
Words: 7
Chars: 39
AI Confidence markers: 3 ("likely", "probably", "might")
Hallucination indicators: 0
Fact density: 0.00 facts/paragraph
Repetition score: 0.00 (low)

# Check for AI hallucination patterns
$ ai-wc --hallucination-markers ai_output.txt
```

### ai-grep - Semantic Search
```bash
# Find contradictions in text
$ ai-grep --contradictions "secure" document.txt
Line 42: "The system is completely secure"
Line 97: "Several vulnerabilities exist"
  → Semantic contradiction detected (0.89 similarity)
  → Conflicts with line 97

# Fact-check claims
$ ai-grep --fact-check "statistics" report.md
Line 23: "90% of users prefer..."
  ⚠️ No source provided
  🔍 Similar claim found: Reuters reports 67% (2024)

# Find AI hallucination markers
$ ai-grep --hallucinations ai_response.txt
Line 15: "As an AI, I cannot access real-time data"
  🚨 Hallucination marker: capability_disclaimer
```

## Advanced Tools

### aicc - AI Compiler
```bash
# Compile natural language to verified Python
$ echo "create a secure password generator" | aicc -l python -O2 --explain
Parsing prompt... done
Generated 3 candidates
Verification pass 1: Syntax ✓
Verification pass 2: Security audit ✓  
Verification pass 3: Import analysis ✓
Verification pass 4: Test generation ✓
Output written to: password_gen.py (confidence: 0.94)

=== Verification Explanation ===
✓ Syntax check: Code parses correctly
✓ Security audit: Uses secrets module (cryptographically secure)
✓ Import analysis: All imports available and safe
✓ Test generation: Tests generated successfully
✓ Style compliance: Has proper docstrings

# Generate multiple candidates
$ aicc "fibonacci in Rust" --candidates 5 --confidence-threshold 0.9
```

### tokentop - Real-time Token Analysis
```bash
# Monitor AI generation in real-time
$ ai_generate --stream | tokentop --patterns
┌─ Token Statistics ──────────────────────┐
│ Tokens/sec: 42.3                        │
│ Perplexity: 12.4 (rising) ⚠️           │
│ Repetition: ████░░░░░░ 42%             │
│ Confidence: ██████░░░░ 65%             │
│                                         │
│ Live patterns detected:                 │
│ - Listing pattern (3rd time)            │
│ - Uncertainty language increasing       │
└─────────────────────────────────────────┘

# Raw token output
$ echo "test input" | tokentop --raw
test
input
```

### factdiff - Semantic Fact Comparison
```bash
# Compare facts between AI outputs
$ factdiff ai_output_v1.txt ai_output_v2.txt
Facts removed:
- Python was created in 1991 ✓
Facts added:
- Python was created in 1989 ❌ (actually 1991)
Facts modified:
- "Guido van Rossum" → "Guide van Rossum" ❌ (typo introduced)
```

## Pipeline Examples

### The "Reality Check" Pipeline
```bash
# Comprehensive validation pipeline
$ ai_generate "user demographics data" | \
  reality-check --population-db | \
  reality-check --zip-codes | \
  reality-check --phone-formats | \
  ai-validate
```

### The "Bullshit Detector" Pipeline
```bash
# Multi-stage content quality analysis
$ ai_output.txt | \
  fact-density | \
  citation-check | \
  confidence-language | \
  bullshit-score --threshold 0.7
Fact density: 0.12 facts/paragraph (low)
Citations: 0/15 claims sourced (poor)
Confidence language: 67% hedging detected (high)
Bullshit score: 0.78 (above threshold) ⚠️
```

### The "Consistency Detective" Pipeline
```bash
# Find logical contradictions
$ ai_generate "technical documentation" | \
  extract-claims | \
  build-knowledge-graph | \
  find-contradictions --visualize
```

### AI Compiler Pipeline
```bash
# Full compilation with verification
$ echo "create a web scraper that respects robots.txt" | \
  aicc -l python -O2 --generate-tests | \
  syntax-check python | \
  import-validator | \
  netprobe --domains | \
  ratelimit-check
```

## Verification Chain Examples

### Custom Verification Pipeline
```bash
# Build reusable verification chain
$ verifychain create "python-api-checker"
> add step: syntax-check python
> add step: import-validator  
> add step: api-endpoint-tester
> add step: rate-limit-analyzer
> save

$ ai_generate "FastAPI application" | verifychain run python-api-checker
```

### Learning Verification
```bash
# Train the system on your verification patterns
$ verify-logger start
$ # ... manually verify and fix AI output ...
$ git diff output.py | verify-logger learn --tag "api-security"

# Later, automatically apply learned patterns
$ ai_generate "REST API" | auto-verify
Applying learned verifications:
  ✓ API security patterns (from 2024-03-15)
  ✓ Error handling (from 2024-03-20)
  ⚠ New pattern detected: Uses library not in your history
```

## Real-time Monitoring

### Stream Processing
```bash
# Process AI output with backpressure
$ ai_generate --stream | aistream \
  --buffer-size 1024 \
  --checkpoint every:paragraph \
  --verify-parallel 4 \
  --rollback-on-error
```

### Distributed Verification
```bash
# Split verification across experts
$ ai_output.txt | \
  chunker --semantic-boundaries | \
  crowdverify --parallel \
    --expert python:alice \
    --expert security:bob \
    --expert database:charlie
```

## Low-Level Debugging

### AI Decision Tracing
```bash
# Trace AI reasoning process
$ strace-ai ai_generate "fibonacci in Rust"
[EMBED] "fibonacci in Rust" → [0.23, -0.45, ...]
[LOOKUP] Top 5 similar training examples
[WEIGH] Pattern "recursive" (0.8) vs "iterative" (0.6)
[CHECK] Rust borrow checker constraints
[GEN] Token "fn" (confidence: 0.99)
[ABORT] Detected potential infinite recursion
[RETRY] With memoization pattern
```

### Memory Verification
```bash
# Check AI-generated C++ for memory issues
$ ai_generate "image processor in C++" | valgrind-ai
Analyzing AI-generated allocations...
⚠️ Line 34: new[] without corresponding delete[]
⚠️ Line 67: Potential buffer overflow (unchecked input)
✓ Stack usage: 2.3KB (acceptable)
```