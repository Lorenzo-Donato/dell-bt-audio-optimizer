use crate::diagnostics::rule::{DiagnosticContext, DiagnosticRule};
use crate::domain::{IssueItem, IssueSeverity};

pub struct WifiCoexistenceRule;

impl DiagnosticRule for WifiCoexistenceRule {
    fn evaluate(&self, ctx: &DiagnosticContext) -> Option<IssueItem> {
        let wifi = ctx.wifi?;
        if wifi.band.is_coexistence_hazard() {
            Some(IssueItem {
                severity: IssueSeverity::Critical,
                title: "Wi-Fi conectado na banda de 2.4 GHz compartilhada!".to_string(),
                description: "O rádio Bluetooth opera exclusivamente na faixa de 2.4 GHz. Como o RTL8821CE compartilha a mesma antena física entre Wi-Fi e BT, o tráfego de Wi-Fi de 2.4 GHz colide diretamente com os pacotes de áudio Bluetooth.".to_string(),
                remediation: "Conecte-se a uma rede Wi-Fi de 5 GHz para isolar as faixas de frequência e desobstruir o rádio Bluetooth.".to_string(),
            })
        } else {
            None
        }
    }
}
