use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use super::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub wifi_pci_id: String,
    pub wifi_name: String,
    pub wifi_driver: String,
    pub bt_usb_id: String,
    pub bt_name: String,
    pub bt_usb_sysfs_path: Option<PathBuf>,
    pub bt_power_control: PowerControlState,
    pub bt_autosuspend_delay_ms: Option<u32>,
    pub btusb_module_autosuspend: Option<bool>,
    pub rtw88_lps_deep_disabled: Option<bool>,
    pub rtw88_aspm_disabled: Option<bool>,
}

impl HardwareInfo {
    pub fn is_bluetooth_power_saving(&self) -> bool {
        self.bt_power_control.is_power_saving_active() || self.btusb_module_autosuspend == Some(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiInfo {
    pub interface: String,
    pub ssid: String,
    pub band: FrequencyBand,
    pub channel: u32,
    pub frequency_mhz: u32,
    pub bitrate: String,
    pub signal_percent: u32,
    pub powersave_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothDevice {
    pub mac_address: String,
    pub name: String,
    pub is_connected: bool,
    pub battery_percentage: Option<u8>,
    pub card_name: Option<String>,
    pub active_profile_name: Option<String>,
    pub active_codec: BluetoothCodec,
    pub hw_volume_enabled: Option<bool>,
    pub autoswitch_profile_enabled: Option<bool>,
}

impl BluetoothDevice {
    pub fn has_hardware_volume_risk(&self) -> bool {
        self.hw_volume_enabled == Some(true)
    }

    pub fn has_codec_choke_risk(&self) -> bool {
        self.active_codec.is_high_bandwidth_risk()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueItem {
    pub severity: IssueSeverity,
    pub title: String,
    pub description: String,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub os_name: String,
    pub kernel_version: String,
    pub hardware: HardwareInfo,
    pub wifi: Option<WifiInfo>,
    pub bluetooth_devices: Vec<BluetoothDevice>,
    pub issues: Vec<IssueItem>,
}

impl DiagnosticReport {
    pub fn has_critical_issues(&self) -> bool {
        self.issues.iter().any(|i| i.severity == IssueSeverity::Critical)
    }
}
