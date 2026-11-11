use std::fs;
use std::process::Command;
use crate::config::constants::{hardware, paths};
use crate::config::templates::{MODPROBE_BTUSB_CONF, MODPROBE_RTW88_CONF, UDEV_BLUETOOTH_RULES};
use crate::domain::{AppError, AppResult};
use crate::hardware::{os, usb};

pub fn apply_system_configurations() -> AppResult<()> {
    if !os::is_running_as_root() {
        return Err(AppError::PermissionDenied);
    }

    // 1. Write modprobe files
    fs::write(paths::MODPROBE_BTUSB_PATH, MODPROBE_BTUSB_CONF).map_err(|e| AppError::Io {
        path: paths::MODPROBE_BTUSB_PATH.into(),
        source: e,
    })?;

    fs::write(paths::MODPROBE_RTW88_PATH, MODPROBE_RTW88_CONF).map_err(|e| AppError::Io {
        path: paths::MODPROBE_RTW88_PATH.into(),
        source: e,
    })?;

    // 2. Dynamically bind UDEV rule to detected Bluetooth controller (with safe default fallback)
    let (vendor, product) = match usb::find_bluetooth_usb_device() {
        Some(dev) => (dev.vendor_id, dev.product_id),
        None => (hardware::REALTEK_VENDOR_ID.to_string(), hardware::REALTEK_BT_PRODUCT_ID.to_string()),
    };

    let udev_content = UDEV_BLUETOOTH_RULES
        .replace("{{VENDOR}}", &vendor)
        .replace("{{PRODUCT}}", &product);

    fs::write(paths::UDEV_RULE_PATH, udev_content).map_err(|e| AppError::Io {
        path: paths::UDEV_RULE_PATH.into(),
        source: e,
    })?;

    // 3. Immediately enforce power/control='on' on active device
    if let Some(dev) = usb::find_bluetooth_usb_device() {
        let _ = usb::set_usb_power_control_on(&dev.sysfs_path);
    }

    // 4. Reload udev rules specifically for USB subsystem (airbag against full-system bus stalls)
    let _ = Command::new("udevadm").args(["control", "--reload-rules"]).status();
    let _ = Command::new("udevadm").args(["trigger", "--subsystem-match=usb"]).status();

    Ok(())
}

#[allow(dead_code)]
pub fn rollback_system_configurations() -> AppResult<()> {
    if !os::is_running_as_root() {
        return Err(AppError::PermissionDenied);
    }

    let _ = fs::remove_file(paths::MODPROBE_BTUSB_PATH);
    let _ = fs::remove_file(paths::MODPROBE_RTW88_PATH);
    let _ = fs::remove_file(paths::UDEV_RULE_PATH);

    let _ = Command::new("udevadm").args(["control", "--reload-rules"]).status();
    let _ = Command::new("udevadm").args(["trigger", "--subsystem-match=usb"]).status();

    Ok(())
}
