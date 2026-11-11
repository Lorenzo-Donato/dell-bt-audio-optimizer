use std::fs;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HealthStatus {
    Optimal,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetric {
    pub name: String,
    pub status: HealthStatus,
    pub value: String,
    pub details: String,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthReport {
    pub overall_status: HealthStatus,
    pub metrics: Vec<HealthMetric>,
}

/// Strategy trait for individual system health audits.
/// Adheres to Single Responsibility (SRP) and Open/Closed (OCP) principles.
pub trait HealthCheck: Send + Sync {
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    fn check(&self) -> HealthMetric;
}

// -----------------------------------------------------------------------------
// Concrete Health Check Implementations
// -----------------------------------------------------------------------------

/// Audits dangerous 'btusb reset=1' parameter (Primary cause of xHCI hard lockups on Intel Alder Lake)
pub struct KernelResetSafetyCheck;

impl HealthCheck for KernelResetSafetyCheck {
    fn name(&self) -> &'static str {
        "Kernel btusb.reset Parameter Check"
    }

    fn check(&self) -> HealthMetric {
        let reset_param = fs::read_to_string("/sys/module/btusb/parameters/reset")
            .unwrap_or_default()
            .trim()
            .to_string();

        if reset_param.eq_ignore_ascii_case("y") || reset_param == "1" {
            HealthMetric {
                name: "Parâmetro Kernel: btusb.reset".to_string(),
                status: HealthStatus::Critical,
                value: format!("{} (PERIGO CRÍTICO DE TRAVAMENTO)", reset_param),
                details: "O driver btusb está instruído a resetar o barramento USB em erros. Em chipsets Intel Alder Lake, esse reset causa deadlocks no controlador xHCI e congelamento total do sistema.".to_string(),
                recommendation: Some("Execute 'sudo ./scripts/apply-system-tuning.sh' para aplicar 'reset=0' ou desative reset=1 em /etc/modprobe.d/btusb.conf.".to_string()),
            }
        } else {
            HealthMetric {
                name: "Parâmetro Kernel: btusb.reset".to_string(),
                status: HealthStatus::Optimal,
                value: format!("{} (Protegido)", if reset_param.is_empty() { "N/A" } else { &reset_param }),
                details: "Reset agressivo do controlador USB está desativado. Controlador xHCI protegido contra deadlocks.".to_string(),
                recommendation: None,
            }
        }
    }
}

/// Audits available system memory to prevent OOM killer locks
pub struct MemoryPressureCheck;

impl HealthCheck for MemoryPressureCheck {
    fn name(&self) -> &'static str {
        "Memory Pressure Check"
    }

    fn check(&self) -> HealthMetric {
        let (mem_total, mem_avail) = read_mem_info();
        if mem_total == 0 {
            return HealthMetric {
                name: "Pressão de Memória RAM".to_string(),
                status: HealthStatus::Optimal,
                value: "N/A".to_string(),
                details: "Não foi possível ler /proc/meminfo.".to_string(),
                recommendation: None,
            };
        }

        let percent_avail = (mem_avail as f64 / mem_total as f64) * 100.0;
        let status = if percent_avail < 5.0 {
            HealthStatus::Critical
        } else if percent_avail < 15.0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Optimal
        };

        HealthMetric {
            name: "Pressão de Memória RAM".to_string(),
            status,
            value: format!("{:.1}% disponível ({}/{} MB)", percent_avail, mem_avail / 1024, mem_total / 1024),
            details: "Memória livre para evitar travamentos por esgotamento (OOM killer).".to_string(),
            recommendation: if status != HealthStatus::Optimal {
                Some("Feche abas pesadas ou aplicações com alto consumo de memória.".to_string())
            } else {
                None
            },
        }
    }
}

/// Audits CPU load average (1m)
pub struct CpuLoadCheck;

impl HealthCheck for CpuLoadCheck {
    fn name(&self) -> &'static str {
        "CPU Load Average Check"
    }

    fn check(&self) -> HealthMetric {
        let load1_str = fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|content| content.split_whitespace().next().map(|s| s.to_string()))
            .unwrap_or_else(|| "0.0".to_string());

        let load_val = load1_str.parse::<f64>().unwrap_or(0.0);
        let status = if load_val > 12.0 {
            HealthStatus::Critical
        } else if load_val > 6.0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Optimal
        };

        HealthMetric {
            name: "Carga da CPU (Load Average 1m)".to_string(),
            status,
            value: load1_str,
            details: "Média de processos aguardando tempo de execução na CPU.".to_string(),
            recommendation: if status != HealthStatus::Optimal {
                Some("Identifique processos em loop com 'top' ou 'htop'.".to_string())
            } else {
                None
            },
        }
    }
}

