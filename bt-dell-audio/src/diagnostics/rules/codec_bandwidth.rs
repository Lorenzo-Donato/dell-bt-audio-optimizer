use crate::diagnostics::rule::{DiagnosticContext, DiagnosticRule};
use crate::domain::{IssueItem, IssueSeverity};

pub struct CodecBandwidthRule;

impl DiagnosticRule for CodecBandwidthRule {
    fn evaluate(&self, ctx: &DiagnosticContext) -> Option<IssueItem> {
        let choked_device = ctx.bluetooth_devices.iter().find(|d| d.has_codec_choke_risk())?;

        Some(IssueItem {
            severity: IssueSeverity::Warning,
            title: format!("Codec de alta taxa (SBC-XQ) ativo em '{}'", choked_device.name),
            description: "O chip combo Realtek RTL8821CE compartilha uma antena única 1x1 SISO com o Wi-Fi. A taxa de bits de ~500kbps do SBC-XQ satura o buffer de rádio e causa estouros e descompasso TWS.".to_string(),
            remediation: "Execute 'bt-dell-audio fix' para priorizar os codecs estáveis SBC ou AAC.".to_string(),
        })
    }
}
