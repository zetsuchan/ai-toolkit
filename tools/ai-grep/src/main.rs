use clap::Parser;
use regex::Regex;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader, Read};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(name = "ai-grep")]
#[command(about = "AI-aware grep for anomaly detection in AI outputs and code")]
struct Args {
    /// Pattern to search for (optional with preset modes)
    pattern: Option<String>,
    
    /// Input files (reads from stdin if none provided)
    files: Vec<PathBuf>,
    
    /// Search for AI hallucination markers
    #[arg(long)]
    hallucinations: bool,
    
    /// Search for code quality issues (TODO, FIXME, etc.)
    #[arg(long)]
    code_issues: bool,
    
    /// Search for security vulnerabilities patterns
    #[arg(long)]
    security: bool,
    
    /// Search for AI training data leakage
    #[arg(long)]
    data_leakage: bool,
    
    /// Search for confidence undermining language
    #[arg(long)]
    low_confidence: bool,
    
    /// Use extended regex patterns (-E flag)
    #[arg(short = 'E', long)]
    extended_regex: bool,
    
    /// Use Perl-compatible regex patterns (-P flag)
    #[arg(short = 'P', long)]
    perl_regex: bool,
    
    /// Case insensitive search (-i flag)
    #[arg(short = 'i', long)]
    ignore_case: bool,
    
    /// Show line numbers (-n flag)
    #[arg(short = 'n', long)]
    line_number: bool,
    
    /// Count matches only (-c flag)
    #[arg(short = 'c', long)]
    count: bool,
    
    /// Invert match (show non-matching lines) (-v flag)
    #[arg(short = 'v', long)]
    invert_match: bool,
    
    /// Show only filenames with matches (-l flag)
    #[arg(short = 'l', long)]
    files_with_matches: bool,
    
    /// Highlight matches with color (--color)
    #[arg(long, value_enum, default_value = "auto")]
    color: ColorMode,
    
    /// Context lines after match
    #[arg(short = 'A', long, default_value = "0")]
    after_context: usize,
    
    /// Context lines before match
    #[arg(short = 'B', long, default_value = "0")]
    before_context: usize,
    
    /// Context lines around match
    #[arg(short = 'C', long)]
    context: Option<usize>,
    
    /// Suppress error messages
    #[arg(short = 's', long)]
    no_messages: bool,
    
    /// Output format: text, json
    #[arg(long, default_value = "text")]
    format: String,
    
    /// Show anomaly severity score
    #[arg(long)]
    severity: bool,
    
    /// List available preset patterns
    #[arg(long)]
    list_presets: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum ColorMode {
    Always,
    Never,
    Auto,
}

#[derive(Debug)]
struct Match {
    line_number: usize,
    content: String,
    matched_text: String,
    start_pos: usize,
    end_pos: usize,
    anomaly_type: AnomalyType,
    severity: Severity,
}

#[derive(Debug)]
enum AnomalyType {
    Hallucination { marker_type: String },
    CodeIssue { issue_type: String },
    Security { vulnerability_type: String },
    DataLeakage { leak_type: String },
    LowConfidence { confidence_marker: String },
    Custom,
}

#[derive(Debug, Clone)]
enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    fn to_score(&self) -> f64 {
        match self {
            Severity::Low => 0.25,
            Severity::Medium => 0.5,
            Severity::High => 0.75,
            Severity::Critical => 1.0,
        }
    }
    
