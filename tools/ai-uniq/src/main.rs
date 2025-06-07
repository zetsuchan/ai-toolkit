use clap::Parser;
use regex::Regex;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(name = "ai-uniq")]
#[command(about = "Statistical verification and deduplication for AI outputs with repetition detection")]
struct Args {
    /// Input files (reads from stdin if none provided)
    files: Vec<PathBuf>,
    
    /// Count occurrences like uniq -c
    #[arg(short = 'c', long)]
    count: bool,
    
    /// Only show duplicate lines like uniq -d
    #[arg(short = 'd', long)]
    duplicates: bool,
    
    /// Only show unique lines like uniq -u
    #[arg(short = 'u', long)]
    unique: bool,
    
    /// Case insensitive comparison like uniq -i
    #[arg(short = 'i', long)]
    ignore_case: bool,
    
    /// Skip first N fields like uniq -f
    #[arg(short = 'f', long)]
    skip_fields: Option<usize>,
    
    /// Skip first N characters like uniq -s
    #[arg(short = 's', long)]
    skip_chars: Option<usize>,
    
    /// Check only first N characters like uniq -w
    #[arg(short = 'w', long)]
    check_chars: Option<usize>,
    
    /// AI-specific: Analyze word repetition patterns
    #[arg(long)]
    word_analysis: bool,
    
    /// AI-specific: Analyze phrase repetition (N-grams)
    #[arg(long)]
    phrase_analysis: bool,
    
    /// AI-specific: N-gram size for phrase analysis
    #[arg(long, default_value = "3")]
    ngram_size: usize,
    
    /// AI-specific: Flag repetitions above threshold
    #[arg(long, default_value = "5")]
    repetition_threshold: usize,
    
    /// AI-specific: Show top N most repeated items
    #[arg(long, default_value = "20")]
    top_n: usize,
    
    /// AI-specific: Detect AI loop patterns
    #[arg(long)]
    detect_loops: bool,
    
    /// AI-specific: Statistical analysis of repetition
    #[arg(long)]
    stats: bool,
    
    /// Sort output by frequency (descending)
    #[arg(long)]
    sort_freq: bool,
    
    /// Sort output numerically
    #[arg(long)]
    numeric_sort: bool,
    
    /// Reverse sort order
    #[arg(short = 'r', long)]
    reverse: bool,
    
    /// Output format: text, json
    #[arg(long, default_value = "text")]
    format: String,
    
    /// Show only items above threshold
    #[arg(long)]
    above_threshold: bool,
    
    /// Minimum repetition count to display
    #[arg(long, default_value = "1")]
    min_count: usize,
}

#[derive(Debug, Clone)]
struct RepetitionStats {
    total_items: usize,
    unique_items: usize,
    duplicate_items: usize,
    max_repetition: usize,
    avg_repetition: f64,
    repetition_ratio: f64,
    entropy: f64,
    loop_indicators: Vec<LoopIndicator>,
}

#[derive(Debug, Clone)]
struct LoopIndicator {
    pattern: String,
    count: usize,
    pattern_type: LoopType,
    severity: Severity,
}

#[derive(Debug, Clone)]
enum LoopType {
    ExactRepeat,
    PhraseLoop,
    WordLoop,
    PatternLoop,
}

#[derive(Debug, Clone)]
enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    fn to_emoji(&self) -> &'static str {
        match self {
            Severity::Low => "ℹ️",
            Severity::Medium => "⚠️", 
            Severity::High => "🚨",
            Severity::Critical => "💀",
        }
    }
    
    fn threshold(&self) -> usize {
        match self {
            Severity::Low => 3,
            Severity::Medium => 5,
            Severity::High => 10,
            Severity::Critical => 20,
        }
    }
}

