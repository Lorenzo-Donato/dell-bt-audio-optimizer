//! Compile-time embedded configuration templates.
//! Prevents bloating of business logic files and guarantees self-contained binaries.

pub const WIREPLUMBER_BLUETOOTH_CONF: &str = include_str!("../../templates/wireplumber_bluetooth.conf.template");
pub const MODPROBE_BTUSB_CONF: &str = include_str!("../../templates/modprobe_btusb.template");
pub const MODPROBE_RTW88_CONF: &str = include_str!("../../templates/modprobe_rtw88.template");
pub const UDEV_BLUETOOTH_RULES: &str = include_str!("../../templates/udev_bluetooth.rules.template");
pub const SYSTEMD_USER_SERVICE: &str = include_str!("../../templates/systemd_user.service.template");
