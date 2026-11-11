use std::thread::sleep;
use crate::audio::{pipewire, wireplumber};
use crate::config::constants::{audio, timing};
use crate::domain::AppResult;

pub struct FixSummary {
    pub config_written_path: std::path::PathBuf,
    pub devices_adjusted: Vec<String>,
}

pub fn execute_audio_stabilization() -> AppResult<FixSummary> {
    // 1. Write optimized WirePlumber configuration
    let config_path = wireplumber::write_optimized_config()?;

    // 2. Restart WirePlumber service
    wireplumber::restart_wireplumber()?;

    // Allow daemon to rebind and enumerate bluetooth transport
    sleep(timing::SERVICE_RELOAD_DELAY);

    // 3. Negotiate stable profiles on active connected devices
    let mut adjusted = Vec::new();
    let devices = pipewire::probe_bluetooth_devices();

    for dev in devices {
        if let Some(card_name) = &dev.card_name {
            // Attempt standard SBC first, fallback to AAC if needed
            for &preferred_codec in &audio::CODEC_FALLBACK_ORDER {
                if pipewire::set_card_profile(card_name, preferred_codec).is_ok() {
                    adjusted.push(format!("{} -> {}", dev.name, preferred_codec));
                    break;
                }
            }

            // Restore stream routing to bluez sink
            let sink_name = format!("{}{}.1", audio::BLUEZ_OUTPUT_PREFIX, dev.mac_address.replace(':', "_"));
            let _ = pipewire::set_default_sink(&sink_name);
            let _ = pipewire::move_all_streams_to_sink(&sink_name);
        }
    }

    Ok(FixSummary {
        config_written_path: config_path,
        devices_adjusted: adjusted,
    })
}
