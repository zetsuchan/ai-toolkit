# ai-uniq

Statistical verification and deduplication tool for AI outputs with advanced repetition detection.

## Overview

`ai-uniq` extends traditional uniq functionality with AI-specific statistical analysis. It detects repetition patterns, calculates entropy, identifies AI loops, and provides comprehensive statistical verification of AI outputs.

## Usage

```bash
# Traditional uniq functionality
ai-uniq file.txt
ai-uniq -c file.txt  # Show counts

# AI-specific analysis modes
ai-uniq --word-analysis file.txt          # Word frequency analysis  
ai-uniq --phrase-analysis file.txt        # N-gram phrase analysis
ai-uniq --detect-loops file.txt           # AI loop detection
ai-uniq --stats file.txt                  # Statistical analysis

# Pipeline usage (equivalent to: tr ' ' '\n' | sort | uniq -c | sort -rn)
ai_generate prompt.txt | ai-uniq --word-analysis --sort-freq

# Flag high repetition patterns
ai-uniq --word-analysis --above-threshold --repetition-threshold 10 file.txt
```

## Features

### Traditional Uniq Compatibility
- All standard uniq flags: `-c`, `-d`, `-u`, `-i`, `-f`, `-s`, `-w`
- Line-based deduplication and counting
- Field and character-based comparison options

### Word Frequency Analysis (`--word-analysis`)
- Splits text into words and counts occurrences
- Identifies words appearing above threshold (default: 5 times)
- Equivalent to `tr ' ' '\n' | sort | uniq -c | sort -rn`
- Flags suspicious word repetition patterns

### Phrase Analysis (`--phrase-analysis`)
- N-gram analysis (default: 3-grams)
- Detects repeated phrases and expressions
- Configurable phrase length with `--ngram-size`
- Identifies memorized text patterns

### AI Loop Detection (`--detect-loops`)
- **Exact Repeats**: Identical line repetitions
- **Word Loops**: Excessive single word repetition  
- **Phrase Loops**: Repeated multi-word expressions
- **Pattern Loops**: Common AI transition phrases

### Statistical Analysis (`--stats`)
- Shannon entropy calculation
- Repetition ratio analysis
- Overall risk assessment
- Comprehensive statistical metrics

## AI Loop Patterns

The tool detects these common AI failure modes:

- **Similarity Loops**: "the same", "similar", "likewise"
- **Reference Loops**: "as mentioned", "as stated", "as discussed"  
- **Importance Loops**: "it is important", "this is important"
- **Conclusion Loops**: "in conclusion", "to summarize"
- **Transition Loops**: "however", "nevertheless", "on the other hand"

## Severity Levels

- ℹ️ **Low** (3+ occurrences): Minor repetition
- ⚠️ **Medium** (5+ occurrences): Moderate concern
- 🚨 **High** (10+ occurrences): Significant problem
- 💀 **Critical** (20+ occurrences): Severe AI malfunction

## Output Formats

### Text Format
```
=== Word Frequency Analysis ===
      15 the
      12 and
       8 to

🚨 FLAGGED: Words appearing ≥5 times:
  🚨 "the" (15x)
  ⚠️ "and" (12x)
  ⚠️ "to" (8x)
```

### Statistical Analysis
```
=== Statistical Analysis ===

Lines:
  Total lines: 42
  Unique lines: 38
  Duplicate lines: 4
  Max repetitions: 3
  Shannon entropy: 4.21

💀 Repetition Risk: Critical
⚠️ High repetition detected - possible AI loop or training data memorization
```

### JSON Format
```json
{
  "analysis_type": "words",
  "total_items": 156,
  "items": [
    {"content": "the", "count": 15},
    {"content": "and", "count": 12}
  ]
}
```

## Configuration Options

- `--repetition-threshold N`: Flag items appearing ≥N times (default: 5)
- `--top-n N`: Show top N most frequent items (default: 20)
- `--ngram-size N`: N-gram size for phrase analysis (default: 3)
- `--min-count N`: Minimum count to display (default: 1)
- `--format FORMAT`: Output format (text or json)

## Installation

```bash
cargo build --release -p ai-uniq
cargo install --path tools/ai-uniq
```

## Examples

### Basic Usage
```bash
# Count duplicate lines (traditional uniq)
ai-uniq -c input.txt

# Show only duplicates
ai-uniq -d input.txt

# Case insensitive, show only unique lines
ai-uniq -i -u input.txt
```

### AI-Specific Analysis
```bash
# Analyze word patterns in AI output
ai_generate prompt.txt | ai-uniq --word-analysis --above-threshold

# Detect phrase loops with 4-grams
ai-uniq --phrase-analysis --ngram-size 4 ai_response.txt

# Comprehensive loop detection
ai-uniq --detect-loops --format json ai_output.txt
```

### Pipeline Integration
```bash
# Statistical verification pipeline
ai_generate batch.txt | ai-uniq --stats --format json | \
  jq '.overall_risk == "Critical"'

# Word frequency equivalent to classic Unix pipeline
ai_output.txt | ai-uniq --word-analysis --sort-freq --top-n 10
```

### Quality Assurance
```bash
# Flag outputs with suspicious repetition
ai-uniq --word-analysis --repetition-threshold 10 --above-threshold *.txt

# Comprehensive analysis with risk assessment
ai-uniq --stats ai_responses/ | grep -E "(High|Critical)"
```