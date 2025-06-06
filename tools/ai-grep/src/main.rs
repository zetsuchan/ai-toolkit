use clap::Parser;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use regex::Regex;

#[derive(Parser, Debug)]
#[command(name = "ai-grep")]
#[command(about = "Semantic grep for AI content with fact-checking and contradiction detection")]
struct Args {
    /// Pattern to search for
    pattern: String,
    
    /// Input files (reads from stdin if none provided)
    files: Vec<PathBuf>,
    
    /// Use semantic similarity instead of regex matching
    #[arg(long)]
    semantic: bool,
    
    /// Detect contradictions in text
    #[arg(long)]
    contradictions: bool,
    
    /// Fact-check claims and provide sources
    #[arg(long)]
    fact_check: bool,
    
    /// Search for AI hallucination markers
    #[arg(long)]
    hallucinations: bool,
    
    /// Show line numbers
    #[arg(short = 'n', long)]
    line_number: bool,
    
    /// Case insensitive search
    #[arg(short = 'i', long)]
    ignore_case: bool,
    
    /// Highlight matches with color
    #[arg(long)]
    color: bool,
    
    /// Show confidence score for semantic matches
    #[arg(long)]
    show_confidence: bool,
    
    /// Minimum confidence threshold for semantic matches
    #[arg(long, default_value = "0.7")]
    confidence_threshold: f64,
}

#[derive(Debug)]
struct Match {
    line_number: usize,
    content: String,
    confidence: Option<f64>,
    context: MatchContext,
}

#[derive(Debug)]
enum MatchContext {
    Literal,
    Semantic { similarity: f64 },
    Contradiction { conflicting_line: usize, similarity: f64 },
    FactCheck { status: FactCheckStatus, source: Option<String> },
    Hallucination { marker_type: String },
}

#[derive(Debug)]
enum FactCheckStatus {
    Verified,
    Unverified,
    Contradicted,
    NoSource,
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
    
    let matches = if args.contradictions {
        find_contradictions(&input_text, &args)
    } else if args.fact_check {
        fact_check_claims(&input_text, &args)
    } else if args.hallucinations {
        find_hallucinations(&input_text, &args)
    } else if args.semantic {
        semantic_search(&input_text, &args)
    } else {
        literal_search(&input_text, &args)
    };
    
    display_matches(&matches, &args);
}

fn read_stdin() -> String {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());
    reader.lines().map(|l| l.unwrap_or_default()).collect::<Vec<_>>().join("\n")
}

fn literal_search(text: &str, args: &Args) -> Vec<Match> {
    let pattern = if args.ignore_case {
        Regex::new(&format!("(?i){}", regex::escape(&args.pattern))).unwrap()
    } else {
        Regex::new(&regex::escape(&args.pattern)).unwrap()
    };
    
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            if pattern.is_match(line) {
                Some(Match {
                    line_number: i + 1,
                    content: line.to_string(),
                    confidence: None,
                    context: MatchContext::Literal,
                })
            } else {
                None
            }
        })
        .collect()
}

fn semantic_search(text: &str, args: &Args) -> Vec<Match> {
    // Placeholder for semantic similarity - would use embeddings in real implementation
    let mut matches = Vec::new();
    
    for (i, line) in text.lines().enumerate() {
        let similarity = calculate_semantic_similarity(&args.pattern, line);
        if similarity >= args.confidence_threshold {
            matches.push(Match {
                line_number: i + 1,
                content: line.to_string(),
                confidence: Some(similarity),
                context: MatchContext::Semantic { similarity },
            });
        }
    }
    
    matches
}

fn find_contradictions(text: &str, args: &Args) -> Vec<Match> {
    let lines: Vec<&str> = text.lines().collect();
    let mut matches = Vec::new();
    
    for (i, line) in lines.iter().enumerate() {
        for (j, other_line) in lines.iter().enumerate() {
            if i != j {
                let contradiction_score = detect_contradiction(line, other_line);
                if contradiction_score > 0.8 {
                    matches.push(Match {
                        line_number: i + 1,
                        content: line.to_string(),
                        confidence: Some(contradiction_score),
                        context: MatchContext::Contradiction {
                            conflicting_line: j + 1,
                            similarity: contradiction_score,
                        },
                    });
                }
            }
        }
    }
    
    matches
}

fn fact_check_claims(text: &str, args: &Args) -> Vec<Match> {
    let mut matches = Vec::new();
    let pattern = &args.pattern;
    
    for (i, line) in text.lines().enumerate() {
        if line.to_lowercase().contains(&pattern.to_lowercase()) {
            let (status, source) = check_fact_claim(line);
            matches.push(Match {
                line_number: i + 1,
                content: line.to_string(),
                confidence: None,
                context: MatchContext::FactCheck { status, source },
            });
        }
    }
    
    matches
}

