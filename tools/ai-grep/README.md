# ai-grep

Anomaly detection tool for AI outputs with preset patterns for common AI failure modes.

## Overview

`ai-grep` extends traditional grep functionality with AI-specific pattern detection. It includes preset patterns for hallucinations, code issues, security problems, data leakage, and low confidence indicators commonly found in AI outputs.

## Usage

```bash
# Search with preset patterns
ai-grep --preset hallucinations ai_output.txt
ai-grep --preset code-issues generated_code.py
ai-grep --preset security ai_response.txt

# Traditional grep functionality  
ai-grep "pattern" file.txt

# Multiple files with line numbers
ai-grep -n "error" *.txt

# Case insensitive search with counts
ai-grep -i -c "warning" logs/

# JSON output for pipelines
ai-grep --preset security --format json ai_output.txt
```

## Preset Patterns

### Hallucinations (`--preset hallucinations`)
- AI self-references: "As an AI", "I cannot", "I don't have access"
- Knowledge cutoff indicators: "September 2021", "as of my last update"
- Uncertainty markers: "I think", "I believe", "might be"
- Training data references: "in my training", "I was taught"

### Code Issues (`--preset code-issues`) 
- Development markers: TODO, FIXME, HACK, BUG, XXX
- Placeholder code: "implementation here", "add code"
- Error indicators: syntax errors, undefined references
- Security anti-patterns: hardcoded secrets, SQL injection risks

### Security (`--preset security`)
- Credential exposure: API keys, passwords, tokens
- Injection vulnerabilities: SQL, command, XSS patterns
- Path traversal: "../", directory access attempts
- Unsafe operations: eval(), system calls with user input

### Data Leakage (`--preset data-leakage`)
- Personal information: emails, phone numbers, SSNs
- Internal references: server names, IP addresses, internal URLs
- Sensitive data patterns: credit cards, license plates
- Training data memorization: exact text reproductions

### Low Confidence (`--preset low-confidence`)
- Hedging language: "possibly", "might", "could be"
- Uncertainty expressions: "I'm not sure", "I think"
- Qualification words: "probably", "likely", "perhaps"
- Disclaimer patterns: "but I could be wrong"

## Severity Levels

- 🟢 **Low**: Minor issues, informational
- 🟡 **Medium**: Potential problems requiring attention  
- 🟠 **High**: Significant issues needing immediate review
- 🔴 **Critical**: Severe problems requiring urgent action

## Output Formats

### Text Format
```
file.txt:42: 🟠 HIGH [hallucinations] As an AI language model, I cannot
file.txt:156: 🔴 CRITICAL [security] password = "admin123"
```

### JSON Format
```json
{
  "matches": [
    {
      "file": "file.txt",
      "line": 42,
      "pattern": "As an AI language model",
      "severity": "High",
      "category": "hallucinations"
    }
  ]
}
```

## Grep Compatibility

Supports standard grep flags:
- `-i`: Case insensitive
- `-n`: Show line numbers  
- `-c`: Count matches
- `-v`: Invert match
- `-l`: List files with matches
- `-E`: Extended regex
- `-P`: Perl regex

## Installation

```bash
cargo build --release -p ai-grep
cargo install --path tools/ai-grep
```

## Examples

### Quality Assurance Pipeline
```bash
# Check AI code generation for issues
ai_generate_code prompt.txt | ai-grep --preset code-issues --format json
```

### Security Audit
```bash  
# Scan AI responses for security problems
ai-grep --preset security --preset data-leakage ai_responses/*.txt
```

### Hallucination Detection
```bash
# Flag responses with AI self-references
ai-grep --preset hallucinations -c chatbot_logs/
```

### Custom Pattern Matching
```bash
# Traditional grep usage with AI context
ai-grep -E "(error|fail|exception)" ai_debug.log
ai-grep -i "rate.?limit" api_responses.txt
```

### Pipeline Integration
```bash
# Count high-severity issues across multiple categories
ai-grep --preset security --preset hallucinations --format json *.txt | \
  jq '[.matches[] | select(.severity == "High" or .severity == "Critical")] | length'
```