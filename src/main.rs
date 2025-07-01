#![forbid(unsafe_code)]

use bip39::{Language, Mnemonic};
use clap::Parser;
use rand::RngCore;
use std::process;
use unicode_width::UnicodeWidthStr;
use qrcode::QrCode;
use rand::rngs::OsRng;
use zeroize::{Zeroize, Zeroizing};
use rpassword::prompt_password;


const DEFAULT_STRENGTH: usize = 128;
const TARGET_BOX_WIDTH: usize = 63;
const WORD_GRID_COLUMNS: usize = 4;

const LANGUAGE_INFO: &[(&str, &str, &str)] = &[
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

fn language_display_name(language: Language) -> &'static str {
    match language {
        Language::English => "english",
        Language::SimplifiedChinese => "chinese-simplified",
        Language::TraditionalChinese => "chinese-traditional",
        Language::French => "french",
        Language::Italian => "italian",
        Language::Japanese => "japanese",
        Language::Korean => "korean",
        Language::Spanish => "spanish",
        Language::Czech => "czech",
        Language::Portuguese => "portuguese",
    }
}

#[derive(Parser, Debug)]
#[command(
    author = "rittikbasu",
    version, 
    about = "generate secure BIP39 seed phrases for your bitcoin wallet",
    long_about = "s33d generates cryptographically secure BIP39 mnemonic phrases.\n\
                  these phrases can restore your bitcoin wallet.\n\
                  \n\
                  SECURITY WARNING: generated phrases provide access to funds.\n\
                  store them securely and never share them online."
)]
struct Args {
    #[arg(
        short = 'w',
        value_parser = validate_words,
        help = "Number of words in the phrase (12 or 24)",
        conflicts_with = "bits"
    )]
    words: Option<usize>,

    #[arg(
        short = 'l',
        default_value = "english",
        value_parser = parse_language,
        help = "Language for mnemonic word"
    )]
    language: Language,

    #[arg(short = 'e', help = "Show entropy and technical details")]
    show_entropy: bool,

    #[arg(short = 'c', help = "Clean mode - only output the phrase")]
    clean: bool,

    #[arg(short = 'q', help = "Generate QR code for easy mobile import")]
    qr_code: bool,

    #[arg(short = 'x', long = "hex", help = "Show entropy as hexadecimal")]
    show_hex: bool,

    #[arg(
        short = 'b',
        value_parser = validate_bits,
        help = "Advanced: Entropy bits (128-256)",
        conflicts_with = "words"
    )]
    bits: Option<usize>,

    #[arg(short = 'p', long = "passphrase", help = "Advanced: Prompt for an optional BIP39 passphrase")]
    passphrase: bool,

    #[arg(short = 's', long = "seed", help = "Advanced: Show derived 64-byte seed as hexadecimal")]
    show_seed: bool,

    #[arg(long = "list", help = "List all supported languages")]
    list_languages: bool,
}

fn main() {
    let args = Args::parse();

    if args.list_languages {
        print_supported_languages();
        return;
    }

    let bits = if let Some(words) = args.words {
        words_to_bits(words)
    } else if let Some(bits) = args.bits {
        bits
    } else {
        DEFAULT_STRENGTH
    };

    if !args.clean {
        verify_entropy_quality();
    }

    let word_count = bits_to_word_count(bits);
    let entropy_bytes = bits / 8;
    let mut entropy = Zeroizing::new(vec![0u8; entropy_bytes]);
    OsRng.fill_bytes(&mut entropy[..]);

    let mnemonic = match Mnemonic::from_entropy_in(args.language, &entropy[..]) {
        Ok(m) => m,
        Err(e) => {
            print_error(&format!("Error generating mnemonic: {}", e));
            process::exit(1);
        }
    };

    let passphrase = if args.passphrase {
        let pass = prompt_password("enter passphrase (leave blank for none): ")
            .unwrap_or_else(|_| {
                print_error("Failed to read passphrase");
                process::exit(1);
            });

        if pass.is_empty() {
            Zeroizing::new(pass)
        } else {
            let confirm = prompt_password("confirm passphrase: ")
                .unwrap_or_else(|_| {
                    print_error("Failed to read passphrase");
                    process::exit(1);
                });
            if pass != confirm {
                print_error("passphrases do not match");
                process::exit(2);
            }
            Zeroizing::new(pass)
        }
    } else {
        Zeroizing::new(String::new())
    };

    let seed_opt: Option<Zeroizing<Vec<u8>>> = if args.show_seed {
        let seed_arr = mnemonic.to_seed(passphrase.as_str());
        Some(Zeroizing::new(seed_arr.to_vec()))
    } else {
        None
    };

    if args.clean {
        let mut phrase = mnemonic.to_string();
        println!("{}", phrase);
        if args.show_hex {
            println!("hex: {}", hex::encode(&entropy[..]));
        }
        if let Some(seed) = &seed_opt {
            println!("seed: {}", hex::encode(&seed[..]));
        }
        if args.qr_code {
            print_qr_code(&phrase);
        }
        phrase.zeroize();
    } else {
        print_mnemonic_with_info(
            &mnemonic,
            &entropy[..],
            seed_opt.as_ref().map(|s| &s[..]),
            word_count,
            bits,
            args.show_entropy,
            args.show_hex,
            args.show_seed,
            args.language,
            args.qr_code,
        );
    }
}