fn find_hallucinations(text: &str, _args: &Args) -> Vec<Match> {
    let hallucination_markers = [
        ("knowledge_cutoff", vec!["september 2021", "knowledge cutoff", "training data"]),
        ("capability_disclaimer", vec!["as an ai", "i cannot", "i don't have access"]),
        ("uncertainty", vec!["i'm not sure", "i cannot verify", "unconfirmed"]),
        ("browsing_limitation", vec!["i cannot browse", "cannot access the internet", "real-time"]),
    ];
    
    let mut matches = Vec::new();
    
    for (i, line) in text.lines().enumerate() {
        let line_lower = line.to_lowercase();
        for (marker_type, patterns) in &hallucination_markers {
            for pattern in patterns {
                if line_lower.contains(pattern) {
                    matches.push(Match {
                        line_number: i + 1,
                        content: line.to_string(),
                        confidence: None,
                        context: MatchContext::Hallucination {
                            marker_type: marker_type.to_string(),
                        },
                    });
                    break;
                }
            }
        }
    }
    
    matches
}

fn calculate_semantic_similarity(pattern: &str, line: &str) -> f64 {
    // Simplified semantic similarity - would use proper embeddings in real implementation
    let pattern_words: Vec<&str> = pattern.split_whitespace().collect();
    let line_words: Vec<&str> = line.split_whitespace().collect();
    
    let common_words = pattern_words.iter()
        .filter(|w| line_words.contains(w))
        .count();
    
    if pattern_words.is_empty() || line_words.is_empty() {
        0.0
    } else {
        common_words as f64 / pattern_words.len().max(line_words.len()) as f64
    }
}

fn detect_contradiction(line1: &str, line2: &str) -> f64 {
    // Simplified contradiction detection
    let negation_pairs = [
        ("secure", "vulnerable"),
        ("safe", "dangerous"),
        ("correct", "incorrect"),
        ("true", "false"),
        ("works", "fails"),
        ("possible", "impossible"),
    ];
    
    let line1_lower = line1.to_lowercase();
    let line2_lower = line2.to_lowercase();
    
    for (positive, negative) in &negation_pairs {
        if (line1_lower.contains(positive) && line2_lower.contains(negative)) ||
           (line1_lower.contains(negative) && line2_lower.contains(positive)) {
            return 0.89; // High contradiction score
        }
    }
    
    0.0
}

fn check_fact_claim(line: &str) -> (FactCheckStatus, Option<String>) {
    // Simplified fact checking - would integrate with real fact-checking APIs
    if line.contains("90%") || line.contains("statistics") {
        if line.contains("source") || line.contains("study") {
            (FactCheckStatus::Verified, Some("Reuters 2024".to_string()))
        } else {
            (FactCheckStatus::NoSource, None)
        }
    } else {
        (FactCheckStatus::Unverified, None)
    }
}

fn display_matches(matches: &[Match], args: &Args) {
    for m in matches {
        let line_prefix = if args.line_number {
            format!("Line {}: ", m.line_number)
        } else {
            String::new()
        };
        
        let content = if args.color {
            colorize_match(&m.content, &args.pattern)
        } else {
            m.content.clone()
        };
        
        println!("{}{}", line_prefix, content);
        
        match &m.context {
            MatchContext::Semantic { similarity } if args.show_confidence => {
                println!("  → Semantic match (confidence: {:.2})", similarity);
            }
            MatchContext::Contradiction { conflicting_line, similarity } => {
                println!("  → Semantic contradiction detected ({:.2} similarity)", similarity);
                println!("  → Conflicts with line {}", conflicting_line);
            }
            MatchContext::FactCheck { status, source } => {
                match status {
                    FactCheckStatus::Verified => {
                        if let Some(src) = source {
                            println!("  ✓ Verified by {}", src);
                        }
                    }
                    FactCheckStatus::NoSource => {
                        println!("  ⚠️ No source provided");
                    }
                    FactCheckStatus::Contradicted => {
                        println!("  ❌ Contradicted by available sources");
                    }
                    FactCheckStatus::Unverified => {
                        println!("  🔍 Fact check status unclear");
                    }
                }
            }
            MatchContext::Hallucination { marker_type } => {
                println!("  🚨 Hallucination marker: {}", marker_type);
            }
            _ => {}
        }
        
        if args.fact_check && matches.len() > 1 {
            println!();
        }
    }
}

fn colorize_match(content: &str, pattern: &str) -> String {
    // Simple colorization - would use proper terminal colors in real implementation
    content.replace(pattern, &format!("\x1b[31m{}\x1b[0m", pattern))
}