#[derive(Debug, Clone)]
struct CountedItem {
    content: String,
    count: usize,
    normalized: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let input = if args.files.is_empty() {
        read_stdin()?
    } else {
        args.files.iter()
            .map(|f| fs::read_to_string(f))
            .collect::<Result<Vec<_>, _>>()?
            .join("\n")
    };
    
    if args.word_analysis {
        analyze_words(&input, &args)?;
    } else if args.phrase_analysis {
        analyze_phrases(&input, &args)?;
    } else if args.detect_loops {
        detect_ai_loops(&input, &args)?;
    } else if args.stats {
        show_statistics(&input, &args)?;
    } else {
        // Traditional uniq functionality
        process_lines(&input, &args)?;
    }
    
    Ok(())
}

fn read_stdin() -> Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn process_lines(input: &str, args: &Args) -> Result<()> {
    let lines: Vec<String> = input.lines().map(|s| s.to_string()).collect();
    let counted_items = count_items(&lines, args);
    
    let filtered_items = filter_items(&counted_items, args);
    let sorted_items = sort_items(filtered_items, args);
    
    output_items(&sorted_items, args)?;
    Ok(())
}

fn analyze_words(input: &str, args: &Args) -> Result<()> {
    // Split into words, similar to: tr ' ' '\n'
    let words: Vec<String> = input
        .split_whitespace()
        .map(|w| normalize_word(w, args))
        .collect();
    
    let counted_words = count_items(&words, args);
    let filtered_words = filter_items(&counted_words, args);
    let sorted_words = sort_items(filtered_words, args);
    
    if args.format == "json" {
        output_json_analysis(&sorted_words, "words", args)?;
    } else {
        println!("=== Word Frequency Analysis ===");
        output_items(&sorted_words, args)?;
        
        if args.above_threshold {
            let flagged: Vec<_> = sorted_words.iter()
                .filter(|item| item.count >= args.repetition_threshold)
                .collect();
                
            if !flagged.is_empty() {
                println!("\n🚨 FLAGGED: Words appearing ≥{} times:", args.repetition_threshold);
                for item in flagged {
                    println!("  {} {} ({}x)", 
                            get_severity_for_count(item.count).to_emoji(),
                            item.content, 
                            item.count);
                }
            }
        }
    }
    
    Ok(())
}

fn analyze_phrases(input: &str, args: &Args) -> Result<()> {
    let words: Vec<String> = input
        .split_whitespace()
        .map(|w| normalize_word(w, args))
        .collect();
    
    let mut phrases = Vec::new();
    
    // Generate N-grams
    for window in words.windows(args.ngram_size) {
        let phrase = window.join(" ");
        phrases.push(phrase);
    }
    
    let counted_phrases = count_items(&phrases, args);
    let filtered_phrases = filter_items(&counted_phrases, args);
    let sorted_phrases = sort_items(filtered_phrases, args);
    
    if args.format == "json" {
        output_json_analysis(&sorted_phrases, "phrases", args)?;
    } else {
        println!("=== {}-gram Phrase Analysis ===", args.ngram_size);
        output_items(&sorted_phrases, args)?;
        
        if args.above_threshold {
            let flagged: Vec<_> = sorted_phrases.iter()
                .filter(|item| item.count >= args.repetition_threshold)
                .collect();
                
            if !flagged.is_empty() {
                println!("\n🚨 FLAGGED: Phrases appearing ≥{} times:", args.repetition_threshold);
                for item in flagged {
                    println!("  {} \"{}\" ({}x)", 
                            get_severity_for_count(item.count).to_emoji(),
                            item.content, 
                            item.count);
                }
            }
        }
    }
    
    Ok(())
}

