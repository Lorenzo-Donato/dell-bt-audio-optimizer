#![allow(dead_code)]

use std::io::{self, Write};
use colored::*;

/// Asks the user for confirmation via standard input.
/// If `auto_confirm` is true (e.g. via `--yes` or `-y`), skips prompt and returns true.
pub fn confirm(message: &str, auto_confirm: bool) -> bool {
    if auto_confirm {
        println!("{} {} (ignorado via flag --yes)", "[AUTO-CONFIRMADO]".green().bold(), message);
        return true;
    }

    print!("{} {} [s/N]: ", "❓".yellow(), message.bold());
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        let trimmed = input.trim().to_lowercase();
        trimmed == "s" || trimmed == "sim" || trimmed == "y" || trimmed == "yes"
    } else {
        false
    }
}
