use std::fs;
use std::path::{Path, PathBuf};
use crate::config::constants::{hardware, paths};
use crate::domain::{AppError, AppResult, PowerControlState};

pub struct UsbDeviceMatch {
    pub sysfs_path: PathBuf,
    pub power_control: PowerControlState,
    pub autosuspend_delay_ms: Option<u32>,
}

pub fn find_realtek_bluetooth_device() -> Option<UsbDeviceMatch> {
    let entries = fs::read_dir(paths::SYS_USB_DEVICES).ok()?;

    for entry in entries.flatten() {
        let dev_path = entry.path();
        let vendor_path = dev_path.join("idVendor");
        let product_path = dev_path.join("idProduct");

        if vendor_path.exists() && product_path.exists() {
            let vendor = fs::read_to_string(&vendor_path).unwrap_or_default().trim().to_string();
            let product = fs::read_to_string(&product_path).unwrap_or_default().trim().to_string();

            if vendor == hardware::REALTEK_VENDOR_ID && product == hardware::REALTEK_BT_PRODUCT_ID {
                let power_control = read_power_control(&dev_path);
                let autosuspend_delay_ms = read_autosuspend_delay(&dev_path);

                return Some(UsbDeviceMatch {
                    sysfs_path: dev_path,
                    power_control,
                    autosuspend_delay_ms,
                });
            }
        }
    }

    None
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