    fn to_emoji(&self) -> &'static str {
        match self {
            Severity::Low => "ℹ️",
            Severity::Medium => "⚠️",
            Severity::High => "🚨",
            Severity::Critical => "💀",
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    if args.list_presets {
        list_preset_patterns();
        return Ok(());
    }
    
    let pattern = get_pattern(&args)?;
    let use_color = should_use_color(&args);
    
    if args.files.is_empty() {
        let input = read_stdin()?;
        process_input(&input, "<stdin>", &pattern, &args, use_color)?;
    } else {
        for file in &args.files {
            let input = fs::read_to_string(file).map_err(|e| {
                if !args.no_messages {
                    eprintln!("ai-grep: {}: {}", file.display(), e);
                }
                e
            });
            
            match input {
                Ok(content) => {
                    process_input(&content, &file.to_string_lossy(), &pattern, &args, use_color)?;
                }
                Err(_) => continue,
            }
        }
    }
    
    Ok(())
}

fn read_stdin() -> Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn get_pattern(args: &Args) -> Result<AnomalyPattern> {
    if args.hallucinations {
        Ok(AnomalyPattern::Preset(PresetPattern::Hallucinations))
    } else if args.code_issues {
        Ok(AnomalyPattern::Preset(PresetPattern::CodeIssues))
    } else if args.security {
        Ok(AnomalyPattern::Preset(PresetPattern::Security))
    } else if args.data_leakage {
        Ok(AnomalyPattern::Preset(PresetPattern::DataLeakage))
    } else if args.low_confidence {
        Ok(AnomalyPattern::Preset(PresetPattern::LowConfidence))
    } else if let Some(pattern) = &args.pattern {
        Ok(AnomalyPattern::Custom(pattern.clone()))
    } else {
        anyhow::bail!("No pattern specified. Use a custom pattern or one of the preset flags (--hallucinations, --code-issues, etc.)")
    }
}

fn should_use_color(args: &Args) -> bool {
    match args.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => atty::is(atty::Stream::Stdout),
    }
}

fn list_preset_patterns() {
    println!("Available preset patterns:");
    println!("  --hallucinations    AI model limitations and disclaimers");
    println!("  --code-issues       Development markers (TODO, FIXME, HACK, etc.)");
    println!("  --security          Security vulnerability patterns");
    println!("  --data-leakage      Training data or PII leakage indicators");
    println!("  --low-confidence    Uncertainty and hedging language");
    println!();
    println!("Examples:");
    println!("  ai-grep --hallucinations < ai_output.txt");
    println!("  ai-grep --code-issues --count src/**/*.rs");
    println!("  ai-grep --security --color=always *.py");
    println!("  ai-grep \"custom pattern\" file.txt");
}

#[derive(Debug)]
enum AnomalyPattern {
    Preset(PresetPattern),
    Custom(String),
}

#[derive(Debug)]
enum PresetPattern {
    Hallucinations,
    CodeIssues,
    Security,
    DataLeakage,
    LowConfidence,
}

fn process_input(input: &str, filename: &str, pattern: &AnomalyPattern, args: &Args, use_color: bool) -> Result<()> {
    let matches = find_matches(input, pattern, args)?;
    
    if args.files_with_matches {
        if !matches.is_empty() {
            println!("{}", filename);
        }
        return Ok(());
    }
    
    if args.count {
        let count = if args.invert_match {
            input.lines().count() - matches.len()
        } else {
            matches.len()
        };
        
        if args.files.len() > 1 {
            println!("{}:{}", filename, count);
        } else {
            println!("{}", count);
        }
        return Ok(());
    }
    
    display_matches(&matches, filename, args, use_color)?;
    Ok(())
}

fn find_matches(input: &str, pattern: &AnomalyPattern, args: &Args) -> Result<Vec<Match>> {
    let mut all_matches = Vec::new();
    
    match pattern {
        AnomalyPattern::Preset(preset) => {
            all_matches.extend(find_preset_matches(input, preset, args)?);
        }
        AnomalyPattern::Custom(pattern_str) => {
            all_matches.extend(find_custom_matches(input, pattern_str, args)?);
        }
    }
    
    // Apply invert match filter
    if args.invert_match {
        let matching_lines: std::collections::HashSet<usize> = all_matches
            .iter()
            .map(|m| m.line_number)
            .collect();
            
        all_matches = input
            .lines()
            .enumerate()
            .filter_map(|(i, line)| {
                let line_num = i + 1;
                if !matching_lines.contains(&line_num) {
                    Some(Match {
                        line_number: line_num,
                        content: line.to_string(),
                        matched_text: line.to_string(),
                        start_pos: 0,
                        end_pos: line.len(),
                        anomaly_type: AnomalyType::Custom,
                        severity: Severity::Low,
                    })
                } else {
                    None
                }
            })
            .collect();
    }
    
    Ok(all_matches)
}

