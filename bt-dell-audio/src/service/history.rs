use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use crate::domain::{sanitizer, AppError, AppResult, DiagnosticReport};

const MAX_REPORT_AGE: Duration = Duration::from_secs(7 * 24 * 3600); // 7 days retention
const MAX_REPORT_COUNT: usize = 5; // Keep maximum 5 recent snapshots

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSnapshot {
    pub filename: String,
    pub timestamp_epoch_secs: u64,
    pub date_formatted: String,
    pub os_name: String,
    pub kernel_version: String,
    pub issues_count: usize,
    pub has_critical: bool,
    pub report: DiagnosticReport,
}

fn get_reports_dir() -> AppResult<PathBuf> {
    let home = std::env::var("HOME").map_err(|_| AppError::EnvVarMissing("HOME".to_string()))?;
    let path = PathBuf::from(home).join(".local/state/bt-dell-audio/reports");
    fs::create_dir_all(&path).map_err(|e| AppError::Io {
        path: path.clone(),
        source: e,
    })?;
    Ok(path)
}

/// Saves an execution snapshot to disk with strict POSIX permissions (0600)
/// and triggers automatic garbage collection of expired reports.
pub fn save_snapshot(report: &DiagnosticReport, anonymize: bool) -> AppResult<PathBuf> {
    let reports_dir = get_reports_dir()?;

    let mut report_clone = report.clone();
    if anonymize {
        anonymize_report(&mut report_clone);
    }

    let now_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let filename = format!("diagnostic_{}.json", now_epoch);
    let target_path = reports_dir.join(&filename);

    let snapshot = DiagnosticSnapshot {
        filename,
        timestamp_epoch_secs: now_epoch,
        date_formatted: chrono_like_format(now_epoch),
        os_name: report_clone.os_name.clone(),
        kernel_version: report_clone.kernel_version.clone(),
        issues_count: report_clone.issues.len(),
        has_critical: report_clone.has_critical_issues(),
        report: report_clone,
    };

    let json_bytes = serde_json::to_string_pretty(&snapshot).map_err(|e| AppError::Serialization(e.to_string()))?;

    fs::write(&target_path, json_bytes).map_err(|e| AppError::Io {
        path: target_path.clone(),
        source: e,
    })?;

    // Hardened file security: 0600 (owner read-write only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(&target_path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o600);
            let _ = fs::set_permissions(&target_path, perms);
        }
    }

    // Run automatic garbage collection to purge stale snapshots
    let _ = prune_expired_reports();

    Ok(target_path)
}

/// Privacy Garbage Collector:
/// Purges any diagnostic snapshots older than 7 days and limits total stored snapshots to 5.
pub fn prune_expired_reports() -> usize {
    let reports_dir = match get_reports_dir() {
        Ok(d) => d,
        Err(_) => return 0,
    };

    let entries = match fs::read_dir(&reports_dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    let mut files: Vec<(PathBuf, SystemTime)> = Vec::new();
    let now = SystemTime::now();
    let mut deleted_count = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && path.extension().is_some_and(|ext| ext == "json")
            && let Ok(metadata) = fs::metadata(&path)
        {
            let mtime = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

            // Rule 1: Age-based garbage collection (> 7 days)
            if let Ok(age) = now.duration_since(mtime)
                && age > MAX_REPORT_AGE
                && fs::remove_file(&path).is_ok()
            {
                deleted_count += 1;
                continue;
            }

            files.push((path, mtime));
        }
    }

    // Rule 2: Count-based garbage collection (keep max 5 newest)
    files.sort_by(|a, b| b.1.cmp(&a.1)); // descending by modified time
    if files.len() > MAX_REPORT_COUNT {
        for (stale_file, _) in files.into_iter().skip(MAX_REPORT_COUNT) {
            if fs::remove_file(&stale_file).is_ok() {
                deleted_count += 1;
            }
        }
    }

    deleted_count
}

/// Retrieves all saved snapshots sorted from newest to oldest
pub fn list_history() -> Vec<DiagnosticSnapshot> {
    let reports_dir = match get_reports_dir() {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let entries = match fs::read_dir(&reports_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut snapshots = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && path.extension().is_some_and(|ext| ext == "json")
            && let Ok(content) = fs::read_to_string(&path)
            && let Ok(snap) = serde_json::from_str::<DiagnosticSnapshot>(&content)
        {
            snapshots.push(snap);
        }
    }

    snapshots.sort_by(|a, b| b.timestamp_epoch_secs.cmp(&a.timestamp_epoch_secs));
    snapshots
}

/// Immediate zero-footprint wipe: deletes all stored diagnostic reports
pub fn clean_all_history() -> AppResult<usize> {
    let reports_dir = get_reports_dir()?;
    let entries = fs::read_dir(&reports_dir).map_err(|e| AppError::Io {
        path: reports_dir.clone(),
        source: e,
    })?;

    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && fs::remove_file(&path).is_ok() {
            count += 1;
        }
    }

    Ok(count)
}

fn anonymize_report(report: &mut DiagnosticReport) {
    if let Some(wifi) = &mut report.wifi {
        wifi.ssid = sanitizer::mask_ssid(&wifi.ssid);
    }

    for dev in &mut report.bluetooth_devices {
        dev.mac_address = sanitizer::mask_mac_address(&dev.mac_address);
    }
}

fn chrono_like_format(epoch_secs: u64) -> String {
    let days_since_epoch = epoch_secs / 86400;
    let time_of_day = epoch_secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    format!("Epoch Day {} - {:02}:{:02}:{:02} UTC", days_since_epoch, hours, minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{FrequencyBand, HardwareInfo, PowerControlState, WifiInfo};

    fn create_dummy_report() -> DiagnosticReport {
        DiagnosticReport {
            os_name: "Test Linux".to_string(),
            kernel_version: "6.0.0-test".to_string(),
            hardware: HardwareInfo {
                wifi_pci_id: "10ec:c821".to_string(),
                wifi_name: "Test Wi-Fi".to_string(),
                wifi_driver: "test_driver".to_string(),
                bt_usb_id: "0bda:c829".to_string(),
                bt_name: "Test BT".to_string(),
                bt_usb_sysfs_path: None,
                bt_power_control: PowerControlState::On,
                bt_autosuspend_delay_ms: None,
                btusb_module_autosuspend: Some(false),
                rtw88_lps_deep_disabled: None,
                rtw88_aspm_disabled: None,
            },
            wifi: Some(WifiInfo {
                interface: "wlan0".to_string(),
                ssid: "SecretHomeNetwork".to_string(),
                band: FrequencyBand::FiveGHz,
                channel: 36,
                frequency_mhz: 5180,
                bitrate: "540 Mbit/s".to_string(),
                signal_percent: 80,
                powersave_enabled: false,
            }),
            bluetooth_devices: vec![],
            issues: vec![],
        }
    }

    #[test]
    fn test_anonymize_report() {
        let mut report = create_dummy_report();
        anonymize_report(&mut report);
        let masked = report.wifi.as_ref().unwrap().ssid.clone();
        assert_ne!(masked, "SecretHomeNetwork");
        assert!(masked.contains("***"));
    }
}