/// Audits PipeWire and WirePlumber daemon health
pub struct AudioServicesCheck;

impl HealthCheck for AudioServicesCheck {
    fn name(&self) -> &'static str {
        "Audio Services Daemon Check"
    }

    fn check(&self) -> HealthMetric {
        let (pw_active, wp_active) = check_audio_services();
        let status = if !pw_active || !wp_active {
            HealthStatus::Critical
        } else {
            HealthStatus::Optimal
        };

        HealthMetric {
            name: "Servidores de Áudio (PipeWire & WirePlumber)".to_string(),
            status,
            value: format!(
                "PipeWire: {}, WirePlumber: {}",
                if pw_active { "Ativo" } else { "Parado" },
                if wp_active { "Ativo" } else { "Parado" }
            ),
            details: "Estado dos serviços multimídia em espaço de usuário.".to_string(),
            recommendation: if status != HealthStatus::Optimal {
                Some("Execute 'systemctl --user restart pipewire wireplumber'.".to_string())
            } else {
                None
            },
        }
    }
}

// -----------------------------------------------------------------------------
// Health Registry & Doctor (Strategy Pattern)
// -----------------------------------------------------------------------------

pub struct HealthDoctor {
    checks: Vec<Box<dyn HealthCheck>>,
}

impl HealthDoctor {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    pub fn with_default_checks() -> Self {
        Self {
            checks: vec![
                Box::new(KernelResetSafetyCheck),
                Box::new(MemoryPressureCheck),
                Box::new(CpuLoadCheck),
                Box::new(AudioServicesCheck),
            ],
        }
    }

    #[allow(dead_code)]
    pub fn register(&mut self, check: Box<dyn HealthCheck>) {
        self.checks.push(check);
    }

    pub fn diagnose(&self) -> SystemHealthReport {
        let metrics: Vec<HealthMetric> = self.checks.iter().map(|c| c.check()).collect();
        let overall_status = metrics.iter().map(|m| m.status).max().unwrap_or(HealthStatus::Optimal);

        SystemHealthReport {
            overall_status,
            metrics,
        }
    }
}

impl Default for HealthDoctor {
    fn default() -> Self {
        Self::with_default_checks()
    }
}

/// Public convenience function to perform a full system health audit
pub fn run_health_check() -> SystemHealthReport {
    HealthDoctor::default().diagnose()
}

// -----------------------------------------------------------------------------
// Internal Helpers
// -----------------------------------------------------------------------------

fn read_mem_info() -> (u64, u64) {
    let mut total = 0;
    let mut avail = 0;

    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                total = parse_kb(line);
            } else if line.starts_with("MemAvailable:") {
                avail = parse_kb(line);
            }
        }
    }

    (total, avail)
}

fn parse_kb(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}

fn check_audio_services() -> (bool, bool) {
    let pw = Command::new("systemctl")
        .args(["--user", "is-active", "pipewire"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
        .unwrap_or(false);

    let wp = Command::new("systemctl")
        .args(["--user", "is-active", "wireplumber"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "active")
        .unwrap_or(false);

    (pw, wp)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCheck {
        mock_status: HealthStatus,
    }

    impl HealthCheck for MockCheck {
        fn name(&self) -> &'static str {
            "Mock Check"
        }
        fn check(&self) -> HealthMetric {
            HealthMetric {
                name: "Mock Metric".to_string(),
                status: self.mock_status,
                value: "mock_value".to_string(),
                details: "mock details".to_string(),
                recommendation: None,
            }
        }
    }

    #[test]
    fn test_health_doctor_aggregation() {
        let mut doctor = HealthDoctor::new();
        doctor.register(Box::new(MockCheck { mock_status: HealthStatus::Optimal }));
        doctor.register(Box::new(MockCheck { mock_status: HealthStatus::Warning }));

        let report = doctor.diagnose();
        assert_eq!(report.overall_status, HealthStatus::Warning);
        assert_eq!(report.metrics.len(), 2);
    }

    #[test]
    fn test_health_doctor_critical_escalation() {
        let mut doctor = HealthDoctor::new();
        doctor.register(Box::new(MockCheck { mock_status: HealthStatus::Optimal }));
        doctor.register(Box::new(MockCheck { mock_status: HealthStatus::Critical }));

        let report = doctor.diagnose();
        assert_eq!(report.overall_status, HealthStatus::Critical);
    }
}