fn find_preset_matches(input: &str, preset: &PresetPattern, args: &Args) -> Result<Vec<Match>> {
    match preset {
        PresetPattern::Hallucinations => find_hallucination_matches(input, args),
        PresetPattern::CodeIssues => find_code_issue_matches(input, args),
        PresetPattern::Security => find_security_matches(input, args),
        PresetPattern::DataLeakage => find_data_leakage_matches(input, args),
        PresetPattern::LowConfidence => find_low_confidence_matches(input, args),
    }
}

fn find_custom_matches(input: &str, pattern_str: &str, args: &Args) -> Result<Vec<Match>> {
    let regex_flags = if args.ignore_case { "(?i)" } else { "" };
    
    let pattern = if args.extended_regex || args.perl_regex {
        // For extended/perl regex, use the pattern as-is (with case flag)
        Regex::new(&format!("{}{}", regex_flags, pattern_str))?
    } else {
        // For basic grep, escape special characters
        Regex::new(&format!("{}{}", regex_flags, regex::escape(pattern_str)))?
    };
    
    let mut matches = Vec::new();
    
    for (line_num, line) in input.lines().enumerate() {
        for mat in pattern.find_iter(line) {
            matches.push(Match {
                line_number: line_num + 1,
                content: line.to_string(),
                matched_text: mat.as_str().to_string(),
                start_pos: mat.start(),
                end_pos: mat.end(),
                anomaly_type: AnomalyType::Custom,
                severity: Severity::Medium,
            });
        }
    }
    
    Ok(matches)
}

fn find_hallucination_matches(input: &str, args: &Args) -> Result<Vec<Match>> {
    let patterns = [
        ("knowledge_cutoff", vec![
            (r"(?i)\b(september 2021|knowledge cutoff|training data|cutoff date)\b", Severity::High),
            (r"(?i)\b(my last update|as of my last training)\b", Severity::High),
        ]),
        ("capability_disclaimer", vec![
            (r"(?i)\b(as an ai|i cannot|i don't have access|i'm not able to)\b", Severity::Critical),
            (r"(?i)\b(i don't have the ability|i cannot browse|cannot access the internet)\b", Severity::High),
        ]),
        ("uncertainty", vec![
            (r"(?i)\b(i'm not sure|i cannot verify|unconfirmed|i don't know)\b", Severity::Medium),
            (r"(?i)\b(i cannot confirm|i'm uncertain|unclear)\b", Severity::Medium),
        ]),
        ("browsing_limitation", vec![
            (r"(?i)\b(real-time|cannot browse|cannot access websites)\b", Severity::High),
            (r"(?i)\b(i don't have internet access|cannot search the web)\b", Severity::High),
        ]),
    ];
    
    let mut matches = Vec::new();
    
    for (marker_type, pattern_list) in &patterns {
        for (pattern_str, severity) in pattern_list {
            let regex = Regex::new(pattern_str)?;
            
            for (line_num, line) in input.lines().enumerate() {
                for mat in regex.find_iter(line) {
                    matches.push(Match {
                        line_number: line_num + 1,
                        content: line.to_string(),
                        matched_text: mat.as_str().to_string(),
                        start_pos: mat.start(),
                        end_pos: mat.end(),
                        anomaly_type: AnomalyType::Hallucination {
                            marker_type: marker_type.to_string(),
                        },
                        severity: severity.clone(),
                    });
                }
            }
        }
    }
    
    Ok(matches)
}

