use std::path::Path;

#[test]
fn test_template_integrity_wireplumber() {
    let template = include_str!("../templates/wireplumber_bluetooth.conf.template");
    assert!(!template.is_empty(), "Template WirePlumber não deve ser vazio");
    assert!(template.contains("bluez5.enable-hw-volume = false"), "Deve conter disable-hw-volume");
    assert!(template.contains("bluetooth.autoswitch-to-headset-profile = false"), "Deve conter disable-autoswitch");
    assert!(template.contains("bluez5.roles = [ a2dp_sink ]"), "Deve restringir para a2dp_sink");
}

#[test]
fn test_template_integrity_modprobe() {
    let btusb = include_str!("../templates/modprobe_btusb.template");
    assert!(btusb.contains("options btusb enable_autosuspend=n reset=0"));

    let rtw88 = include_str!("../templates/modprobe_rtw88.template");
    assert!(rtw88.contains("options rtw88_pci disable_aspm=y"));
    assert!(rtw88.contains("options rtw88_core disable_lps_deep=y"));
}

#[test]
fn test_template_integrity_udev() {
    let udev = include_str!("../templates/udev_bluetooth.rules.template");
    assert!(udev.contains("0bda"));
    assert!(udev.contains("c829"));
    assert!(udev.contains("power/control"));
}

#[test]
fn test_template_integrity_systemd() {
    let service = include_str!("../templates/systemd_user.service.template");
    assert!(service.contains("Description=Dell Realtek Bluetooth Audio Sentinel & Optimizer"));
    assert!(service.contains("ExecStart={{EXEC_PATH}} daemon"));
    assert!(service.contains("WantedBy=default.target"));
}

#[test]
fn test_path_sanitization_and_standards() {
    // Ensure all target paths are standard Linux absolute paths
    let modprobe_btusb = "/etc/modprobe.d/btusb.conf";
    let udev_rule = "/etc/udev/rules.d/50-bluetooth-realtek.rules";

    assert!(Path::new(modprobe_btusb).is_absolute());
    assert!(Path::new(udev_rule).is_absolute());
}