fn detect_ai_loops(input: &str, args: &Args) -> Result<()> {
    let mut loop_indicators = Vec::new();
    
    // Check for exact line repetitions
    let lines: Vec<String> = input.lines().map(|s| s.to_string()).collect();
    let line_counts = count_items(&lines, args);
    
    for item in &line_counts {
        if item.count >= args.repetition_threshold {
            loop_indicators.push(LoopIndicator {
                pattern: item.content.clone(),
                count: item.count,
                pattern_type: LoopType::ExactRepeat,
                severity: get_severity_for_count(item.count),
            });
        }
    }
    
    // Check for word loops
    let words: Vec<String> = input
        .split_whitespace()
        .map(|w| normalize_word(w, args))
        .collect();
    let word_counts = count_items(&words, args);
    
    for item in &word_counts {
        if item.count >= args.repetition_threshold * 2 { // Higher threshold for words
            loop_indicators.push(LoopIndicator {
                pattern: item.content.clone(),
                count: item.count,
                pattern_type: LoopType::WordLoop,
                severity: get_severity_for_count(item.count),
            });
        }
    }
    
    // Check for phrase loops (3-grams)
    let mut phrases = Vec::new();
    for window in words.windows(3) {
        phrases.push(window.join(" "));
    }
    let phrase_counts = count_items(&phrases, args);
    
    for item in &phrase_counts {
        if item.count >= args.repetition_threshold {
            loop_indicators.push(LoopIndicator {
                pattern: item.content.clone(),
                count: item.count,
                pattern_type: LoopType::PhraseLoop,
                severity: get_severity_for_count(item.count),
            });
        }
    }
    
    // Check for pattern loops (regex-based)
    detect_pattern_loops(input, &mut loop_indicators, args);
    
    // Sort by severity and count
    loop_indicators.sort_by(|a, b| {
        b.count.cmp(&a.count)
            .then_with(|| match (&a.severity, &b.severity) {
                (Severity::Critical, _) => std::cmp::Ordering::Less,
                (_, Severity::Critical) => std::cmp::Ordering::Greater,
                (Severity::High, _) => std::cmp::Ordering::Less,
                (_, Severity::High) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            })
    });
    
    if args.format == "json" {
        output_json_loops(&loop_indicators, args)?;
    } else {
        output_loop_analysis(&loop_indicators, args)?;
    }
    
    Ok(())
}

fn detect_pattern_loops(input: &str, loop_indicators: &mut Vec<LoopIndicator>, args: &Args) {
    // Common AI loop patterns
    let patterns = [
        (r"(?i)\b(the same|similar|likewise|similarly|in the same way)\b", "similarity_loop"),
        (r"(?i)\b(as mentioned|as stated|as discussed|as noted)\b", "reference_loop"),
        (r"(?i)\b(it is important|it's important|this is important)\b", "importance_loop"),
        (r"(?i)\b(in conclusion|to conclude|in summary|to summarize)\b", "conclusion_loop"),
        (r"(?i)\b(however|nevertheless|nonetheless|on the other hand)\b", "transition_loop"),
    ];
    
    for (pattern_str, pattern_name) in &patterns {
        if let Ok(regex) = Regex::new(pattern_str) {
            let matches: Vec<_> = regex.find_iter(input).collect();
            let count = matches.len();
            
            if count >= args.repetition_threshold {
                loop_indicators.push(LoopIndicator {
                    pattern: format!("{} pattern", pattern_name),
                    count,
                    pattern_type: LoopType::PatternLoop,
                    severity: get_severity_for_count(count),
                });
            }
        }
    }
}

fn show_statistics(input: &str, args: &Args) -> Result<()> {
    let lines: Vec<String> = input.lines().map(|s| s.to_string()).collect();
    let words: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
    
    let line_counts = count_items(&lines, args);
    let word_counts = count_items(&words, args);
    
    let line_stats = calculate_stats(&line_counts);
    let word_stats = calculate_stats(&word_counts);
    
    if args.format == "json" {
        output_json_stats(&line_stats, &word_stats, args)?;
    } else {
        println!("=== Statistical Analysis ===");
        println!();
        println!("Lines:");
        print_stats(&line_stats, "line");
        println!();
        println!("Words:");
        print_stats(&word_stats, "word");
        
        // Overall assessment
        let overall_risk = assess_repetition_risk(&line_stats, &word_stats);
        println!();
        println!("=== Overall Assessment ===");
        println!("{} Repetition Risk: {:?}", overall_risk.to_emoji(), overall_risk);
        
        if matches!(overall_risk, Severity::High | Severity::Critical) {
            println!("⚠️  High repetition detected - possible AI loop or training data memorization");
        }
    }
    
    Ok(())
}