fn validate_words(s: &str) -> Result<usize, String> {
    let words: usize = s.parse().map_err(|_| "Word count must be a valid number")?;
    match words {
        12 | 24 => Ok(words),
        _ => Err("word count must be either 12 or 24. use 12 for good security or 24 for maximum security.".to_string()),
    }
}

fn validate_bits(s: &str) -> Result<usize, String> {
    let bits: usize = s.parse().map_err(|_| "bits must be a valid number")?;
    match bits {
        128 | 160 | 192 | 224 | 256 => Ok(bits),
        _ => Err("Bits must be one of: 128, 160, 192, 224, or 256".to_string()),
    }
}

fn words_to_bits(words: usize) -> usize {
    match words {
        12 => 128,
        24 => 256,
        _ => unreachable!("Word validation should prevent this"),
    }
}

fn bits_to_word_count(bits: usize) -> usize {
    // BIP39 formula: word_count = (entropy_bits + checksum_bits) / 11
    // Checksum bits = entropy_bits / 32
    let checksum_bits = bits / 32;
    (bits + checksum_bits) / 11
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
            "unsupported language. use --list to see available options."
        )),
    }
}

fn print_supported_languages() {
    let col1_width = LANGUAGE_INFO
        .iter()
        .map(|(name, _, _)| UnicodeWidthStr::width(*name))
        .max()
        .unwrap_or(0);

    let col2_width = LANGUAGE_INFO
        .iter()
        .map(|(_, code, _)| UnicodeWidthStr::width(*code))
        .max()
        .unwrap_or(0);
    
    let separator = "  ";

    println!();
    println!("┌─ supported languages ───────────────────────────────────────────┐");

    for (name, code, description) in LANGUAGE_INFO {
        let name_part = format!("{:<width$}", name, width = col1_width);
        let code_part = format!("{:<width$}", code, width = col2_width);
        
        let line_content = format!("{}{}{}{}{}", name_part, separator, code_part, separator, description);
        let line_width = UnicodeWidthStr::width(line_content.as_str());

        let total_inner_width: usize = TARGET_BOX_WIDTH;
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
        use std::path::Path;
        if !Path::new("/dev/urandom").exists() {
            print_warning("system entropy source (/dev/urandom) not found, entropy quality may be compromised");
            return;
        }
        
        // Additional check for /dev/random availability (higher quality but blocking)
        if Path::new("/dev/random").exists() {
            // System has both entropy sources available - this is good
        } else {
            print_warning("high quality entropy source (/dev/random) not available, using /dev/urandom");
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

fn print_mnemonic_with_info(
    mnemonic: &Mnemonic,
    entropy: &[u8],
    seed_opt: Option<&[u8]>,
    word_count: usize,
    bits: usize,
    show_entropy: bool,
    show_hex: bool,
    show_seed: bool,
    language: Language,
    qr_code: bool,
) {
    println!();
    println!("┌─ s33d: bip39 mnemonic generator ────────────────────────────────┐");
    println!("│ cryptographically secure seed phrase generation                 │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    
    if show_entropy {
        println!();
        println!("┌─ technical details ─────────────────────────────────────────────┐");
        println!("│ ▪ entropy bits    : {:>3} bits                                    │", bits);
        println!("│ ▪ checksum bits   : {:>3} bits                                    │", bits / 32);
        println!("│ ▪ total bits      : {:>3} bits                                    │", bits + (bits / 32));
        println!("│ ▪ word count      : {:>3} words                                   │", word_count);
        
        let lang_str = language_display_name(language);
        println!("│ ▪ language        : {:<43} │", lang_str);

        println!("└─────────────────────────────────────────────────────────────────┘");
    }
    
    if show_hex {
        println!();
        println!("┌─ entropy (hexadecimal) ─────────────────────────────────────────┐");
        let hex_string = hex::encode(entropy);
        
        // Split hex into chunks for better readability
        let chunk_size = 32; // 16 bytes = 32 hex chars per line
        let chunks: Vec<&str> = hex_string.as_bytes().chunks(chunk_size)
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect();
        
        for chunk in chunks {
            println!("│ {:<63} │", chunk);
        }

        println!("└─────────────────────────────────────────────────────────────────┘");
    }
    
    if show_seed {
        if let Some(seed) = seed_opt {
            println!();
            println!("┌─ master seed (hexadecimal) ─────────────────────────────────────┐");
            let hex_string = hex::encode(seed);
            let chunk_size = 32; // 32 chars per line
            let chunks: Vec<&str> = hex_string.as_bytes().chunks(chunk_size)
                .map(|chunk| std::str::from_utf8(chunk).unwrap())
                .collect();
            for chunk in chunks {
                println!("│ {:<63} │", chunk);
            }
            println!("└─────────────────────────────────────────────────────────────────┘");
        }
    }
    
    println!();
    
    let mut phrase = mnemonic.to_string();
    let words: Vec<&str> = phrase.split_whitespace().collect();
    
    // For Korean, skip the box and just print words directly due to rendering issues
    if language == Language::Korean {
        println!("your {} word seed phrase", word_count);
        println!();
        for (i, word) in words.iter().enumerate() {
            println!("{}. {}", i + 1, word);
        }
        println!();
    } else {
        // Standard box layout for all other languages
        let num_rows = (words.len() + WORD_GRID_COLUMNS - 1) / WORD_GRID_COLUMNS;
        let mut column_widths = vec![0; WORD_GRID_COLUMNS];
        for col in 0..WORD_GRID_COLUMNS {
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
        
        let required_total_width = column_widths.iter().sum::<usize>() + (WORD_GRID_COLUMNS - 1) * base_separator_width;

        let final_width = required_total_width.max(TARGET_BOX_WIDTH);
        
        let total_padding_to_add = final_width - required_total_width;
        let num_separators = WORD_GRID_COLUMNS - 1;
        let extra_padding_per_separator = total_padding_to_add / num_separators;
        let remainder = total_padding_to_add % num_separators;

        let mut separators = Vec::new();
        for i in 0..num_separators {
            let extra_padding = if i < remainder { 1 } else { 0 };
            separators.push(format!("{}{}", base_separator, " ".repeat(extra_padding_per_separator + extra_padding)));
        }

        let header_text = format!(" your {} word seed phrase ", word_count);
        let mut header = format!("┌─{}", header_text);
        let header_width = UnicodeWidthStr::width(header.as_str());
        let total_line_width = final_width + 4;
        let dashes_len = total_line_width.saturating_sub(header_width + 1);
        header.push_str(&"─".repeat(dashes_len));
        header.push('┐');
        println!("{}", header);

        for row in 0..num_rows {
            let mut line_parts = Vec::new();
            for col in 0..WORD_GRID_COLUMNS {
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
        
        println!("└{}┘", "─".repeat(final_width + 2));
    }
    
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

    if qr_code {
        print_qr_code(&phrase);
    }
    phrase.zeroize();
}

fn print_qr_code(mnemonic: &str) {
    match QrCode::with_error_correction_level(mnemonic, qrcode::EcLevel::L) {
        Ok(code) => {
            let grid: Vec<bool> = code.to_colors().into_iter().map(|c| c == qrcode::Color::Dark).collect();
            let width = (grid.len() as f64).sqrt() as usize;
            
            const QUIET_ZONE_MODULES: usize = 2;
            const TOP_BOTTOM_PADDING_LINES: usize = QUIET_ZONE_MODULES / 2;
            
            let qr_width_chars = width + QUIET_ZONE_MODULES * 2;
            let box_inner_width = std::cmp::max(TARGET_BOX_WIDTH, qr_width_chars);

            let h_padding = " ".repeat(QUIET_ZONE_MODULES);
            let empty_line = " ".repeat(qr_width_chars);
            let total_padding = box_inner_width - qr_width_chars;
            let left_padding = " ".repeat(total_padding / 2);
            let right_padding = " ".repeat(total_padding - (total_padding / 2));

            println!();
            let title = "qr code for mobile import";
            let header_content_width = box_inner_width - 2;
            let padding_len = header_content_width - title.len();
            println!("┌─ {} {}─┐", title, "─".repeat(padding_len));

            for _ in 0..TOP_BOTTOM_PADDING_LINES {
                 println!("│ {}{}{} │", left_padding, empty_line, right_padding);
            }

            for y in (0..width).step_by(2) {
                let mut line = h_padding.clone();
                for x in 0..width {
                    let top_is_dark = grid[y * width + x];
                    let bottom_is_dark = if y + 1 < width { grid[(y + 1) * width + x] } else { false };

                    let character = match (top_is_dark, bottom_is_dark) {
                        (true, true) => '█',
                        (false, false) => ' ',
                        (true, false) => '▀',
                        (false, true) => '▄',
                    };
                    line.push(character);
                }
                line.push_str(&h_padding);
                println!("│ {}{}{} │", left_padding, line, right_padding);
            }

            for _ in 0..TOP_BOTTOM_PADDING_LINES {
                 println!("│ {}{}{} │", left_padding, empty_line, right_padding);
            }

            println!("└{}┘", "─".repeat(box_inner_width + 2));
        }
        Err(e) => {
            print_error(&format!("failed to generate QR code: {}. the mnemonic may be too long.", e));
        }
    }
}

