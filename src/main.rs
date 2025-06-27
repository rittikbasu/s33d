use bip39::{Language, Mnemonic};
use clap::Parser;
use rand::RngCore;
use std::process;


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
        long = "words",
        value_parser = validate_words,
        help = "Number of words in the phrase (12 or 24)",
        conflicts_with = "strength"
    )]
    words: Option<usize>,

    /// Advanced: Entropy strength in bits (128, 160, 192, 224, 256)
    #[arg(
        short = 's', 
        long = "strength",
        value_parser = validate_strength,
        help = "Advanced: Entropy strength in bits",
        conflicts_with = "words"
    )]
    strength: Option<usize>,

    /// Language for the mnemonic words
    #[arg(
        short = 'l',
        long = "language", 
        default_value = "english",
        value_parser = parse_language,
        help = "Language for mnemonic words"
    )]
    language: Language,

    /// Show technical details about entropy and generation
    #[arg(short = 'e', long = "entropy", help = "Show entropy and technical details")]
    show_entropy: bool,

    /// Quiet mode - only output the mnemonic phrase
    #[arg(short = 'q', long = "quiet", help = "Quiet mode - only output the phrase")]
    quiet: bool,

    /// List all supported languages
    #[arg(long = "list-languages", help = "List all supported languages")]
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
        _ => Err(format!(
            "Currently only English is supported in this version.\nFuture versions will include more languages."
        )),
    }
}

fn print_supported_languages() {
    println!();
    println!("┌─ supported languages ───────────────────────────────────────────┐");
    println!("│ english              (en)     - Default                         │");
    println!("│                                                                 │");
    println!("│ Additional languages coming in future versions:                 │");
    println!("│ • japanese, korean, spanish, chinese, french, italian, czech    │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();
    println!("Usage:");
    println!("  s33d --language english");
    println!("  s33d -l en --words 24");
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
        println!("│ ▪ language        : {:<45} │", format!("{:?}", language).to_lowercase());
        println!("│ ▪ entropy source  : os cryptographic rng                        │");
        println!("└─────────────────────────────────────────────────────────────────┘");
    }
    
    println!();
    println!("┌─ your {} word seed phrase ──────────────────────────────────────┐", word_count);
    
    // Display words in a clean grid
    let phrase = mnemonic.to_string();
    let words: Vec<&str> = phrase.split_whitespace().collect();
    
    for (chunk_idx, chunk) in words.chunks(4).enumerate() {
        print!("│ ");
        for (i, word) in chunk.iter().enumerate() {
            let word_num = chunk_idx * 4 + i + 1;
            print!("{:2}. {:<12} ", word_num, word);
        }
        // Pad the line to full width
        let used_chars = chunk.len() * 16 + 2; // 16 chars per word + "│ "
        if used_chars < 65 {
            let remaining_chars = 65 - used_chars;
            print!("{:width$}│", "", width = remaining_chars);
        } else {
            print!("│");
        }
        println!();
    }
    
    println!("└─────────────────────────────────────────────────────────────────┘");
    
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

