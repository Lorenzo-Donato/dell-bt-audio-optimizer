use std::fs;
use crate::config::constants::{hardware, paths};
use super::wifi;

pub fn read_rtw88_lps_deep_disabled() -> Option<bool> {
    fs::read_to_string(paths::SYS_MODULE_RTW88_LPS_DEEP)
        .ok()
        .map(|val| parse_kernel_bool(val.trim()))
}

pub fn read_rtw88_aspm_disabled() -> Option<bool> {
    fs::read_to_string(paths::SYS_MODULE_RTW88_ASPM)
        .ok()
        .map(|val| parse_kernel_bool(val.trim()))
}

/// Dynamically discovers the active Wi-Fi PCI identifier (vendor:device) from sysfs.
pub fn get_wifi_pci_identifier() -> String {
    let iface = wifi::find_active_wifi_interface();
    let dev_dir = format!("/sys/class/net/{}/device", iface);
    let vendor_file = format!("{}/vendor", dev_dir);
    let device_file = format!("{}/device", dev_dir);

    if let Ok(v) = fs::read_to_string(vendor_file)
        && let Ok(d) = fs::read_to_string(device_file)
    {
        let v_clean = v.trim().trim_start_matches("0x");
        let d_clean = d.trim().trim_start_matches("0x");
        let driver = get_wifi_driver_name();
        return format!("{}:{} ({})", v_clean, d_clean, driver);
    }

    format!("{} ({})", hardware::REALTEK_WIFI_PCI_ID, "Realtek RTL8821CE")
}

/// Dynamically discovers the active Wi-Fi kernel driver name from sysfs.
pub fn get_wifi_driver_name() -> String {
    let iface = wifi::find_active_wifi_interface();
    let driver_link = format!("/sys/class/net/{}/device/driver", iface);
    if let Ok(target) = fs::canonicalize(driver_link)
        && let Some(name) = target.file_name().and_then(|n| n.to_str())
    {
        return name.to_string();
    }
    hardware::REALTEK_WIFI_DRIVER_NAME.to_string()
}

fn parse_kernel_bool(s: &str) -> bool {
    s.eq_ignore_ascii_case(hardware::KERNEL_PARAM_TRUE) || s == "1"
}