fn count_items(items: &[String], args: &Args) -> Vec<CountedItem> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    
    for item in items {
        let normalized = normalize_item(item, args);
        *counts.entry(normalized.clone()).or_insert(0) += 1;
    }
    
    counts.into_iter()
        .map(|(normalized, count)| CountedItem {
            content: normalized.clone(),
            count,
            normalized,
        })
        .collect()
}

fn normalize_item(item: &str, args: &Args) -> String {
    let mut result = item.to_string();
    
    if args.ignore_case {
        result = result.to_lowercase();
    }
    
    if let Some(skip_chars) = args.skip_chars {
        if skip_chars < result.len() {
            result = result[skip_chars..].to_string();
        }
    }
    
    if let Some(check_chars) = args.check_chars {
        if check_chars < result.len() {
            result = result[..check_chars].to_string();
        }
    }
    
    if let Some(skip_fields) = args.skip_fields {
        let fields: Vec<&str> = result.split_whitespace().collect();
        if skip_fields < fields.len() {
            result = fields[skip_fields..].join(" ");
        }
    }
    
    result
}

fn normalize_word(word: &str, args: &Args) -> String {
    let mut result = word.to_string();
    
    // Remove punctuation for word analysis
    result = result.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();
    
    if args.ignore_case {
        result = result.to_lowercase();
    }
    
    result
}

fn filter_items(items: &[CountedItem], args: &Args) -> Vec<CountedItem> {
    items.iter()
        .filter(|item| {
            if args.duplicates {
                item.count > 1
            } else if args.unique {
                item.count == 1
            } else {
                item.count >= args.min_count
            }
        })
        .cloned()
        .collect()
}

fn sort_items(mut items: Vec<CountedItem>, args: &Args) -> Vec<CountedItem> {
    if args.sort_freq {
        items.sort_by(|a, b| {
            if args.reverse {
                a.count.cmp(&b.count)
            } else {
                b.count.cmp(&a.count)
            }
        });
    } else if args.numeric_sort {
        items.sort_by(|a, b| {
            let a_num = a.content.parse::<i64>().unwrap_or(0);
            let b_num = b.content.parse::<i64>().unwrap_or(0);
            if args.reverse {
                b_num.cmp(&a_num)
            } else {
                a_num.cmp(&b_num)
            }
        });
    } else {
        items.sort_by(|a, b| {
            if args.reverse {
                b.content.cmp(&a.content)
            } else {
                a.content.cmp(&b.content)
            }
        });
    }
    
    // Apply top_n limit
    if args.word_analysis || args.phrase_analysis {
        items.truncate(args.top_n);
    }
    
    items
}

fn output_items(items: &[CountedItem], args: &Args) -> Result<()> {
    for item in items {
        if args.count {
            println!("{:8} {}", item.count, item.content);
        } else {
            println!("{}", item.content);
        }
    }
    Ok(())
}

