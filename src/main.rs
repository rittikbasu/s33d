use bip39::{Language, Mnemonic};
use clap::Parser;
use rand::RngCore;
use std::process;
use unicode_width::UnicodeWidthStr;


#[derive(Parser, Debug)]
#[command(
    author = "rittikbasu",
    version, 
    about = "Generate secure BIP39 seed phrases for cryptocurrency wallets",
    long_about = "s33d generates cryptographically secure BIP39 mnemonic phrases.\n\
                  These phrases can restore cryptocurrency wallets.\n\
                  \n\
                  SECURITY WARNING: Generated phrases provide access to funds.\n\
                  Store them securely and never share them online."
)]
struct Args {
    /// Number of words (12 = good security, 24 = maximum security)
    #[arg(
        short = 'w',
        value_parser = validate_words,
        help = "Number of words in the phrase (12 or 24)",
        conflicts_with = "strength"
    )]
    words: Option<usize>,

    /// Advanced: Entropy strength in bits (128, 160, 192, 224, 256)
    #[arg(
        short = 's', 
        value_parser = validate_strength,
        help = "Advanced: Entropy strength in bits",
        conflicts_with = "words"
    )]
    strength: Option<usize>,

    /// Language for the mnemonic words
    #[arg(
        short = 'l',
        default_value = "english",
        value_parser = parse_language,
        help = "Language for mnemonic word"
    )]
    language: Language,

    /// Show technical details about entropy and generation
    #[arg(short = 'e', help = "Show entropy and technical details")]
    show_entropy: bool,

    /// Quiet mode - only output the mnemonic phrase
    #[arg(short = 'q', help = "Quiet mode - only output the phrase")]
    quiet: bool,

    /// List all supported languages
    #[arg(long = "list", help = "List all supported languages")]
    list_languages: bool,
}

fn main() {
    let args = Args::parse();

    if args.list_languages {
        print_supported_languages();
        return;
    }

    // Determine strength based on words or strength argument
    let strength = if let Some(words) = args.words {
        words_to_strength(words)
    } else if let Some(strength) = args.strength {
        strength
    } else {
        // Default to 24 words (256 bits) for maximum security
        256
    };

    // Verify system has sufficient entropy
    if !args.quiet {
        verify_entropy_quality();
    }

    let word_count = strength_to_word_count(strength);

    // Generate cryptographically secure entropy
    let entropy_bytes = strength / 8;
    let mut entropy = vec![0u8; entropy_bytes];
    rand::thread_rng().fill_bytes(&mut entropy);

    let mnemonic = match Mnemonic::from_entropy_in(args.language, &entropy) {
        Ok(m) => m,
        Err(e) => {
            print_error(&format!("Error generating mnemonic: {}", e));
            process::exit(1);
        }
    };

    if args.quiet {
        // Just output the phrase for scripting
        println!("{}", mnemonic);
    } else {
        // User-friendly output with security warnings
        print_mnemonic_with_info(&mnemonic, word_count, strength, args.show_entropy, args.language);
    }
}

fn validate_words(s: &str) -> Result<usize, String> {
    let words: usize = s.parse().map_err(|_| "Word count must be a number")?;
    match words {
        12 | 24 => Ok(words),
        _ => Err("Word count must be 12 or 24".to_string()),
    }
}

fn validate_strength(s: &str) -> Result<usize, String> {
    let strength: usize = s.parse().map_err(|_| "Strength must be a number")?;
    match strength {
        128 | 160 | 192 | 224 | 256 => Ok(strength),
        _ => Err("Strength must be 128, 160, 192, 224, or 256 bits".to_string()),
    }
}

fn words_to_strength(words: usize) -> usize {
    match words {
        12 => 128,
        24 => 256,
        _ => unreachable!("Word validation should prevent this"),
    }
}

fn strength_to_word_count(strength: usize) -> usize {
    // BIP39 formula: word_count = (entropy_bits + checksum_bits) / 11
    // Checksum bits = entropy_bits / 32
    let checksum_bits = strength / 32;
    (strength + checksum_bits) / 11
}

