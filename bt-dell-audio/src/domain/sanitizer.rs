//! Privacy Guard & Data Sanitization utilities.
//! Ensures personal identifiers (MAC addresses, Wi-Fi SSIDs, home paths)
//! are never accidentally leaked in logs, issue reports, or public exports.

/// Masks the middle host bytes of a MAC address (e.g., 08:D8:33:4B:92:A1 -> 08:D8:33:**:**:A1).
/// Preserves the IEEE OUI vendor prefix for diagnostics while anonymizing the physical device.
pub fn mask_mac_address(mac: &str) -> String {
    let parts: Vec<&str> = mac.split(':').collect();
    if parts.len() == 6 {
        format!("{}:{}:{}:**:**:{}", parts[0], parts[1], parts[2], parts[5])
    } else if mac.is_empty() {
        "N/A".to_string()
    } else {
        "**:**:**:**:**:**".to_string()
    }
}

/// Masks a Wi-Fi SSID to protect private home/office network names.
/// (e.g., "MiFibra-5G" -> "MiF***5G", short SSIDs -> "***")
pub fn mask_ssid(ssid: &str) -> String {
    let chars: Vec<char> = ssid.chars().collect();
    let len = chars.len();

    if len <= 3 {
        "***".to_string()
    } else if len <= 6 {
        format!("{}***", chars[0])
    } else {
        let prefix: String = chars[..3].iter().collect();
        let suffix: String = chars[len - 2..].iter().collect();
        format!("{}***{}", prefix, suffix)
    }
}

/// Sanitizes absolute filesystem paths by replacing local usernames with a generic token.
/// (e.g., "/home/username/projects" -> "~/.../projects" or "/home/[USER]/projects")
pub fn sanitize_path_string(path_str: &str) -> String {
    if let Ok(home) = std::env::var("HOME")
        && path_str.starts_with(&home)
    {
        return path_str.replacen(&home, "~", 1);
    }

    // Generic Linux /home/<user> fallback replacement
    if path_str.starts_with("/home/") {
        let parts: Vec<&str> = path_str.split('/').collect();
        if parts.len() >= 3 {
            let user = parts[2];
            return path_str.replacen(&format!("/home/{}", user), "/home/[USER]", 1);
        }
    }

    path_str.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_mac_address() {
        assert_eq!(mask_mac_address("08:D8:33:4B:92:A1"), "08:D8:33:**:**:A1");
        assert_eq!(mask_mac_address(""), "N/A");
        assert_eq!(mask_mac_address("invalid_mac"), "**:**:**:**:**:**");
    }

    #[test]
    fn test_mask_ssid() {
        assert_eq!(mask_ssid("MiFibra-5G"), "MiF***5G");
        assert_eq!(mask_ssid("Home"), "H***");
        assert_eq!(mask_ssid("AB"), "***");
    }

    #[test]
    fn test_sanitize_path_string() {
        assert_eq!(
            sanitize_path_string("/home/john_doe/.config/wireplumber"),
            "/home/[USER]/.config/wireplumber"
        );
    }
}