fn find_code_issue_matches(input: &str, args: &Args) -> Result<Vec<Match>> {
    let patterns = [
        (r"(?i)\b(todo|fixme|hack|bug|xxx|note|warn|warning)\b", "development_marker", Severity::Medium),
        (r"(?i)\b(deprecated|obsolete|legacy|remove|delete)\b", "deprecation", Severity::Low),
        (r"(?i)\b(temporary|temp|quick.?fix|workaround)\b", "temporary_code", Severity::Medium),
        (r"(?i)\b(unsafe|danger|critical|urgent)\b", "safety_concern", Severity::High),
        (r"(?i)//\s*(TODO|FIXME|HACK|BUG|XXX)", "comment_marker", Severity::Medium),
        (r"#\s*(TODO|FIXME|HACK|BUG|XXX)", "comment_marker", Severity::Medium),
    ];
    
    let mut matches = Vec::new();
    
    for (pattern_str, issue_type, severity) in &patterns {
        let regex = Regex::new(pattern_str)?;
        
        for (line_num, line) in input.lines().enumerate() {
            for mat in regex.find_iter(line) {
                matches.push(Match {
                    line_number: line_num + 1,
                    content: line.to_string(),
                    matched_text: mat.as_str().to_string(),
                    start_pos: mat.start(),
                    end_pos: mat.end(),
                    anomaly_type: AnomalyType::CodeIssue {
                        issue_type: issue_type.to_string(),
                    },
                    severity: severity.clone(),
                });
            }
        }
    }
    
    Ok(matches)
}

fn find_security_matches(input: &str, args: &Args) -> Result<Vec<Match>> {
    let patterns = [
        (r"(?i)\b(password|passwd|pwd)\s*=\s*[\x22\x27][^\x22\x27]*[\x22\x27]", "hardcoded_password", Severity::Critical),
        (r"(?i)\b(api.?key|secret.?key|private.?key)\s*[=:]\s*[\x22\x27][^\x22\x27]+[\x22\x27]", "hardcoded_secret", Severity::Critical),
        (r"(?i)\b(eval|exec|system|shell_exec)\s*\(", "code_injection", Severity::High),
        (r"(?i)\b(sql\s+injection|xss|csrf|rce|lfi|rfi)\b", "vulnerability_mention", Severity::High),
        (r"(?i)\b(unsafe|memcpy|strcpy|gets|scanf)\b", "unsafe_function", Severity::High),
        (r"(?i)\b(admin|root|administrator)\s*[/:]\s*\w+", "privileged_access", Severity::Medium),
    ];
    
    let mut matches = Vec::new();
    
    for (pattern_str, vuln_type, severity) in &patterns {
        let regex = Regex::new(pattern_str)?;
        
        for (line_num, line) in input.lines().enumerate() {
            for mat in regex.find_iter(line) {
                matches.push(Match {
                    line_number: line_num + 1,
                    content: line.to_string(),
                    matched_text: mat.as_str().to_string(),
                    start_pos: mat.start(),
                    end_pos: mat.end(),
                    anomaly_type: AnomalyType::Security {
                        vulnerability_type: vuln_type.to_string(),
                    },
                    severity: severity.clone(),
                });
            }
        }
    }
    
    Ok(matches)
}

