//! Centralized configuration constants for hardware identifiers, system paths,
//! timeouts, and audio defaults. No magic numbers or magic strings in business logic.

use std::time::Duration;

pub mod hardware {
    pub const REALTEK_VENDOR_ID: &str = "0bda";
    pub const REALTEK_BT_PRODUCT_ID: &str = "c829";
    pub const REALTEK_WIFI_PCI_ID: &str = "10ec:c821";
    pub const REALTEK_WIFI_DRIVER_NAME: &str = "rtw88_8821ce";

    pub const USB_POWER_CONTROL_ON: &str = "on";
    pub const KERNEL_PARAM_TRUE: &str = "Y";
}

pub mod paths {
    pub const SYS_USB_DEVICES: &str = "/sys/bus/usb/devices";
    pub const SYS_MODULE_BTUSB_AUTOSUSPEND: &str = "/sys/module/btusb/parameters/enable_autosuspend";
    pub const SYS_MODULE_RTW88_LPS_DEEP: &str = "/sys/module/rtw88_core/parameters/disable_lps_deep";
    pub const SYS_MODULE_RTW88_ASPM: &str = "/sys/module/rtw88_pci/parameters/disable_aspm";

    pub const OS_RELEASE_FILE: &str = "/etc/os-release";
    pub const PROC_VERSION_FILE: &str = "/proc/version";
    pub const PROC_SELF_STATUS_FILE: &str = "/proc/self/status";

    pub const MODPROBE_BTUSB_PATH: &str = "/etc/modprobe.d/btusb.conf";
    pub const MODPROBE_RTW88_PATH: &str = "/etc/modprobe.d/rtw88.conf";
    pub const UDEV_RULE_PATH: &str = "/etc/udev/rules.d/50-bluetooth-realtek.rules";

    pub const WIREPLUMBER_CONFIG_DIR_REL: &str = ".config/wireplumber/wireplumber.conf.d";
    pub const WIREPLUMBER_BT_TWEAKS_FILE: &str = "80-bluetooth-tweaks.conf";
    pub const WIREPLUMBER_BACKUP_SUFFIX: &str = ".bak";

    pub const SYSTEMD_USER_DIR_REL: &str = ".config/systemd/user";
    pub const SYSTEMD_SERVICE_FILE: &str = "bt-dell-audio.service";
}

pub mod audio {
    pub const CODEC_SBC: &str = "a2dp-sink-sbc";
    pub const CODEC_AAC: &str = "a2dp-sink";
    pub const CODEC_FALLBACK_ORDER: [&str; 2] = [CODEC_SBC, CODEC_AAC];

    pub const BLUEZ_CARD_PREFIX: &str = "bluez_card.";
    pub const BLUEZ_OUTPUT_PREFIX: &str = "bluez_output.";
}

pub mod timing {
    use super::Duration;
    pub const DAEMON_POLL_INTERVAL: Duration = Duration::from_secs(5);
    pub const WATCH_REFRESH_INTERVAL: Duration = Duration::from_secs(2);
    pub const SERVICE_RELOAD_DELAY: Duration = Duration::from_millis(500);
}
