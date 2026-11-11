pub mod rule;
pub mod rules;

use crate::audio::pipewire;
use crate::domain::{DiagnosticReport, HardwareInfo};
use crate::hardware::{os, pci, usb, wifi};
use rule::{DiagnosticContext, DiagnosticEngine};
use rules::*;

pub fn run_full_diagnosis() -> DiagnosticReport {
    let os_name = os::read_os_pretty_name();
    let kernel_version = os::read_kernel_version();

    let usb_match = usb::find_realtek_bluetooth_device();
    let btusb_module_autosuspend = usb::read_btusb_module_autosuspend();
    let rtw88_lps_deep_disabled = pci::read_rtw88_lps_deep_disabled();
    let rtw88_aspm_disabled = pci::read_rtw88_aspm_disabled();

    let hardware = HardwareInfo {
        wifi_pci_id: pci::get_wifi_pci_identifier(),
        wifi_name: "Realtek RTL8821CE 802.11ac PCIe Wireless".to_string(),
        wifi_driver: pci::get_wifi_driver_name(),
        bt_usb_id: "0bda:c829 (Realtek Bluetooth Radio)".to_string(),
        bt_name: "Realtek Semiconductor Corp. Bluetooth Radio".to_string(),
        bt_usb_sysfs_path: usb_match.as_ref().map(|m| m.sysfs_path.clone()),
        bt_power_control: usb_match.as_ref().map(|m| m.power_control).unwrap_or(crate::domain::PowerControlState::Unknown),
        bt_autosuspend_delay_ms: usb_match.as_ref().and_then(|m| m.autosuspend_delay_ms),
        btusb_module_autosuspend,
        rtw88_lps_deep_disabled,
        rtw88_aspm_disabled,
    };

    let wifi = wifi::probe_active_wifi();
    let bluetooth_devices = pipewire::probe_bluetooth_devices();

    let mut engine = DiagnosticEngine::new();
    engine
        .register(UsbAutosuspendRule)
        .register(HardwareVolumeRule)
        .register(CodecBandwidthRule)
        .register(HeadsetAutoswitchRule)
        .register(WifiCoexistenceRule);

    let ctx = DiagnosticContext {
        hardware: &hardware,
        wifi: wifi.as_ref(),
        bluetooth_devices: &bluetooth_devices,
    };

    let issues = engine.analyze(&ctx);

    DiagnosticReport {
        os_name,
        kernel_version,
        hardware,
        wifi,
        bluetooth_devices,
        issues,
    }
}
