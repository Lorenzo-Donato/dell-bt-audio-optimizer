use std::fs;
use crate::config::constants::paths;

pub fn read_os_pretty_name() -> String {
    if let Ok(content) = fs::read_to_string(paths::OS_RELEASE_FILE) {
        for line in content.lines() {
            if let Some(stripped) = line.strip_prefix("PRETTY_NAME=") {
                return stripped.trim_matches('"').to_string();
            }
        }
    }
    "Linux (Distribuição Desconhecida)".to_string()
}

pub fn read_kernel_version() -> String {
    if let Ok(content) = fs::read_to_string(paths::PROC_VERSION_FILE) {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 3 {
            return format!("{} {}", parts[0], parts[2]);
        }
    }
    "Kernel Desconhecido".to_string()
}
