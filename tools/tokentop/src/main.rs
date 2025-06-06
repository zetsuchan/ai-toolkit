use clap::Parser;
use std::collections::{HashMap, VecDeque};
use std::io::{self, BufRead, BufReader};
use std::time::{Duration, Instant};
use std::sync::mpsc;
use std::thread;

#[derive(Parser, Debug)]
#[command(name = "tokentop")]
#[command(about = "Real-time token analysis for AI generation - like htop but for AI tokens")]
struct Args {
    /// Update interval in milliseconds
    #[arg(short, long, default_value = "100")]
    interval: u64,
    
    /// Buffer size for rolling statistics
    #[arg(long, default_value = "1000")]
    buffer_size: usize,
    
    /// Show detailed pattern analysis
    #[arg(long)]
    patterns: bool,
    
    /// Alert threshold for repetition (0.0-1.0)
    #[arg(long, default_value = "0.4")]
    repetition_threshold: f64,
    
    /// Alert threshold for perplexity
    #[arg(long, default_value = "20.0")]
    perplexity_threshold: f64,
    
    /// Minimum confidence threshold
    #[arg(long, default_value = "0.5")]
    confidence_threshold: f64,
    
    /// Show raw tokens instead of analysis
    #[arg(long)]
    raw: bool,
}

#[derive(Debug, Clone)]
struct TokenStats {
    timestamp: Instant,
    token: String,
    perplexity: f64,
    confidence: f64,
    is_repetitive: bool,
}

#[derive(Debug)]
struct AnalysisState {
    tokens_per_second: f64,
    avg_perplexity: f64,
    repetition_score: f64,
    confidence_score: f64,
    detected_patterns: Vec<String>,
    warning_flags: Vec<String>,
    token_buffer: VecDeque<TokenStats>,
    pattern_tracker: HashMap<String, usize>,
}

impl AnalysisState {
    fn new(buffer_size: usize) -> Self {
        Self {
            tokens_per_second: 0.0,
            avg_perplexity: 0.0,
            repetition_score: 0.0,
            confidence_score: 0.0,
            detected_patterns: Vec::new(),
            warning_flags: Vec::new(),
            token_buffer: VecDeque::with_capacity(buffer_size),
            pattern_tracker: HashMap::new(),
        }
    }
    
    fn update(&mut self, token_stats: TokenStats, args: &Args) {
        // Add to buffer
        if self.token_buffer.len() >= self.token_buffer.capacity() {
            if let Some(old_token) = self.token_buffer.pop_front() {
                // Remove old pattern tracking
                let count = self.pattern_tracker.get(&old_token.token).unwrap_or(&0);
                if *count <= 1 {
                    self.pattern_tracker.remove(&old_token.token);
                } else {
                    self.pattern_tracker.insert(old_token.token, count - 1);
                }
            }
        }
        
        // Update pattern tracking
        *self.pattern_tracker.entry(token_stats.token.clone()).or_insert(0) += 1;
        
        self.token_buffer.push_back(token_stats);
        
        // Recalculate statistics
        self.calculate_metrics(args);
    }
    
    fn calculate_metrics(&mut self, args: &Args) {
        if self.token_buffer.is_empty() {
            return;
        }
        
        let now = Instant::now();
        
        // Calculate tokens per second
        let recent_tokens: Vec<_> = self.token_buffer.iter()
            .filter(|t| now.duration_since(t.timestamp) < Duration::from_secs(1))
            .collect();
        self.tokens_per_second = recent_tokens.len() as f64;
        
        // Calculate average perplexity
        self.avg_perplexity = self.token_buffer.iter()
            .map(|t| t.perplexity)
            .sum::<f64>() / self.token_buffer.len() as f64;
        
        // Calculate confidence score
        self.confidence_score = self.token_buffer.iter()
            .map(|t| t.confidence)
            .sum::<f64>() / self.token_buffer.len() as f64;
        
        // Calculate repetition score
        let total_tokens = self.token_buffer.len();
        let unique_tokens = self.pattern_tracker.len();
        self.repetition_score = if total_tokens > 0 {
            1.0 - (unique_tokens as f64 / total_tokens as f64)
        } else {
            0.0
        };
        
        // Detect patterns
        self.detect_patterns();
        
        // Check for warnings
        self.check_warnings(args);
    }
    
