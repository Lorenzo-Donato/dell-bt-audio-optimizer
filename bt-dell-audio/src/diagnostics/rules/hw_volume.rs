use crate::diagnostics::rule::{DiagnosticContext, DiagnosticRule};
use crate::domain::{IssueItem, IssueSeverity};

pub struct HardwareVolumeRule;

impl DiagnosticRule for HardwareVolumeRule {
    fn evaluate(&self, ctx: &DiagnosticContext) -> Option<IssueItem> {
        let faulty_device = ctx.bluetooth_devices.iter().find(|d| d.has_hardware_volume_risk())?;

        Some(IssueItem {
            severity: IssueSeverity::Critical,
            title: format!("Volume de Hardware (AVRCP) ativo em '{}'", faulty_device.name),
            description: "Fones JBL Wave Beam 2 possuem um bug crônico de firmware na sincronização de ganho de hardware. Qualquer oscilação faz o DSP interno atenuar brutalmente o volume ou oscilar entre esquerda e direita.".to_string(),
            remediation: "Execute 'bt-dell-audio fix' para desativar o volume de hardware no WirePlumber e usar controle seguro por software.".to_string(),
        })
    }
}
