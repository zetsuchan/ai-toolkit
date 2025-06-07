use clap::Parser;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(name = "semdiff")]
#[command(about = "Semantic diff - compare meaning changes between AI outputs, not just text changes")]
struct Args {
    /// First file to compare (use - for stdin)
    file1: String,
    
    /// Second file to compare (use - for stdin for second input)
    file2: Option<String>,
    
    /// Show unified diff format
    #[arg(short, long)]
    unified: bool,
    
    /// Context lines for unified diff
    #[arg(short, long, default_value = "3")]
    context: usize,
    
    /// Ignore case when comparing
    #[arg(short, long)]
    ignore_case: bool,
    
    /// Ignore whitespace differences
    #[arg(short = 'w', long)]
    ignore_whitespace: bool,
    
    /// Show semantic similarity score (0.0-1.0)
    #[arg(long)]
    similarity_score: bool,
    
    /// Highlight key concept changes
    #[arg(long)]
    concept_diff: bool,
    
    /// Detect factual contradictions
    #[arg(long)]
    contradiction_check: bool,
    
    /// Show confidence marker changes
    #[arg(long)]
    confidence_diff: bool,
    
    /// Minimum semantic difference threshold (0.0-1.0)
    #[arg(long, default_value = "0.1")]
    threshold: f64,
    
    /// Output format: text, json
    #[arg(long, default_value = "text")]
    format: String,
}

#[derive(Debug, Clone)]
struct SemanticChunk {
    text: String,
    concepts: HashSet<String>,
    facts: Vec<String>,
    confidence_markers: Vec<String>,
    sentiment: f64,
}

#[derive(Debug)]
struct SemanticDiff {
    similarity_score: f64,
    concept_changes: Vec<ConceptChange>,
    fact_changes: Vec<FactChange>,
    confidence_changes: Vec<ConfidenceChange>,
    contradictions: Vec<Contradiction>,
    text_diff: Vec<DiffLine>,
}

#[derive(Debug)]
struct ConceptChange {
    concept: String,
    change_type: ChangeType,
    context: String,
}

#[derive(Debug)]
struct FactChange {
    fact: String,
    change_type: ChangeType,
    old_value: Option<String>,
    new_value: Option<String>,
}

#[derive(Debug)]
struct ConfidenceChange {
    marker: String,
    old_confidence: f64,
    new_confidence: f64,
    context: String,
}

#[derive(Debug)]
struct Contradiction {
    statement1: String,
    statement2: String,
    contradiction_type: String,
}

#[derive(Debug)]
enum ChangeType {
    Added,
    Removed,
    Modified,
}

#[derive(Debug)]
struct DiffLine {
    line_type: LineType,
    content: String,
    line_number: Option<usize>,
}

#[derive(Debug)]
enum LineType {
    Context,
    Added,
    Removed,
    Modified,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let text1 = read_input(&args.file1)?;
    let text2 = if let Some(file2) = &args.file2 {
        read_input(file2)?
    } else {
        // If only one file specified, read second from stdin
        read_stdin()?
    };
    
    let chunk1 = analyze_text(&text1, &args);
    let chunk2 = analyze_text(&text2, &args);
    
    let semantic_diff = compare_semantics(&chunk1, &chunk2, &args);
    
    // Filter by threshold
    if semantic_diff.similarity_score >= (1.0 - args.threshold) {
        if args.format == "json" {
            println!("{{\"similarity\": {:.3}, \"changes\": []}}", semantic_diff.similarity_score);
        } else {
            println!("No significant semantic differences (similarity: {:.3})", semantic_diff.similarity_score);
        }
        return Ok(());
    }
    
    output_diff(&semantic_diff, &args)?;
    
    Ok(())
}

fn read_input(filename: &str) -> Result<String> {
    if filename == "-" {
        read_stdin()
    } else {
        Ok(fs::read_to_string(filename)?)
    }
}

