# semdiff

Semantic diff tool for AI outputs that compares meaning beyond just text changes.

## Overview

`semdiff` extends traditional diff functionality by analyzing semantic similarity, fact extraction, confidence markers, and contradictions between AI outputs. Unlike regular diff that only shows line-by-line changes, semdiff understands the meaning and significance of differences.

## Usage

```bash
# Basic semantic comparison
semdiff file1.txt file2.txt

# Compare against stdin
ai_generate prompt.txt | semdiff reference.txt -

# JSON output for pipelines
semdiff --format json file1.txt file2.txt

# Adjust similarity threshold (0.0-1.0)
semdiff --threshold 0.8 file1.txt file2.txt

# Verbose analysis with all details
semdiff --verbose file1.txt file2.txt
```

## Features

### Semantic Similarity Analysis
- Concept extraction and overlap scoring
- Vocabulary similarity measurement
- Overall semantic similarity rating

### Fact Extraction & Comparison
- Automatic detection of dates, numbers, measurements
- Fact alignment and difference identification
- Missing or conflicting fact detection

### Confidence Analysis
- Hedging language detection ("might", "possibly", "likely")
- Definitive statement identification
- Confidence level comparison between texts

### Contradiction Detection
- Opposing fact identification
- Logical inconsistency flagging
- Conflicting claim analysis

## Output Formats

### Text Format
```
=== Semantic Similarity Analysis ===
Overall similarity: 0.85 (High)
Concept overlap: 12/15 concepts (80%)
Vocabulary similarity: 0.72

=== Fact Comparison ===
✅ Aligned facts: 8
⚠️  Different facts: 2
❌ Contradictions: 1
```

### JSON Format
```json
{
  "similarity": {
    "overall": 0.85,
    "concept_overlap": 0.80,
    "vocabulary_similarity": 0.72
  },
  "facts": {
    "aligned": 8,
    "different": 2,
    "contradictions": 1
  }
}
```

## Use Cases

- **AI Model Comparison**: Compare outputs from different models on same prompt
- **Version Validation**: Verify prompt changes don't alter meaning significantly  
- **Fact Checking**: Identify when AI outputs contain contradictory information
- **Quality Assurance**: Ensure AI responses maintain semantic consistency

## Installation

```bash
cargo build --release -p semdiff
cargo install --path tools/semdiff
```

## Examples

### Model Comparison
```bash
# Compare GPT vs Claude on same prompt
ai_generate_gpt prompt.txt > gpt_output.txt
ai_generate_claude prompt.txt > claude_output.txt
semdiff gpt_output.txt claude_output.txt
```

### Template Validation  
```bash
# Ensure new prompt version maintains meaning
ai_generate old_prompt.txt | semdiff templates/expected.txt -
```

### Pipeline Integration
```bash
# Flag outputs with low semantic similarity
ai_generate batch_prompts.txt | semdiff --format json reference.txt - | jq '.similarity.overall < 0.5'
```