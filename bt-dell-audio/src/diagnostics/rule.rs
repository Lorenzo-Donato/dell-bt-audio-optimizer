use crate::domain::{BluetoothDevice, HardwareInfo, IssueItem, WifiInfo};

pub struct DiagnosticContext<'a> {
    pub hardware: &'a HardwareInfo,
    pub wifi: Option<&'a WifiInfo>,
    pub bluetooth_devices: &'a [BluetoothDevice],
}

pub trait DiagnosticRule: Send + Sync {
    fn evaluate(&self, ctx: &DiagnosticContext) -> Option<IssueItem>;
}

pub struct DiagnosticEngine {
    rules: Vec<Box<dyn DiagnosticRule>>,
}

impl DiagnosticEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn register<R: DiagnosticRule + 'static>(&mut self, rule: R) -> &mut Self {
        self.rules.push(Box::new(rule));
        self
    }

    pub fn analyze(&self, ctx: &DiagnosticContext) -> Vec<IssueItem> {
        self.rules
            .iter()
            .filter_map(|rule| rule.evaluate(ctx))
            .collect()
    }
}

impl Default for DiagnosticEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{BluetoothCodec, BluetoothDevice, HardwareInfo, IssueSeverity, PowerControlState};
    use crate::diagnostics::rules::hw_volume::HardwareVolumeRule;

    #[test]
    fn test_hardware_volume_rule_detection() {
        let hw = HardwareInfo {
            wifi_pci_id: "".into(),
            wifi_name: "".into(),
            wifi_driver: "".into(),
            bt_usb_id: "".into(),
            bt_name: "".into(),
            bt_usb_sysfs_path: None,
            bt_power_control: PowerControlState::On,
            bt_autosuspend_delay_ms: None,
            btusb_module_autosuspend: Some(false),
            rtw88_lps_deep_disabled: Some(true),
            rtw88_aspm_disabled: Some(true),
        };

        let device_with_hw_vol = BluetoothDevice {
            mac_address: "AA:BB:CC:DD:EE:FF".into(),
            name: "JBL Wave Beam 2".into(),
            is_connected: true,
            battery_percentage: Some(90),
            card_name: Some("bluez_card.test".into()),
            active_profile_name: Some("a2dp-sink".into()),
            active_codec: BluetoothCodec::Sbc,
            hw_volume_enabled: Some(true),
            autoswitch_profile_enabled: Some(false),
        };

        let devices = vec![device_with_hw_vol];
        let ctx = DiagnosticContext {
            hardware: &hw,
            wifi: None,
            bluetooth_devices: &devices,
        };

        let rule = HardwareVolumeRule;
        let issue = rule.evaluate(&ctx);

        assert!(issue.is_some());
        let item = issue.unwrap();
        assert_eq!(item.severity, IssueSeverity::Critical);
        assert!(item.title.contains("JBL Wave Beam 2"));
    }
}