fn read_stdin() -> Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn analyze_text(text: &str, args: &Args) -> SemanticChunk {
    let processed_text = if args.ignore_case {
        text.to_lowercase()
    } else {
        text.to_string()
    };
    
    let processed_text = if args.ignore_whitespace {
        Regex::new(r"\s+").unwrap().replace_all(&processed_text, " ").to_string()
    } else {
        processed_text
    };
    
    let concepts = extract_concepts(&processed_text);
    let facts = extract_facts(&processed_text);
    let confidence_markers = extract_confidence_markers(&processed_text);
    let sentiment = calculate_sentiment(&processed_text);
    
    SemanticChunk {
        text: processed_text,
        concepts,
        facts,
        confidence_markers,
        sentiment,
    }
}

fn extract_concepts(text: &str) -> HashSet<String> {
    let mut concepts = HashSet::new();
    
    // Extract noun phrases (simplified)
    let noun_phrase_regex = Regex::new(r"\b[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*\b").unwrap();
    for cap in noun_phrase_regex.find_iter(text) {
        concepts.insert(cap.as_str().to_string());
    }
    
    // Extract technical terms
    let tech_terms_regex = Regex::new(r"\b[a-z]+(?:_[a-z]+)*\b|\b[A-Z]{2,}\b").unwrap();
    for cap in tech_terms_regex.find_iter(text) {
        if cap.as_str().len() > 3 {
            concepts.insert(cap.as_str().to_string());
        }
    }
    
    // Extract quoted concepts
    let quoted_regex = Regex::new(r#""([^"]+)"|'([^']+)'"#).unwrap();
    for cap in quoted_regex.captures_iter(text) {
        if let Some(quoted) = cap.get(1).or_else(|| cap.get(2)) {
            concepts.insert(quoted.as_str().to_string());
        }
    }
    
    concepts
}

fn extract_facts(text: &str) -> Vec<String> {
    let mut facts = Vec::new();
    
    // Extract statements with numbers, dates, or measurements
    let fact_patterns = [
        r"\b\d+(?:\.\d+)?\s*(?:percent|%|million|billion|thousand|years?|days?|hours?|minutes?|seconds?)\b",
        r"\b(?:January|February|March|April|May|June|July|August|September|October|November|December)\s+\d{1,2},?\s+\d{4}\b",
        r"\b\d{4}-\d{2}-\d{2}\b",
        r"\b\d+(?:\.\d+)?\s*(?:kg|pounds?|lbs?|meters?|feet|inches?|miles?|km|celsius|fahrenheit)\b",
    ];
    
    for pattern in &fact_patterns {
        let regex = Regex::new(pattern).unwrap();
        for mat in regex.find_iter(text) {
            // Get surrounding context
            let start = text[..mat.start()].rfind('.').map(|i| i + 1).unwrap_or(0);
            let end = text[mat.end()..].find('.').map(|i| mat.end() + i + 1).unwrap_or(text.len());
            let sentence = text[start..end].trim();
            if !sentence.is_empty() {
                facts.push(sentence.to_string());
            }
        }
    }
    
    facts
}

fn extract_confidence_markers(text: &str) -> Vec<String> {
    let confidence_patterns = [
        "likely", "probably", "might", "could", "perhaps", "possibly",
        "seems", "appears", "suggests", "indicates", "presumably",
        "allegedly", "supposedly", "apparently", "potentially",
        "definitely", "certainly", "absolutely", "clearly", "obviously",
        "undoubtedly", "without doubt", "surely", "indeed"
    ];
    
    let mut markers = Vec::new();
    let text_lower = text.to_lowercase();
    
    for pattern in &confidence_patterns {
        let regex = Regex::new(&format!(r"\b{}\b", regex::escape(pattern))).unwrap();
        for mat in regex.find_iter(&text_lower) {
            // Get surrounding context (20 chars before and after)
            let start = mat.start().saturating_sub(20);
            let end = (mat.end() + 20).min(text.len());
            let context = text[start..end].trim();
            markers.push(format!("{} ({})", pattern, context));
        }
    }
    
    markers
}

fn calculate_sentiment(text: &str) -> f64 {
    let positive_words = ["good", "great", "excellent", "positive", "successful", "correct", "accurate", "effective"];
    let negative_words = ["bad", "poor", "terrible", "negative", "failed", "incorrect", "inaccurate", "ineffective"];
    
    let text_lower = text.to_lowercase();
    let positive_count = positive_words.iter().map(|word| text_lower.matches(word).count()).sum::<usize>();
    let negative_count = negative_words.iter().map(|word| text_lower.matches(word).count()).sum::<usize>();
    
    let total = positive_count + negative_count;
    if total == 0 {
        0.5 // Neutral
    } else {
        positive_count as f64 / total as f64
    }
}

fn compare_semantics(chunk1: &SemanticChunk, chunk2: &SemanticChunk, args: &Args) -> SemanticDiff {
    let similarity_score = calculate_similarity_score(chunk1, chunk2);
    let concept_changes = compare_concepts(&chunk1.concepts, &chunk2.concepts);
    let fact_changes = compare_facts(&chunk1.facts, &chunk2.facts);
    let confidence_changes = compare_confidence(&chunk1.confidence_markers, &chunk2.confidence_markers);
    let contradictions = find_contradictions(&chunk1.facts, &chunk2.facts);
    let text_diff = if args.unified {
        create_unified_diff(&chunk1.text, &chunk2.text, args.context)
    } else {
        create_simple_diff(&chunk1.text, &chunk2.text)
    };
    
    SemanticDiff {
        similarity_score,
        concept_changes,
        fact_changes,
        confidence_changes,
        contradictions,
        text_diff,
    }
}

fn calculate_similarity_score(chunk1: &SemanticChunk, chunk2: &SemanticChunk) -> f64 {
    // Jaccard similarity for concepts
    let intersection = chunk1.concepts.intersection(&chunk2.concepts).count();
    let union = chunk1.concepts.union(&chunk2.concepts).count();
    let concept_similarity = if union > 0 {
        intersection as f64 / union as f64
    } else {
        1.0
    };
    
    // Fact overlap
    let fact_overlap = chunk1.facts.iter()
        .filter(|f1| chunk2.facts.iter().any(|f2| facts_similar(f1, f2)))
        .count();
    let total_facts = chunk1.facts.len().max(chunk2.facts.len());
    let fact_similarity = if total_facts > 0 {
        fact_overlap as f64 / total_facts as f64
    } else {
        1.0
    };
    
    // Sentiment similarity
    let sentiment_similarity = 1.0 - (chunk1.sentiment - chunk2.sentiment).abs();
    
    // Weighted average
    (concept_similarity * 0.5 + fact_similarity * 0.3 + sentiment_similarity * 0.2)
}

fn facts_similar(fact1: &str, fact2: &str) -> bool {
    // Simple similarity check - could be enhanced with more sophisticated NLP
    let words1: HashSet<&str> = fact1.split_whitespace().collect();
    let words2: HashSet<&str> = fact2.split_whitespace().collect();
    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();
    
    if union > 0 {
        intersection as f64 / union as f64 > 0.6
    } else {
        false
    }
}

fn compare_concepts(concepts1: &HashSet<String>, concepts2: &HashSet<String>) -> Vec<ConceptChange> {
    let mut changes = Vec::new();
    
    for concept in concepts1.difference(concepts2) {
        changes.push(ConceptChange {
            concept: concept.clone(),
            change_type: ChangeType::Removed,
            context: concept.clone(),
        });
    }
    
    for concept in concepts2.difference(concepts1) {
        changes.push(ConceptChange {
            concept: concept.clone(),
            change_type: ChangeType::Added,
            context: concept.clone(),
        });
    }
    
    changes
}

fn compare_facts(facts1: &[String], facts2: &[String]) -> Vec<FactChange> {
    let mut changes = Vec::new();
    
    for fact1 in facts1 {
        if !facts2.iter().any(|f2| facts_similar(fact1, f2)) {
            changes.push(FactChange {
                fact: fact1.clone(),
                change_type: ChangeType::Removed,
                old_value: Some(fact1.clone()),
                new_value: None,
            });
        }
    }
    
    for fact2 in facts2 {
        if !facts1.iter().any(|f1| facts_similar(f1, fact2)) {
            changes.push(FactChange {
                fact: fact2.clone(),
                change_type: ChangeType::Added,
                old_value: None,
                new_value: Some(fact2.clone()),
            });
        }
    }
    
    changes
}

fn compare_confidence(markers1: &[String], markers2: &[String]) -> Vec<ConfidenceChange> {
    let mut changes = Vec::new();
    
    let confidence1 = calculate_confidence_level(markers1);
    let confidence2 = calculate_confidence_level(markers2);
    
    if (confidence1 - confidence2).abs() > 0.1 {
        changes.push(ConfidenceChange {
            marker: "overall_confidence".to_string(),
            old_confidence: confidence1,
            new_confidence: confidence2,
            context: format!("Markers: {} -> {}", markers1.len(), markers2.len()),
        });
    }
    
    changes
}

fn calculate_confidence_level(markers: &[String]) -> f64 {
    let high_confidence = ["definitely", "certainly", "absolutely", "clearly", "obviously"];
    let low_confidence = ["might", "could", "perhaps", "possibly", "potentially"];
    
    let high_count = markers.iter()
        .filter(|m| high_confidence.iter().any(|hc| m.contains(hc)))
        .count();
    let low_count = markers.iter()
        .filter(|m| low_confidence.iter().any(|lc| m.contains(lc)))
        .count();
    
    let total = high_count + low_count;
    if total == 0 {
        0.5
    } else {
        high_count as f64 / total as f64
    }
}

fn find_contradictions(facts1: &[String], facts2: &[String]) -> Vec<Contradiction> {
    let mut contradictions = Vec::new();
    
    // Simple contradiction detection - look for opposite statements
    for fact1 in facts1 {
        for fact2 in facts2 {
            if might_contradict(fact1, fact2) {
                contradictions.push(Contradiction {
                    statement1: fact1.clone(),
                    statement2: fact2.clone(),
                    contradiction_type: "potential_contradiction".to_string(),
                });
            }
        }
    }
    
    contradictions
}

fn might_contradict(fact1: &str, fact2: &str) -> bool {
    let opposing_pairs = [
        ("increase", "decrease"),
        ("more", "less"),
        ("higher", "lower"),
        ("better", "worse"),
        ("positive", "negative"),
        ("success", "failure"),
        ("true", "false"),
        ("correct", "incorrect"),
    ];
    
    let fact1_lower = fact1.to_lowercase();
    let fact2_lower = fact2.to_lowercase();
    
    for (word1, word2) in &opposing_pairs {
        if (fact1_lower.contains(word1) && fact2_lower.contains(word2)) ||
           (fact1_lower.contains(word2) && fact2_lower.contains(word1)) {
            // Check if they're talking about the same subject
            let words1: HashSet<&str> = fact1.split_whitespace().collect();
            let words2: HashSet<&str> = fact2.split_whitespace().collect();
            let common_words = words1.intersection(&words2).count();
            if common_words >= 2 {
                return true;
            }
        }
    }
    
    false
}

fn create_unified_diff(text1: &str, text2: &str, context: usize) -> Vec<DiffLine> {
    let lines1: Vec<&str> = text1.lines().collect();
    let lines2: Vec<&str> = text2.lines().collect();
    
    let mut diff_lines = Vec::new();
    
    // Simple line-by-line diff (could be enhanced with proper diff algorithm)
    let max_lines = lines1.len().max(lines2.len());
    
    for i in 0..max_lines {
        let line1 = lines1.get(i);
        let line2 = lines2.get(i);
        
        match (line1, line2) {
            (Some(l1), Some(l2)) => {
                if l1 != l2 {
                    diff_lines.push(DiffLine {
                        line_type: LineType::Removed,
                        content: format!("-{}", l1),
                        line_number: Some(i + 1),
                    });
                    diff_lines.push(DiffLine {
                        line_type: LineType::Added,
                        content: format!("+{}", l2),
                        line_number: Some(i + 1),
                    });
                } else {
                    diff_lines.push(DiffLine {
                        line_type: LineType::Context,
                        content: format!(" {}", l1),
                        line_number: Some(i + 1),
                    });
                }
            }
            (Some(l1), None) => {
                diff_lines.push(DiffLine {
                    line_type: LineType::Removed,
                    content: format!("-{}", l1),
                    line_number: Some(i + 1),
                });
            }
            (None, Some(l2)) => {
                diff_lines.push(DiffLine {
                    line_type: LineType::Added,
                    content: format!("+{}", l2),
                    line_number: Some(i + 1),
                });
            }
            (None, None) => break,
        }
    }
    
    diff_lines
}

fn create_simple_diff(text1: &str, text2: &str) -> Vec<DiffLine> {
    if text1 == text2 {
        vec![DiffLine {
            line_type: LineType::Context,
            content: "No text differences".to_string(),
            line_number: None,
        }]
    } else {
        vec![
            DiffLine {
                line_type: LineType::Removed,
                content: format!("< {}", text1.replace('\n', "\\n")),
                line_number: None,
            },
            DiffLine {
                line_type: LineType::Added,
                content: format!("> {}", text2.replace('\n', "\\n")),
                line_number: None,
            }
        ]
    }
}

fn output_diff(diff: &SemanticDiff, args: &Args) -> Result<()> {
    if args.format == "json" {
        output_json(diff)?;
        return Ok(());
    }
    
    // Text output
    if args.similarity_score {
        println!("Semantic similarity: {:.3}", diff.similarity_score);
        println!();
    }
    
    if args.concept_diff && !diff.concept_changes.is_empty() {
        println!("=== Concept Changes ===");
        for change in &diff.concept_changes {
            let symbol = match change.change_type {
                ChangeType::Added => "+",
                ChangeType::Removed => "-",
                ChangeType::Modified => "~",
            };
            println!("{} {}", symbol, change.concept);
        }
        println!();
    }
    
    if !diff.fact_changes.is_empty() {
        println!("=== Fact Changes ===");
        for change in &diff.fact_changes {
            match change.change_type {
                ChangeType::Added => println!("+ {}", change.fact),
                ChangeType::Removed => println!("- {}", change.fact),
                ChangeType::Modified => {
                    if let (Some(old), Some(new)) = (&change.old_value, &change.new_value) {
                        println!("~ {} -> {}", old, new);
                    }
                }
            }
        }
        println!();
    }
    
    if args.confidence_diff && !diff.confidence_changes.is_empty() {
        println!("=== Confidence Changes ===");
        for change in &diff.confidence_changes {
            println!("{}: {:.2} -> {:.2} ({})", 
                     change.marker, change.old_confidence, change.new_confidence, change.context);
        }
        println!();
    }
    
    if args.contradiction_check && !diff.contradictions.is_empty() {
        println!("=== Potential Contradictions ===");
        for contradiction in &diff.contradictions {
            println!("! {} <-> {}", contradiction.statement1, contradiction.statement2);
        }
        println!();
    }
    
    if args.unified {
        println!("=== Text Diff ===");
        for line in &diff.text_diff {
            println!("{}", line.content);
        }
    }
    
    Ok(())
}

fn output_json(diff: &SemanticDiff) -> Result<()> {
    let mut output = serde_json::Map::new();
    
    output.insert("similarity_score".to_string(), 
                  serde_json::Value::Number(serde_json::Number::from_f64(diff.similarity_score).unwrap()));
    
    output.insert("concept_changes".to_string(), 
                  serde_json::Value::Number(serde_json::Number::from(diff.concept_changes.len())));
    
    output.insert("fact_changes".to_string(), 
                  serde_json::Value::Number(serde_json::Number::from(diff.fact_changes.len())));
    
    output.insert("confidence_changes".to_string(), 
                  serde_json::Value::Number(serde_json::Number::from(diff.confidence_changes.len())));
    
    output.insert("contradictions".to_string(), 
                  serde_json::Value::Number(serde_json::Number::from(diff.contradictions.len())));
    
    println!("{}", serde_json::to_string_pretty(&output)?);
    
    Ok(())
}