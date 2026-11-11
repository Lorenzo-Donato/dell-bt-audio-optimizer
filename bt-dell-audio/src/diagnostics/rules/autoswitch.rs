use crate::audio::wireplumber;
use crate::diagnostics::rule::{DiagnosticContext, DiagnosticRule};
use crate::domain::{IssueItem, IssueSeverity};

pub struct HeadsetAutoswitchRule;

impl DiagnosticRule for HeadsetAutoswitchRule {
    fn evaluate(&self, _ctx: &DiagnosticContext) -> Option<IssueItem> {
        if wireplumber::is_autoswitch_profile_enabled() {
            Some(IssueItem {
                severity: IssueSeverity::Warning,
                title: "Troca automática para perfil de Headset (HSP/HFP) ativada".to_string(),
                description: "Programas como Steam ou navegadores forçam a degradação para mono 8kHz telefônico ao consultar dispositivos de gravação. Como você possui um microfone Fifine USB dedicado, essa troca é totalmente indesejada.".to_string(),
                remediation: "Definir 'bluetooth.autoswitch-to-headset-profile = false' e restringir a 'a2dp_sink' via 'bt-dell-audio fix'.".to_string(),
            })
        } else {
            None
        }
    }
}
