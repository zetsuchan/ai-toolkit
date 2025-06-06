use clap::Parser;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, Read};
use std::path::PathBuf;
use regex::Regex;

#[derive(Parser, Debug)]
#[command(name = "ai-wc")]
#[command(about = "Enhanced word count with AI-specific quality metrics")]
struct Args {
    /// Input files (reads from stdin if none provided)
    files: Vec<PathBuf>,
    
    /// Show hallucination markers count
    #[arg(long)]
    hallucination_markers: bool,
    
    /// Calculate fact density (facts per paragraph)
    #[arg(long)]
    fact_density: bool,
    
    /// Detect repetition patterns that indicate AI loops
    #[arg(long)]
    repetition_score: bool,
    
    /// Count AI confidence markers
    #[arg(long)]
    confidence_markers: bool,
    
    /// Show all AI-specific metrics
    #[arg(long)]
    ai_metrics: bool,
    
    /// Traditional word count (lines, words, chars)
    #[arg(short, long)]
    traditional: bool,
}

struct TextMetrics {
    lines: usize,
    words: usize,
    chars: usize,
    confidence_markers: usize,
    hallucination_indicators: usize,
    fact_density: f64,
    repetition_score: f64,
}

fn main() {
    let args = Args::parse();
    
    let input_text = if args.files.is_empty() {
        read_stdin()
    } else {
        args.files.iter()
            .map(|f| fs::read_to_string(f).unwrap_or_else(|_| String::new()))
            .collect::<Vec<_>>()
            .join("\n")
    };
    
    let metrics = analyze_text(&input_text);
    
    if args.traditional {
        println!("{:8} {:8} {:8}", metrics.lines, metrics.words, metrics.chars);
        return;
    }
    
    // Enhanced AI-specific output
    println!("Lines: {}", metrics.lines);
    println!("Words: {}", metrics.words);
    println!("Chars: {}", metrics.chars);
    
    if args.confidence_markers || args.ai_metrics {
        println!("AI Confidence markers: {} (\"likely\", \"probably\", \"might\")", metrics.confidence_markers);
    }
    
    if args.hallucination_markers || args.ai_metrics {
        println!("Hallucination indicators: {}", metrics.hallucination_indicators);
    }
    
    if args.fact_density || args.ai_metrics {
        println!("Fact density: {:.2} facts/paragraph", metrics.fact_density);
    }
    
    if args.repetition_score || args.ai_metrics {
        let repetition_level = if metrics.repetition_score > 0.5 { 
            "high - possible loop" 
        } else if metrics.repetition_score > 0.3 { 
            "medium" 
        } else { 
            "low" 
        };
        println!("Repetition score: {:.2} ({})", metrics.repetition_score, repetition_level);
    }
}

fn read_stdin() -> String {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).expect("Failed to read from stdin");
    buffer
}

fn analyze_text(text: &str) -> TextMetrics {
    let lines = text.lines().count();
    let words = text.split_whitespace().count();
    let chars = text.chars().count();
    
    // AI-specific analysis
    let confidence_markers = count_confidence_markers(text);
    let hallucination_indicators = count_hallucination_indicators(text);
    let fact_density = calculate_fact_density(text);
    let repetition_score = calculate_repetition_score(text);
    
    TextMetrics {
        lines,
        words,
        chars,
        confidence_markers,
        hallucination_indicators,
        fact_density,
        repetition_score,
    }
}

fn count_confidence_markers(text: &str) -> usize {
    let confidence_words = [
        "likely", "probably", "might", "could", "perhaps", "possibly", 
        "seems", "appears", "suggests", "indicates", "presumably", 
        "allegedly", "supposedly", "apparently", "potentially"
    ];
    
    let text_lower = text.to_lowercase();
    confidence_words.iter()
        .map(|&word| text_lower.matches(word).count())
        .sum()
}

fn count_hallucination_indicators(text: &str) -> usize {
    let hallucination_patterns = [
        "as an ai", "i cannot", "i don't have access", "september 2021",
        "knowledge cutoff", "i'm not able", "i cannot browse", "real-time",
        "i don't know", "i'm uncertain", "i can't verify", "unverified"
    ];
    
    let text_lower = text.to_lowercase();
    hallucination_patterns.iter()
        .map(|&pattern| text_lower.matches(pattern).count())
        .sum()
}

fn calculate_fact_density(text: &str) -> f64 {
    let paragraphs: Vec<&str> = text.split("\n\n").filter(|p| !p.trim().is_empty()).collect();
    if paragraphs.is_empty() {
        return 0.0;
    }
    
    // Simple heuristic: count sentences with numbers, dates, or proper nouns
    let fact_indicators = Regex::new(r"\b\d+\b|\b[A-Z][a-z]+ \d{4}\b|\b[A-Z][a-z]{2,}\b").unwrap();
    
    let total_facts: usize = paragraphs.iter()
        .map(|p| fact_indicators.find_iter(p).count())
        .sum();
    
    total_facts as f64 / paragraphs.len() as f64
}

fn calculate_repetition_score(text: &str) -> f64 {
    let sentences: Vec<&str> = text.split('.').filter(|s| s.trim().len() > 10).collect();
    if sentences.len() < 2 {
        return 0.0;
    }
    
    let mut phrase_counts = HashMap::new();
    
    // Count 3-word phrases
    for sentence in &sentences {
        let words: Vec<&str> = sentence.split_whitespace().collect();
        for window in words.windows(3) {
            let phrase = window.join(" ").to_lowercase();
            *phrase_counts.entry(phrase).or_insert(0) += 1;
        }
    }
    
    let repeated_phrases = phrase_counts.values().filter(|&&count| count > 1).count();
    let total_phrases = phrase_counts.len();
    
    if total_phrases == 0 {
        0.0
    } else {
        repeated_phrases as f64 / total_phrases as f64
    }
}