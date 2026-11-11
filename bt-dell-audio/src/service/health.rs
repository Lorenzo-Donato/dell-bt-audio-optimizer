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

pub fn run_health_check() -> SystemHealthReport {
    let mut metrics = Vec::new();
    let mut worst_status = HealthStatus::Optimal;

    // 1. Audit dangerous 'btusb reset=1' parameter (Primary cause of xHCI hard lockups)
    let reset_param = fs::read_to_string("/sys/module/btusb/parameters/reset")
        .unwrap_or_default()
        .trim()
        .to_string();

    if reset_param.eq_ignore_ascii_case("y") || reset_param == "1" {
        worst_status = HealthStatus::Critical;
        metrics.push(HealthMetric {
            name: "Parâmetro Kernel: btusb.reset".to_string(),
            status: HealthStatus::Critical,
            value: format!("{} (PERIGO CRÍTICO DE TRAVAMENTO)", reset_param),
            details: "O driver btusb está instruído a resetar o barramento USB em erros. Em chipsets Intel Alder Lake, esse reset causa deadlocks no controlador xHCI e congelamento total do sistema.".to_string(),
            recommendation: Some("Execute 'sudo ./scripts/apply-system-tuning.sh' para aplicar 'reset=0' ou desative reset=1 em /etc/modprobe.d/btusb.conf.".to_string()),
        });
    } else {
        metrics.push(HealthMetric {
            name: "Parâmetro Kernel: btusb.reset".to_string(),
            status: HealthStatus::Optimal,
            value: format!("{} (Protegido)", if reset_param.is_empty() { "N/A" } else { &reset_param }),
            details: "Reset agressivo do controlador USB está desativado. Controlador xHCI protegido contra deadlocks.".to_string(),
            recommendation: None,
        });
    }

    // 2. Audit Memory Pressure
    let (mem_total, mem_avail) = read_mem_info();
    if mem_total > 0 {
        let percent_avail = (mem_avail as f64 / mem_total as f64) * 100.0;
        let mem_status = if percent_avail < 5.0 {
            HealthStatus::Critical
        } else if percent_avail < 15.0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Optimal
        };

        if mem_status > worst_status {
            worst_status = mem_status;
        }

        metrics.push(HealthMetric {
            name: "Pressão de Memória RAM".to_string(),
            status: mem_status,
            value: format!("{:.1}% disponível ({}/{} MB)", percent_avail, mem_avail / 1024, mem_total / 1024),
            details: "Memória livre para evitar travamentos por esgotamento (OOM killer).".to_string(),
            recommendation: if mem_status != HealthStatus::Optimal {
                Some("Feche abas pesadas ou aplicações com alto consumo de memória.".to_string())
            } else {
                None
            },
        });
    }

    // 3. Audit System Load Average
    if let Ok(loadavg) = fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = loadavg.split_whitespace().collect();
        let load1 = parts.first().unwrap_or(&"0.0");
        let load_val = load1.parse::<f64>().unwrap_or(0.0);

        let load_status = if load_val > 12.0 {
            HealthStatus::Critical
        } else if load_val > 6.0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Optimal
        };

        if load_status > worst_status {
            worst_status = load_status;
        }

        metrics.push(HealthMetric {
            name: "Carga da CPU (Load Average 1m)".to_string(),
            status: load_status,
            value: load1.to_string(),
            details: "Média de processos aguardando tempo de execução na CPU.".to_string(),
            recommendation: if load_status != HealthStatus::Optimal {
                Some("Identifique processos em loop com 'top' ou 'htop'.".to_string())
            } else {
                None
            },
        });
    }

    // 4. Audit PipeWire & WirePlumber Services
    let (pw_active, wp_active) = check_audio_services();
    let audio_status = if !pw_active || !wp_active {
        HealthStatus::Critical
    } else {
        HealthStatus::Optimal
    };

    if audio_status > worst_status {
        worst_status = audio_status;
    }

    metrics.push(HealthMetric {
        name: "Servidores de Áudio (PipeWire & WirePlumber)".to_string(),
        status: audio_status,
        value: format!("PipeWire: {}, WirePlumber: {}", if pw_active { "Ativo" } else { "Parado" }, if wp_active { "Ativo" } else { "Parado" }),
        details: "Estado dos serviços multimídia em espaço de usuário.".to_string(),
        recommendation: if audio_status != HealthStatus::Optimal {
            Some("Execute 'systemctl --user restart pipewire wireplumber'.".to_string())
        } else {
            None
        },
    });

    SystemHealthReport {
        overall_status: worst_status,
        metrics,
    }
}

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