fn calculate_stats(items: &[CountedItem]) -> RepetitionStats {
    let total_items: usize = items.iter().map(|i| i.count).sum();
    let unique_items = items.len();
    let duplicate_items = items.iter().filter(|i| i.count > 1).count();
    let max_repetition = items.iter().map(|i| i.count).max().unwrap_or(0);
    let avg_repetition = if unique_items > 0 {
        total_items as f64 / unique_items as f64
    } else {
        0.0
    };
    let repetition_ratio = if total_items > 0 {
        duplicate_items as f64 / total_items as f64
    } else {
        0.0
    };
    
    // Calculate entropy (Shannon entropy)
    let entropy = if total_items > 0 {
        items.iter()
            .map(|item| {
                let p = item.count as f64 / total_items as f64;
                if p > 0.0 {
                    -p * p.log2()
                } else {
                    0.0
                }
            })
            .sum()
    } else {
        0.0
    };
    
    RepetitionStats {
        total_items,
        unique_items,
        duplicate_items,
        max_repetition,
        avg_repetition,
        repetition_ratio,
        entropy,
        loop_indicators: Vec::new(),
    }
}

fn get_severity_for_count(count: usize) -> Severity {
    if count >= 20 {
        Severity::Critical
    } else if count >= 10 {
        Severity::High
    } else if count >= 5 {
        Severity::Medium
    } else {
        Severity::Low
    }
}

fn assess_repetition_risk(line_stats: &RepetitionStats, word_stats: &RepetitionStats) -> Severity {
    let line_risk = if line_stats.max_repetition >= 10 { 3 } 
                   else if line_stats.max_repetition >= 5 { 2 }
                   else if line_stats.max_repetition >= 3 { 1 } 
                   else { 0 };
    
    let word_risk = if word_stats.max_repetition >= 50 { 3 }
                   else if word_stats.max_repetition >= 20 { 2 }
                   else if word_stats.max_repetition >= 10 { 1 }
                   else { 0 };
    
    let entropy_risk = if line_stats.entropy < 2.0 { 2 }
                      else if line_stats.entropy < 3.0 { 1 }
                      else { 0 };
    
    let total_risk = line_risk + word_risk + entropy_risk;
    
    match total_risk {
        0..=2 => Severity::Low,
        3..=4 => Severity::Medium,
        5..=6 => Severity::High,
        _ => Severity::Critical,
    }
}

fn print_stats(stats: &RepetitionStats, item_type: &str) {
    println!("  Total {}s: {}", item_type, stats.total_items);
    println!("  Unique {}s: {}", item_type, stats.unique_items);
    println!("  Duplicate {}s: {}", item_type, stats.duplicate_items);
    println!("  Max repetitions: {}", stats.max_repetition);
    println!("  Avg repetitions: {:.2}", stats.avg_repetition);
    println!("  Repetition ratio: {:.2}%", stats.repetition_ratio * 100.0);
    println!("  Shannon entropy: {:.2}", stats.entropy);
}

fn output_loop_analysis(loop_indicators: &[LoopIndicator], args: &Args) -> Result<()> {
    if loop_indicators.is_empty() {
        println!("✅ No AI loops detected (threshold: {})", args.repetition_threshold);
        return Ok(());
    }
    
    println!("=== AI Loop Detection Results ===");
    println!("🚨 {} potential loops detected:", loop_indicators.len());
    println!();
    
    for indicator in loop_indicators {
        println!("{} {:?} Loop: \"{}\" ({}x)", 
                indicator.severity.to_emoji(),
                indicator.pattern_type,
                indicator.pattern,
                indicator.count);
    }
    
    let critical_count = loop_indicators.iter()
        .filter(|i| matches!(i.severity, Severity::Critical))
        .count();
    
    if critical_count > 0 {
        println!();
        println!("💀 CRITICAL: {} severe loops detected - likely AI malfunction", critical_count);
    }
    
    Ok(())
}