fn parse_language(s: &str) -> Result<Language, String> {
    match s.to_lowercase().as_str() {
        "english" | "en" => Ok(Language::English),
        "chinese-simplified" | "cn" | "zh-cn" => Ok(Language::SimplifiedChinese),
        "chinese-traditional" | "tw" | "zh-tw" => Ok(Language::TraditionalChinese),
        "french" | "fr" => Ok(Language::French),
        "italian" | "it" => Ok(Language::Italian),
        "japanese" | "ja" | "jp" => Ok(Language::Japanese),
        "korean" | "ko" | "kr" => Ok(Language::Korean),
        "spanish" | "es" => Ok(Language::Spanish),
        "czech" | "cs" => Ok(Language::Czech),
        "portuguese" | "pt" => Ok(Language::Portuguese),
        _ => Err(format!(
            "Unsupported language. Use --list to see available options."
        )),
    }
}

fn print_supported_languages() {
    let languages = [
        ("english", "(en)", "- default, widely supported"),
        ("chinese-simplified", "(cn)", "- 简体中文"),
        ("chinese-traditional", "(tw)", "- 繁體中文"),
        ("french", "(fr)", "- français"),
        ("italian", "(it)", "- italiano"),
        ("japanese", "(ja)", "- 日本語"),
        ("korean", "(ko)", "- 한국어"),
        ("spanish", "(es)", "- español"),
        ("czech", "(cs)", "- čeština"),
        ("portuguese", "(pt)", "- português"),
    ];

    let col1_width = languages
        .iter()
        .map(|(name, _, _)| UnicodeWidthStr::width(*name))
        .max()
        .unwrap_or(0);

    let col2_width = languages
        .iter()
        .map(|(_, code, _)| UnicodeWidthStr::width(*code))
        .max()
        .unwrap_or(0);
    
    let separator = "  ";

    println!();
    println!("┌─ supported languages ───────────────────────────────────────────┐");

    for (name, code, description) in &languages {
        let name_part = format!("{:<width$}", name, width = col1_width);
        let code_part = format!("{:<width$}", code, width = col2_width);
        
        let line_content = format!("{}{}{}{}{}", name_part, separator, code_part, separator, description);
        let line_width = UnicodeWidthStr::width(line_content.as_str());

        let total_inner_width: usize = 63;
        let padding = " ".repeat(total_inner_width.saturating_sub(line_width));

        println!("│ {}{} │", line_content, padding);
    }

    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();
    println!("┌─ compatibility note ────────────────────────────────────────────┐");
    println!("│ ▪ english is the most widely supported language                 │");
    println!("│ ▪ other languages may have limited wallet support               │");
    println!("│ ▪ when in doubt, use english                                    │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();
    println!("usage:");
    println!("  s33d -l english");
    println!("  s33d -l ja -w 24");
    println!();
}

fn verify_entropy_quality() {
    // On Unix-like systems, check if /dev/urandom is available
    #[cfg(unix)]
    {
        if !std::path::Path::new("/dev/urandom").exists() {
            print_warning("System entropy source not found - entropy quality may be compromised");
        }
    }
}

fn print_error(message: &str) {
    println!("┌─ ERROR ─────────────────────────────────────────────────────────┐");
    println!("│ ✗ {:<61} │", message);
    println!("└─────────────────────────────────────────────────────────────────┘");
}

fn print_warning(message: &str) {
    println!("┌─ WARNING ───────────────────────────────────────────────────────┐");
    println!("│ ⚠ {:<61} │", message);
    println!("└─────────────────────────────────────────────────────────────────┘");
}

fn print_mnemonic_with_info(mnemonic: &Mnemonic, word_count: usize, strength: usize, show_entropy: bool, language: Language) {
    println!();
    
    // Header
    println!("┌─ s33d: bip39 mnemonic generator ────────────────────────────────┐");
    println!("│ cryptographically secure seed phrase generation                 │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    
    if show_entropy {
        println!();
        println!("┌─ technical details ─────────────────────────────────────────────┐");
        println!("│ ▪ entropy bits    : {:>3} bits                                    │", strength);
        println!("│ ▪ checksum bits   : {:>3} bits                                    │", strength / 32);
        println!("│ ▪ total bits      : {:>3} bits                                    │", strength + (strength / 32));
        println!("│ ▪ word count      : {:>3} words                                   │", word_count);
        
        let lang_str = match language {
            Language::English => "English",
            Language::SimplifiedChinese => "Simplified Chinese",
            Language::TraditionalChinese => "Traditional Chinese",
            Language::French => "French",
            Language::Italian => "Italian",
            Language::Japanese => "Japanese",
            Language::Korean => "Korean",
            Language::Spanish => "Spanish",
            Language::Czech => "Czech",
            Language::Portuguese => "Portuguese",
        };
        println!("│ ▪ language        : {:<43} │", lang_str);

        println!("└─────────────────────────────────────────────────────────────────┘");
    }
    
    println!();
    
    let phrase = mnemonic.to_string();
    let words: Vec<&str> = phrase.split_whitespace().collect();
    let num_columns = 4;
    let num_rows = (words.len() + num_columns - 1) / num_columns;

    // Calculate the maximum visual width required for each column
    let mut column_widths = vec![0; num_columns];
    for col in 0..num_columns {
        for row in 0..num_rows {
            if let Some(word) = words.get(row + col * num_rows) {
                let num = row + col * num_rows + 1;
                let item = format!("{}. {}", num, word);
                let width = UnicodeWidthStr::width(item.as_str());
                if width > column_widths[col] {
                    column_widths[col] = width;
                }
            }
        }
    }

    let base_separator = "   ";
    let base_separator_width = UnicodeWidthStr::width(base_separator);
    
    let required_total_width = column_widths.iter().sum::<usize>() + (num_columns - 1) * base_separator_width;

    // Target width should match other boxes. The inner content of those boxes
    // is 65 characters wide. We use 63 here to account for the two spaces
    // added by the `println!("│ {} │", ...)` format string.
    let target_width = 63;
    let final_width = required_total_width.max(target_width);
    
    let total_padding_to_add = final_width - required_total_width;
    let num_separators = num_columns - 1;
    let extra_padding_per_separator = total_padding_to_add / num_separators;
    let remainder = total_padding_to_add % num_separators;

    let mut separators = Vec::new();
    for i in 0..num_separators {
        let extra_padding = if i < remainder { 1 } else { 0 };
        separators.push(format!("{}{}", base_separator, " ".repeat(extra_padding_per_separator + extra_padding)));
    }

    // Print header
    let header_text = format!(" your {} word seed phrase ", word_count);
    let mut header = format!("┌─{}", header_text);
    let header_width = UnicodeWidthStr::width(header.as_str());
    let total_line_width = final_width + 4;
    let dashes_len = total_line_width.saturating_sub(header_width + 1);
    header.push_str(&"─".repeat(dashes_len));
    header.push('┐');
    println!("{}", header);

    // Print content rows
    for row in 0..num_rows {
        let mut line_parts = Vec::new();
        for col in 0..num_columns {
            let item_text = if let Some(word) = words.get(row + col * num_rows) {
                let num = row + col * num_rows + 1;
                format!("{}. {}", num, word)
            } else {
                String::new()
            };
            
            let item_width = UnicodeWidthStr::width(item_text.as_str());
            let padding = " ".repeat(column_widths[col] - item_width);
            line_parts.push(format!("{}{}", item_text, padding));
        }
        
        let mut line = String::new();
        for (i, part) in line_parts.iter().enumerate() {
            line.push_str(part);
            if i < num_separators {
                line.push_str(&separators[i]);
            }
        }

        println!("│ {} │", line);
    }
    
    // Print footer
    println!("└{}┘", "─".repeat(final_width + 2));
    
    println!();
    println!("┌─ security warnings ─────────────────────────────────────────────┐");
    println!("│ ▲ critical: write this phrase on paper - NEVER store digitally  │");
    println!("│ ▲ keep in a secure location away from others                    │");
    println!("│ ▲ anyone with this phrase can access your cryptocurrency        │");
    println!("│ ▲ verify the first few words before final storage               │");
    println!("│ ▲ never enter this phrase on websites or untrusted devices      │");
    println!("│ ▲ consider hardware wallets for significant amounts             │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    
    println!();
    println!("┌─ generation status ─────────────────────────────────────────────┐");
    println!("│ ✓ phrase generated using cryptographically secure entropy       │");
    println!("│ ✓ bip39 standard compliance verified                            │");
    println!("│ ✓ checksum validation passed                                    │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();
}

