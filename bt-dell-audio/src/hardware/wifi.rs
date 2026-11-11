use std::fs;
use std::process::Command;
use crate::domain::{FrequencyBand, WifiInfo};

/// Dynamically discovers the active wireless network interface name
/// by querying /sys/class/net/ for interfaces with a 'wireless' or 'phy80211' directory.
/// Adapts dynamically to wlan0, wlp2s0, wlo1, or any custom Linux network name.
pub fn find_active_wifi_interface() -> String {
    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let path = entry.path();
            if (path.join("wireless").exists() || path.join("phy80211").exists())
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
            {
                return name.to_string();
            }
        }
    }
    "wlan0".to_string()
}

pub fn probe_active_wifi() -> Option<WifiInfo> {
    let iface = find_active_wifi_interface();

    // nmcli tabular output without BSSID to prevent colon splitting
    let output = Command::new("nmcli")
        .args(["-t", "-m", "tabular", "-f", "IN-USE,SSID,CHAN,FREQ,RATE,SIGNAL,SECURITY", "dev", "wifi"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    text.lines().find_map(|line| parse_nmcli_line(line, &iface))
}

fn parse_nmcli_line(line: &str, iface: &str) -> Option<WifiInfo> {
    if !line.starts_with('*') {
        return None;
    }

    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() < 6 {
        return None;
    }

    let ssid = parts.get(1).unwrap_or(&"").to_string();
    let channel_str = parts.get(2).unwrap_or(&"0");
    let channel = channel_str.parse::<u32>().unwrap_or(0);

    let freq_str = parts.get(3).unwrap_or(&"0");
    let frequency_mhz = freq_str
        .split_whitespace()
        .next()
        .unwrap_or("0")
        .parse::<u32>()
        .unwrap_or(0);

    let bitrate = parts.get(4).unwrap_or(&"N/A").to_string();
    let signal_percent = parts.get(5).unwrap_or(&"0").parse::<u32>().unwrap_or(0);

    let band = if frequency_mhz > 3000 || channel > 14 {
        FrequencyBand::FiveGHz
    } else if frequency_mhz > 0 || channel > 0 {
        FrequencyBand::TwoPointFourGHz
    } else {
        FrequencyBand::Unknown
    };

    let powersave_enabled = check_powersave(iface);

    Some(WifiInfo {
        interface: iface.to_string(),
        ssid,
        band,
        channel,
        frequency_mhz,
        bitrate,
        signal_percent,
        powersave_enabled,
    })
}

fn check_powersave(interface: &str) -> bool {
    Command::new("iw")
        .args(["dev", interface, "get", "power_save"])
        .output()
        .map(|o| {
            let s = String::from_utf8_lossy(&o.stdout);
            s.to_lowercase().contains("on")
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nmcli_line_valid_5ghz() {
        let sample = "*:MiFibra-5G:36:5180 MHz:540 Mbit/s:78:WPA2";
        let info = parse_nmcli_line(sample, "wlan0").expect("Deve parsear linha 5GHz");
        assert_eq!(info.ssid, "MiFibra-5G");
        assert_eq!(info.channel, 36);
        assert_eq!(info.frequency_mhz, 5180);
        assert_eq!(info.band, FrequencyBand::FiveGHz);
        assert_eq!(info.signal_percent, 78);
        assert_eq!(info.interface, "wlan0");
    }

    #[test]
    fn test_parse_nmcli_line_not_in_use() {
        let sample = " :OutraRede:6:2437 MHz:54 Mbit/s:40:WPA2";
        assert!(parse_nmcli_line(sample, "wlan0").is_none());
    }
}
