use std::process::Command;
use crate::config::constants::audio;
use crate::domain::{AppError, AppResult, BluetoothCodec, BluetoothDevice};

pub fn probe_bluetooth_devices() -> Vec<BluetoothDevice> {
    let output = match Command::new("pactl").args(["list", "cards"]).output() {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => return Vec::new(),
    };

    if !output.contains(audio::BLUEZ_CARD_PREFIX) {
        return Vec::new();
    }

    output
        .split("Card #")
        .filter_map(parse_bluez_card)
        .collect()
}

fn parse_bluez_card(card_str: &str) -> Option<BluetoothDevice> {
    if !card_str.contains(audio::BLUEZ_CARD_PREFIX) {
        return None;
    }

    let mut name = "Bluetooth Device".to_string();
    let mut mac = String::new();
    let mut active_profile_name = None;
    let mut active_codec = BluetoothCodec::Other;
    let mut hw_vol_enabled = None;
    let mut card_name = None;

    for line in card_str.lines() {
        let trimmed = line.trim();
        if let Some(cname) = trimmed.strip_prefix("Name: ") {
            let m = cname.trim_start_matches(audio::BLUEZ_CARD_PREFIX).replace('_', ":");
            mac = m;
            card_name = Some(cname.to_string());
        } else if let Some(desc) = trimmed.strip_prefix("device.description = ") {
            name = desc.trim_matches('"').to_string();
        } else if let Some(prof) = trimmed.strip_prefix("Active Profile: ") {
            active_codec = BluetoothCodec::from(prof);
            active_profile_name = Some(prof.to_string());
        } else if let Some(val) = trimmed.strip_prefix("bluez5.enable-hw-volume = ") {
            let clean = val.trim_matches('"');
            hw_vol_enabled = Some(clean == "true");
        }
    }

    let battery_percentage = read_battery_percentage(&mac);

    Some(BluetoothDevice {
        mac_address: mac,
        name,
        is_connected: true,
        battery_percentage,
        card_name,
        active_profile_name,
        active_codec,
        hw_volume_enabled: hw_vol_enabled,
        autoswitch_profile_enabled: None,
    })
}

pub fn set_card_profile(card_name: &str, profile_name: &str) -> AppResult<()> {
    let status = Command::new("pactl")
        .args(["set-card-profile", card_name, profile_name])
        .status()
        .map_err(|e| AppError::CommandExecution {
            command: format!("pactl set-card-profile {} {}", card_name, profile_name),
            details: e.to_string(),
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::CommandExecution {
            command: format!("pactl set-card-profile {} {}", card_name, profile_name),
            details: format!("Código de saída: {:?}", status.code()),
        })
    }
}

pub fn set_default_sink(sink_name: &str) -> AppResult<()> {
    let _ = Command::new("pactl").args(["set-default-sink", sink_name]).status();
    Ok(())
}

pub fn move_all_streams_to_sink(sink_name: &str) -> AppResult<()> {
    if let Ok(output) = Command::new("pactl").args(["list", "sink-inputs", "short"]).output() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if let Some(id) = line.split_whitespace().next() {
                let _ = Command::new("pactl").args(["move-sink-input", id, sink_name]).status();
            }
        }
    }
    Ok(())
}

fn read_battery_percentage(mac: &str) -> Option<u8> {
    if mac.is_empty() {
        return None;
    }

    let output = Command::new("bluetoothctl").args(["info", mac]).output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        if line.contains("Battery Percentage:")
            && let (Some(start), Some(end)) = (line.find('('), line.find(')'))
        {
            return line[start + 1..end].parse::<u8>().ok();
        }
    }

    None
}