fn find_data_leakage_matches(input: &str, args: &Args) -> Result<Vec<Match>> {
    let patterns = [
        (r"\b\d{3}-\d{2}-\d{4}\b", "ssn", Severity::Critical),
        (r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b", "credit_card", Severity::Critical),
        (r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b", "email", Severity::Medium),
        (r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b", "ip_address", Severity::Low),
        (r"(?i)\b(training.?data|dataset|corpus|model.?weights)\b", "training_reference", Severity::Medium),
        (r"(?i)\b(openai|anthropic|google|microsoft|meta)\s+(internal|confidential)", "internal_reference", Severity::High),
    ];
    
    let mut matches = Vec::new();
    
    for (pattern_str, leak_type, severity) in &patterns {
        let regex = Regex::new(pattern_str)?;
        
        for (line_num, line) in input.lines().enumerate() {
            for mat in regex.find_iter(line) {
                matches.push(Match {
                    line_number: line_num + 1,
                    content: line.to_string(),
                    matched_text: mat.as_str().to_string(),
                    start_pos: mat.start(),
                    end_pos: mat.end(),
                    anomaly_type: AnomalyType::DataLeakage {
                        leak_type: leak_type.to_string(),
                    },
                    severity: severity.clone(),
                });
            }
        }
    }
    
    Ok(matches)
}

fn find_low_confidence_matches(input: &str, args: &Args) -> Result<Vec<Match>> {
    let patterns = [
        (r"(?i)\b(might|maybe|perhaps|possibly|potentially|could be)\b", "uncertainty", Severity::Low),
        (r"(?i)\b(likely|probably|seems|appears|suggests)\b", "hedging", Severity::Low),
        (r"(?i)\b(i think|i believe|i guess|i assume)\b", "opinion", Severity::Medium),
        (r"(?i)\b(allegedly|supposedly|reportedly|apparently)\b", "hearsay", Severity::Medium),
        (r"(?i)\b(unverified|unconfirmed|uncertain|unclear)\b", "verification_issue", Severity::High),
    ];
    
    let mut matches = Vec::new();
    
    for (pattern_str, confidence_type, severity) in &patterns {
        let regex = Regex::new(pattern_str)?;
        
        for (line_num, line) in input.lines().enumerate() {
            for mat in regex.find_iter(line) {
                matches.push(Match {
                    line_number: line_num + 1,
                    content: line.to_string(),
                    matched_text: mat.as_str().to_string(),
                    start_pos: mat.start(),
                    end_pos: mat.end(),
                    anomaly_type: AnomalyType::LowConfidence {
                        confidence_marker: confidence_type.to_string(),
                    },
                    severity: severity.clone(),
                });
            }
        }
    }
    
    Ok(matches)
}

fn display_matches(matches: &[Match], filename: &str, args: &Args, use_color: bool) -> Result<()> {
    if args.format == "json" {
        return output_json(matches, filename, args);
    }
    
    for (i, m) in matches.iter().enumerate() {
        let line_prefix = if args.line_number {
            if args.files.len() > 1 {
                format!("{}:{}:", filename, m.line_number)
            } else {
                format!("{}:", m.line_number)
            }
        } else if args.files.len() > 1 {
            format!("{}:", filename)
        } else {
            String::new()
        };
        
        let content = if use_color {
            highlight_match(&m.content, &m.matched_text, m.start_pos, m.end_pos)
        } else {
            m.content.clone()
        };
        
        println!("{}{}", line_prefix, content);
        
        // Show anomaly details if requested
        if args.severity {
            println!("  {} Severity: {:?} ({:.2})", 
                     m.severity.to_emoji(), 
                     m.severity, 
                     m.severity.to_score());
        }
        
        match &m.anomaly_type {
            AnomalyType::Hallucination { marker_type } => {
                println!("  🚨 AI hallucination marker: {}", marker_type);
            }
            AnomalyType::CodeIssue { issue_type } => {
                println!("  🔧 Code issue: {}", issue_type);
            }
            AnomalyType::Security { vulnerability_type } => {
                println!("  🔒 Security concern: {}", vulnerability_type);
            }
            AnomalyType::DataLeakage { leak_type } => {
                println!("  📊 Data leakage: {}", leak_type);
            }
            AnomalyType::LowConfidence { confidence_marker } => {
                println!("  📉 Low confidence: {}", confidence_marker);
            }
            AnomalyType::Custom => {}
        }
        
        // Add context lines if requested
        if args.after_context > 0 || args.before_context > 0 {
            // Implementation for context lines would go here
        }
        
        if i < matches.len() - 1 && (args.severity || !matches!(m.anomaly_type, AnomalyType::Custom)) {
            println!();
        }
    }
    
    Ok(())
}

fn highlight_match(content: &str, matched_text: &str, start_pos: usize, end_pos: usize) -> String {
    let before = &content[..start_pos];
    let after = &content[end_pos..];
    format!("{}{}{}{}{}", 
            before, 
            "\x1b[31m\x1b[1m", // Red + bold
            matched_text, 
            "\x1b[0m", // Reset
            after)
}

fn output_json(matches: &[Match], filename: &str, args: &Args) -> Result<()> {
    let mut output = serde_json::Map::new();
    
    output.insert("file".to_string(), serde_json::Value::String(filename.to_string()));
    output.insert("match_count".to_string(), serde_json::Value::Number(serde_json::Number::from(matches.len())));
    
    let matches_json: Vec<serde_json::Value> = matches.iter().map(|m| {
        let mut match_obj = serde_json::Map::new();
        match_obj.insert("line_number".to_string(), serde_json::Value::Number(serde_json::Number::from(m.line_number)));
        match_obj.insert("content".to_string(), serde_json::Value::String(m.content.clone()));
        match_obj.insert("matched_text".to_string(), serde_json::Value::String(m.matched_text.clone()));
        match_obj.insert("start_pos".to_string(), serde_json::Value::Number(serde_json::Number::from(m.start_pos)));
        match_obj.insert("end_pos".to_string(), serde_json::Value::Number(serde_json::Number::from(m.end_pos)));
        match_obj.insert("severity".to_string(), serde_json::Value::String(format!("{:?}", m.severity)));
        match_obj.insert("severity_score".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(m.severity.to_score()).unwrap()));
        
        let anomaly_info = match &m.anomaly_type {
            AnomalyType::Hallucination { marker_type } => {
                let mut info = serde_json::Map::new();
                info.insert("type".to_string(), serde_json::Value::String("hallucination".to_string()));
                info.insert("marker_type".to_string(), serde_json::Value::String(marker_type.clone()));
                serde_json::Value::Object(info)
            }
            AnomalyType::CodeIssue { issue_type } => {
                let mut info = serde_json::Map::new();
                info.insert("type".to_string(), serde_json::Value::String("code_issue".to_string()));
                info.insert("issue_type".to_string(), serde_json::Value::String(issue_type.clone()));
                serde_json::Value::Object(info)
            }
            AnomalyType::Security { vulnerability_type } => {
                let mut info = serde_json::Map::new();
                info.insert("type".to_string(), serde_json::Value::String("security".to_string()));
                info.insert("vulnerability_type".to_string(), serde_json::Value::String(vulnerability_type.clone()));
                serde_json::Value::Object(info)
            }
            AnomalyType::DataLeakage { leak_type } => {
                let mut info = serde_json::Map::new();
                info.insert("type".to_string(), serde_json::Value::String("data_leakage".to_string()));
                info.insert("leak_type".to_string(), serde_json::Value::String(leak_type.clone()));
                serde_json::Value::Object(info)
            }
            AnomalyType::LowConfidence { confidence_marker } => {
                let mut info = serde_json::Map::new();
                info.insert("type".to_string(), serde_json::Value::String("low_confidence".to_string()));
                info.insert("confidence_marker".to_string(), serde_json::Value::String(confidence_marker.clone()));
                serde_json::Value::Object(info)
            }
            AnomalyType::Custom => {
                let mut info = serde_json::Map::new();
                info.insert("type".to_string(), serde_json::Value::String("custom".to_string()));
                serde_json::Value::Object(info)
            }
        };
        
        match_obj.insert("anomaly".to_string(), anomaly_info);
        serde_json::Value::Object(match_obj)
    }).collect();
    
    output.insert("matches".to_string(), serde_json::Value::Array(matches_json));
    
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