    fn detect_patterns(&mut self) {
        self.detected_patterns.clear();
        
        // Look for repeated phrases
        let tokens: Vec<String> = self.token_buffer.iter()
            .map(|t| t.token.clone())
            .collect();
        
        // Check for 3-gram repetitions
        for window in tokens.windows(3) {
            let phrase = window.join(" ");
            if let Some(&count) = self.pattern_tracker.get(&phrase) {
                if count >= 3 {
                    self.detected_patterns.push(format!("Repeated phrase: \"{}\" ({}x)", phrase, count));
                }
            }
        }
        
        // Check for listing patterns
        let listing_indicators = ["1.", "2.", "3.", "-", "*", "•"];
        let list_count = tokens.iter()
            .filter(|t| listing_indicators.iter().any(|&ind| t.starts_with(ind)))
            .count();
        
        if list_count >= 3 {
            self.detected_patterns.push("Listing pattern detected".to_string());
        }
        
        // Check for uncertainty language
        let uncertainty_words = ["maybe", "perhaps", "probably", "might", "could"];
        let uncertainty_count = tokens.iter()
            .filter(|t| uncertainty_words.iter().any(|&word| t.to_lowercase().contains(word)))
            .count();
        
        if uncertainty_count as f64 / tokens.len() as f64 > 0.1 {
            self.detected_patterns.push("High uncertainty language".to_string());
        }
    }
    
