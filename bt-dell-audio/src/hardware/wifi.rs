use std::process::Command;
use crate::domain::{FrequencyBand, WifiInfo};

pub fn probe_active_wifi() -> Option<WifiInfo> {
    // nmcli tabular output without BSSID to prevent colon splitting
    let output = Command::new("nmcli")
        .args(["-t", "-m", "tabular", "-f", "IN-USE,SSID,CHAN,FREQ,RATE,SIGNAL,SECURITY", "dev", "wifi"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.starts_with('*') {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 6 {
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

                let powersave_enabled = check_powersave("wlan0");

                return Some(WifiInfo {
                    interface: "wlan0".to_string(),
                    ssid,
                    band,
                    channel,
                    frequency_mhz,
                    bitrate,
                    signal_percent,
                    powersave_enabled,
                });
            }
        }
    }

    None
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
