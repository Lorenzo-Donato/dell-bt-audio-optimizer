use crate::diagnostics::rule::{DiagnosticContext, DiagnosticRule};
use crate::domain::{IssueItem, IssueSeverity};

pub struct UsbAutosuspendRule;

impl DiagnosticRule for UsbAutosuspendRule {
    fn evaluate(&self, ctx: &DiagnosticContext) -> Option<IssueItem> {
        if ctx.hardware.is_bluetooth_power_saving() {
            Some(IssueItem {
                severity: IssueSeverity::Critical,
                title: "USB Autosuspend ativo no controlador Bluetooth (0bda:c829)".to_string(),
                description: "O Linux suspende o rádio Bluetooth a cada intervalo ocioso, gerando atraso na entrega de pacotes, micro-cortes e desincronização entre os fones L e R.".to_string(),
                remediation: "Execute 'sudo ./scripts/apply-system-tuning.sh' ou execute 'bt-dell-audio systune' para gravar regras definitivas em modprobe e udev.".to_string(),
            })
        } else {
            None
        }
    }
}
