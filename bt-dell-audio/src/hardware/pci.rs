use std::fs;
use crate::config::constants::{hardware, paths};

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

pub fn get_wifi_pci_identifier() -> String {
    format!("{} ({})", hardware::REALTEK_WIFI_PCI_ID, "Realtek RTL8821CE")
}

pub fn get_wifi_driver_name() -> String {
    hardware::REALTEK_WIFI_DRIVER_NAME.to_string()
}

fn parse_kernel_bool(s: &str) -> bool {
    s.eq_ignore_ascii_case(hardware::KERNEL_PARAM_TRUE) || s == "1"
}