fn output_json_analysis(items: &[CountedItem], analysis_type: &str, _args: &Args) -> Result<()> {
    let mut output = serde_json::Map::new();
    output.insert("analysis_type".to_string(), serde_json::Value::String(analysis_type.to_string()));
    output.insert("total_items".to_string(), serde_json::Value::Number(serde_json::Number::from(items.len())));
    
    let items_json: Vec<serde_json::Value> = items.iter().map(|item| {
        let mut item_obj = serde_json::Map::new();
        item_obj.insert("content".to_string(), serde_json::Value::String(item.content.clone()));
        item_obj.insert("count".to_string(), serde_json::Value::Number(serde_json::Number::from(item.count)));
        serde_json::Value::Object(item_obj)
    }).collect();
    
    output.insert("items".to_string(), serde_json::Value::Array(items_json));
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn output_json_loops(loop_indicators: &[LoopIndicator], _args: &Args) -> Result<()> {
    let mut output = serde_json::Map::new();
    output.insert("analysis_type".to_string(), serde_json::Value::String("loop_detection".to_string()));
    output.insert("loops_detected".to_string(), serde_json::Value::Number(serde_json::Number::from(loop_indicators.len())));
    
    let loops_json: Vec<serde_json::Value> = loop_indicators.iter().map(|indicator| {
        let mut loop_obj = serde_json::Map::new();
        loop_obj.insert("pattern".to_string(), serde_json::Value::String(indicator.pattern.clone()));
        loop_obj.insert("count".to_string(), serde_json::Value::Number(serde_json::Number::from(indicator.count)));
        loop_obj.insert("type".to_string(), serde_json::Value::String(format!("{:?}", indicator.pattern_type)));
        loop_obj.insert("severity".to_string(), serde_json::Value::String(format!("{:?}", indicator.severity)));
        serde_json::Value::Object(loop_obj)
    }).collect();
    
    output.insert("loops".to_string(), serde_json::Value::Array(loops_json));
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn output_json_stats(line_stats: &RepetitionStats, word_stats: &RepetitionStats, _args: &Args) -> Result<()> {
    let mut output = serde_json::Map::new();
    
    let mut line_stats_obj = serde_json::Map::new();
    line_stats_obj.insert("total_items".to_string(), serde_json::Value::Number(serde_json::Number::from(line_stats.total_items)));
    line_stats_obj.insert("unique_items".to_string(), serde_json::Value::Number(serde_json::Number::from(line_stats.unique_items)));
    line_stats_obj.insert("duplicate_items".to_string(), serde_json::Value::Number(serde_json::Number::from(line_stats.duplicate_items)));
    line_stats_obj.insert("max_repetition".to_string(), serde_json::Value::Number(serde_json::Number::from(line_stats.max_repetition)));
    line_stats_obj.insert("avg_repetition".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(line_stats.avg_repetition).unwrap()));
    line_stats_obj.insert("repetition_ratio".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(line_stats.repetition_ratio).unwrap()));
    line_stats_obj.insert("entropy".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(line_stats.entropy).unwrap()));
    
    let mut word_stats_obj = serde_json::Map::new();
    word_stats_obj.insert("total_items".to_string(), serde_json::Value::Number(serde_json::Number::from(word_stats.total_items)));
    word_stats_obj.insert("unique_items".to_string(), serde_json::Value::Number(serde_json::Number::from(word_stats.unique_items)));
    word_stats_obj.insert("duplicate_items".to_string(), serde_json::Value::Number(serde_json::Number::from(word_stats.duplicate_items)));
    word_stats_obj.insert("max_repetition".to_string(), serde_json::Value::Number(serde_json::Number::from(word_stats.max_repetition)));
    word_stats_obj.insert("avg_repetition".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(word_stats.avg_repetition).unwrap()));
    word_stats_obj.insert("repetition_ratio".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(word_stats.repetition_ratio).unwrap()));
    word_stats_obj.insert("entropy".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(word_stats.entropy).unwrap()));
    
    output.insert("line_stats".to_string(), serde_json::Value::Object(line_stats_obj));
    output.insert("word_stats".to_string(), serde_json::Value::Object(word_stats_obj));
    
    let overall_risk = assess_repetition_risk(line_stats, word_stats);
    output.insert("overall_risk".to_string(), serde_json::Value::String(format!("{:?}", overall_risk)));
    
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}