    fn check_warnings(&mut self, args: &Args) {
        self.warning_flags.clear();
        
        if self.repetition_score > args.repetition_threshold {
            self.warning_flags.push("High repetition detected".to_string());
        }
        
        if self.avg_perplexity > args.perplexity_threshold {
            self.warning_flags.push("Perplexity rising".to_string());
        }
        
        if self.confidence_score < args.confidence_threshold {
            self.warning_flags.push("Low confidence".to_string());
        }
        
        // Check for potential hallucination markers
        let recent_tokens: Vec<String> = self.token_buffer.iter()
            .rev()
            .take(20)
            .map(|t| t.token.clone())
            .collect();
        
        let hallucination_markers = ["As an AI", "I cannot", "I don't have access"];
        for marker in &hallucination_markers {
            if recent_tokens.iter().any(|t| t.contains(marker)) {
                self.warning_flags.push(format!("Hallucination marker: {}", marker));
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    
    if args.raw {
        run_raw_mode();
        return;
    }
    
    let (tx, rx) = mpsc::channel();
    
    // Spawn input reader thread
    thread::spawn(move || {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin.lock());
        
        for line in reader.lines() {
            if let Ok(line) = line {
                let tokens = tokenize_line(&line);
                for token in tokens {
                    let stats = analyze_token(&token);
                    if tx.send(stats).is_err() {
                        break;
                    }
                }
            }
        }
    });
    
    let mut state = AnalysisState::new(args.buffer_size);
    let mut last_update = Instant::now();
    
    // Initialize terminal
    print!("\x1b[2J\x1b[H"); // Clear screen and move cursor to top
    
    loop {
        // Check for new tokens
        while let Ok(token_stats) = rx.try_recv() {
            state.update(token_stats, &args);
        }
        
        // Update display at specified interval
        if last_update.elapsed() >= Duration::from_millis(args.interval) {
            display_stats(&state, &args);
            last_update = Instant::now();
        }
        
        thread::sleep(Duration::from_millis(10));
    }
}

fn run_raw_mode() {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());
    
    for line in reader.lines() {
        if let Ok(line) = line {
            let tokens = tokenize_line(&line);
            for token in tokens {
                println!("{}", token);
            }
        }
    }
}

fn tokenize_line(line: &str) -> Vec<String> {
    // Simple whitespace tokenization - would use proper tokenizer in real implementation
    line.split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

fn analyze_token(token: &str) -> TokenStats {
    // Simplified token analysis - would use actual language models
    let perplexity = calculate_perplexity(token);
    let confidence = calculate_confidence(token);
    let is_repetitive = is_repetitive_token(token);
    
    TokenStats {
        timestamp: Instant::now(),
        token: token.to_string(),
        perplexity,
        confidence,
        is_repetitive,
    }
}

fn calculate_perplexity(token: &str) -> f64 {
    // Simplified perplexity calculation
    match token.len() {
        1..=3 => 5.0 + (token.len() as f64 * 2.0),
        4..=8 => 10.0 + (token.len() as f64 * 1.5),
        _ => 15.0 + (token.len() as f64 * 0.5),
    }
}

fn calculate_confidence(token: &str) -> f64 {
    // Higher confidence for common words, lower for unusual patterns
    if token.chars().all(char::is_alphabetic) {
        0.8
    } else if token.contains(char::is_numeric) {
        0.6
    } else {
        0.4
    }
}

fn is_repetitive_token(token: &str) -> bool {
    // Check if token has repetitive character patterns
    if token.len() < 3 {
        return false;
    }
    
    let chars: Vec<char> = token.chars().collect();
    for window in chars.windows(2) {
        if window[0] == window[1] {
            return true;
        }
    }
    
    false
}

fn display_stats(state: &AnalysisState, args: &Args) {
    // Move cursor to top and clear screen
    print!("\x1b[H\x1b[2J");
    
    // Draw border
    println!("┌─ Token Statistics ──────────────────────┐");
    println!("│ Tokens/sec: {:<28.1} │", state.tokens_per_second);
    
    let perplexity_status = if state.avg_perplexity > args.perplexity_threshold {
        " ⚠️"
    } else {
        ""
    };
    println!("│ Perplexity: {:<24.1}{} │", state.avg_perplexity, perplexity_status);
    
    // Draw repetition bar
    let repetition_bar = create_progress_bar(state.repetition_score, 10);
    let repetition_percent = (state.repetition_score * 100.0) as u32;
    println!("│ Repetition: {:<19} {}% │", repetition_bar, repetition_percent);
    
    // Draw confidence bar
    let confidence_bar = create_progress_bar(state.confidence_score, 10);
    let confidence_percent = (state.confidence_score * 100.0) as u32;
    println!("│ Confidence: {:<19} {}% │", confidence_bar, confidence_percent);
    println!("│                                         │");
    
    // Show detected patterns
    if args.patterns && !state.detected_patterns.is_empty() {
        println!("│ Live patterns detected:                 │");
        for (i, pattern) in state.detected_patterns.iter().take(3).enumerate() {
            println!("│ - {:<36} │", truncate_string(pattern, 36));
        }
    } else {
        println!("│ No patterns detected                    │");
        println!("│                                         │");
        println!("│                                         │");
    }
    
    // Show warnings
    if !state.warning_flags.is_empty() {
        println!("│                                         │");
        println!("│ ⚠️  Warnings:                            │");
        for warning in &state.warning_flags {
            println!("│ - {:<36} │", truncate_string(warning, 36));
        }
    }
    
    println!("└─────────────────────────────────────────┘");
    
    // Show recent tokens if verbose
    if args.patterns && !state.token_buffer.is_empty() {
        println!("\nRecent tokens:");
        let recent: Vec<_> = state.token_buffer.iter().rev().take(10).collect();
        for token_stats in recent.iter().rev() {
            let confidence_indicator = if token_stats.confidence > 0.7 {
                "✓"
            } else if token_stats.confidence > 0.4 {
                "~"
            } else {
                "!"
            };
            println!("{} {} (p:{:.1}, c:{:.2})", 
                    confidence_indicator, 
                    token_stats.token, 
                    token_stats.perplexity, 
                    token_stats.confidence);
        }
    }
}

fn create_progress_bar(value: f64, width: usize) -> String {
    let filled = (value * width as f64) as usize;
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}