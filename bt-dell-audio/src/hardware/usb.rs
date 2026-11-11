use std::fs;
use std::path::{Path, PathBuf};
use crate::config::constants::{hardware, paths};
use crate::domain::{AppError, AppResult, PowerControlState};

#[derive(Debug, Clone)]
pub struct UsbDeviceMatch {
    pub vendor_id: String,
    pub product_id: String,
    #[allow(dead_code)]
    pub product_name: String,
    pub sysfs_path: PathBuf,
    pub power_control: PowerControlState,
    pub autosuspend_delay_ms: Option<u32>,
}

/// Dynamically discovers any connected Bluetooth USB controller by inspecting
/// the Linux kernel sysfs subsystem (/sys/class/bluetooth/ and /sys/bus/usb/devices).
/// Adapts dynamically to Realtek, Intel, MediaTek, Broadcom, or generic adapters.
pub fn find_bluetooth_usb_device() -> Option<UsbDeviceMatch> {
    // 1. Primary Strategy: Traverse /sys/class/bluetooth/hci* to active USB parent
    if let Ok(entries) = fs::read_dir("/sys/class/bluetooth") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str())
                && name.starts_with("hci")
                && !name.contains(':')
                && let Ok(device_link) = fs::canonicalize(path.join("device"))
            {
                let mut current = Some(device_link.as_path());
                while let Some(p) = current {
                    let vendor_f = p.join("idVendor");
                    let product_f = p.join("idProduct");
                    if vendor_f.exists() && product_f.exists() {
                        let vendor = fs::read_to_string(&vendor_f).unwrap_or_default().trim().to_string();
                        let product = fs::read_to_string(&product_f).unwrap_or_default().trim().to_string();
                        let prod_name = fs::read_to_string(p.join("product"))
                            .unwrap_or_else(|_| "Bluetooth Radio".to_string())
                            .trim()
                            .to_string();

                        let power_control = read_power_control(p);
                        let autosuspend_delay_ms = read_autosuspend_delay(p);

                        return Some(UsbDeviceMatch {
                            vendor_id: vendor,
                            product_id: product,
                            product_name: prod_name,
                            sysfs_path: p.to_path_buf(),
                            power_control,
                            autosuspend_delay_ms,
                        });
                    }
                    current = p.parent();
                }
            }
        }
    }

    // 2. Secondary Strategy: Scan /sys/bus/usb/devices for USB Wireless Controller class (e0:01:01)
    if let Ok(entries) = fs::read_dir(paths::SYS_USB_DEVICES) {
        for entry in entries.flatten() {
            let dev_path = entry.path();
            let vendor_path = dev_path.join("idVendor");
            let product_path = dev_path.join("idProduct");

            if vendor_path.exists() && product_path.exists() {
                let vendor = fs::read_to_string(&vendor_path).unwrap_or_default().trim().to_string();
                let product = fs::read_to_string(&product_path).unwrap_or_default().trim().to_string();

                if is_bluetooth_usb_interface(&dev_path) || vendor == hardware::REALTEK_VENDOR_ID {
                    let prod_name = fs::read_to_string(dev_path.join("product"))
                        .unwrap_or_else(|_| "Bluetooth Radio".to_string())
                        .trim()
                        .to_string();

                    let power_control = read_power_control(&dev_path);
                    let autosuspend_delay_ms = read_autosuspend_delay(&dev_path);

                    return Some(UsbDeviceMatch {
                        vendor_id: vendor,
                        product_id: product,
                        product_name: prod_name,
                        sysfs_path: dev_path,
                        power_control,
                        autosuspend_delay_ms,
                    });
                }
            }
        }
    }

    None
}

/// Backwards-compatible alias for existing callers
pub fn find_realtek_bluetooth_device() -> Option<UsbDeviceMatch> {
    find_bluetooth_usb_device()
}

pub fn read_btusb_module_autosuspend() -> Option<bool> {
    fs::read_to_string(paths::SYS_MODULE_BTUSB_AUTOSUSPEND)
        .ok()
        .map(|s| s.trim().eq_ignore_ascii_case(hardware::KERNEL_PARAM_TRUE) || s.trim() == "1")
}

pub fn set_usb_power_control_on(dev_path: &Path) -> AppResult<()> {
    let pctrl_file = dev_path.join("power/control");
    if pctrl_file.exists() {
        fs::write(&pctrl_file, hardware::USB_POWER_CONTROL_ON).map_err(|e| AppError::Io {
            path: pctrl_file,
            source: e,
        })?;
    }
    Ok(())
}

fn is_bluetooth_usb_interface(dev_path: &Path) -> bool {
    // Check if any sub-interface matches USB class e0 (wireless), subclass 01, protocol 01 (Bluetooth)
    if let Ok(entries) = fs::read_dir(dev_path) {
        for entry in entries.flatten() {
            let class_f = entry.path().join("bInterfaceClass");
            let sub_f = entry.path().join("bInterfaceSubClass");
            let proto_f = entry.path().join("bInterfaceProtocol");

            if class_f.exists()
                && let Ok(c) = fs::read_to_string(class_f)
                && c.trim() == "e0"
                && let Ok(s) = fs::read_to_string(sub_f)
                && s.trim() == "01"
                && let Ok(p) = fs::read_to_string(proto_f)
                && p.trim() == "01"
            {
                return true;
            }
        }
    }
    false
}

fn read_power_control(dev_path: &Path) -> PowerControlState {
    let pctrl = dev_path.join("power/control");
    if let Ok(content) = fs::read_to_string(pctrl) {
        PowerControlState::from(content.trim())
    } else {
        PowerControlState::Unknown
    }
}

fn read_autosuspend_delay(dev_path: &Path) -> Option<u32> {
    let delay_file = dev_path.join("power/autosuspend_delay_ms");
    fs::read_to_string(delay_file)
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
}
