use colored::*;
use crate::audio::wireplumber;
use crate::domain::{sanitizer, DiagnosticReport, IssueSeverity, PowerControlState};
use crate::service::history::DiagnosticSnapshot;
use super::theme::{borders, symbols};

pub fn render_checkup_report(report: &DiagnosticReport, anonymize: bool) {
    println!("\n{}", borders::TOP.cyan().bold());
    println!("{}", "║        CHECKUP COMPLETO DE ÁUDIO, BLUETOOTH E WI-FI (DELL)        ║".cyan().bold());
    println!("{}", borders::BOTTOM.cyan().bold());

    if anonymize {
        println!("  {}", "[PRIVACY SHIELD ATIVO] Identificadores físicos (MAC, SSID) anonimizados para compartilhamento seguro.".bright_black().italic());
    }

    // 1. OS & Kernel
    println!("\n{}", "─── 1. SISTEMA OPERACIONAL & KERNEL ───".white().bold());
    println!("  {:<25} {}", "Distribuição:".bright_black(), report.os_name.green().bold());
    println!("  {:<25} {}", "Kernel:".bright_black(), report.kernel_version.yellow());

    // 2. Hardware
    println!("\n{}", "─── 2. HARDWARE & DRIVERS (CHIP COMBO) ───".white().bold());
    println!("  {:<25} {}", "Placa Wi-Fi (PCIe):".bright_black(), report.hardware.wifi_name.white());
    println!("  {:<25} {}", "Driver Wi-Fi:".bright_black(), report.hardware.wifi_driver.cyan());
    println!("  {:<25} {}", "Rádio Bluetooth (USB):".bright_black(), report.hardware.bt_name.white());

    let power_ctrl_display = match report.hardware.bt_power_control {
        PowerControlState::Auto => "auto (ATIVO - Causa cortes de áudio)".red().bold(),
        PowerControlState::On => "on (DESATIVADO - Estável)".green().bold(),
        PowerControlState::Unknown => "desconhecido".yellow(),
    };
    println!("  {:<25} {}", "BT USB power/control:".bright_black(), power_ctrl_display);

    let module_auto = match report.hardware.btusb_module_autosuspend {
        Some(true) => "Y (Ativado por padrão no kernel)".yellow(),
        Some(false) => "N (Desativado via modprobe)".green().bold(),
        None => "N/A".bright_black(),
    };
    println!("  {:<25} {}", "btusb.enable_autosuspend:".bright_black(), module_auto);

    // 3. Wi-Fi
    println!("\n{}", "─── 3. REDE WI-FI & COEXISTÊNCIA DE RÁDIO ───".white().bold());
    if let Some(w) = &report.wifi {
        let ssid_display = if anonymize {
            sanitizer::mask_ssid(&w.ssid)
        } else {
            w.ssid.clone()
        };
        println!("  {:<25} {}", "Rede Conectada (SSID):".bright_black(), ssid_display.cyan().bold());
        let band_disp = if w.band.is_coexistence_hazard() {
            format!("{} (CRÍTICO - Colide diretamente com Bluetooth)", w.band.as_str()).red().bold()
        } else {
            format!("{} (Excelente - Sem interferência com Bluetooth)", w.band.as_str()).green().bold()
        };
        println!("  {:<25} {}", "Frequência / Banda:".bright_black(), band_disp);
        println!("  {:<25} Canal {}, Sinal: {}%", "Canal & Sinal:".bright_black(), w.channel, w.signal_percent);
        println!("  {:<25} {}", "Wi-Fi Power Save:".bright_black(), if w.powersave_enabled { "on".yellow() } else { "off".green() });
    } else {
        println!("  {}", "Nenhuma rede Wi-Fi ativa detectada.".yellow());
    }

    // 4. Bluetooth Devices
    println!("\n{}", "─── 4. DISPOSITIVOS DE ÁUDIO BLUETOOTH ───".white().bold());
    let autoswitch_global = wireplumber::is_autoswitch_profile_enabled();

    if report.bluetooth_devices.is_empty() {
        println!("  {}", "Nenhum fone/dispositivo Bluetooth conectado no momento.".yellow());
    } else {
        for dev in &report.bluetooth_devices {
            let mac_display = if anonymize {
                sanitizer::mask_mac_address(&dev.mac_address)
            } else {
                dev.mac_address.clone()
            };

            println!("  {} {} [{}]", symbols::BULLET.cyan(), dev.name.cyan().bold(), mac_display.bright_black());
            if let Some(bat) = dev.battery_percentage {
                println!("    Bateria: {}%", bat);
            }

            let codec_disp = if dev.active_codec.is_high_bandwidth_risk() {
                dev.active_codec.display_name().yellow().bold()
            } else {
                dev.active_codec.display_name().green().bold()
            };
            println!("    Codec Ativo: {}", codec_disp);

            let hw_vol_disp = match dev.hw_volume_enabled {
                Some(true) => "ATIVADO (PERIGOSO - Causa quedas abruptas de volume)".red().bold(),
                Some(false) => "DESATIVADO (SEGURO - Volume suave controlado por software)".green().bold(),
                None => "Não configurado".bright_black(),
            };
            println!("    Volume de Hardware (AVRCP): {}", hw_vol_disp);

            let auto_disp = if autoswitch_global {
                "ATIVADO (Degrada para modo chamada se app abrir mic)".yellow()
            } else {
                "DESATIVADO (Mantém sempre estéreo de alta fidelidade)".green().bold()
            };
            println!("    Autoswitch de Headset: {}", auto_disp);
        }
    }

    // 5. Issues & Recommendations
    println!("\n{}", "─── 5. DIAGNÓSTICO DE PROBLEMAS & RECOMENDAÇÕES ───".white().bold());
    if report.issues.is_empty() {
        println!("  {} Nenhum problema detectado! Seu sistema está com estabilidade máxima.", symbols::CHECK.green().bold());
    } else {
        for (i, issue) in report.issues.iter().enumerate() {
            let tag = match issue.severity {
                IssueSeverity::Critical => issue.severity.tag().red().bold(),
                IssueSeverity::Warning => issue.severity.tag().yellow().bold(),
                IssueSeverity::Info => issue.severity.tag().blue(),
                IssueSeverity::Good => issue.severity.tag().green(),
            };
            println!("  [{}] [{}] {}", (i + 1), tag, issue.title.bold());
            println!("      Descrição: {}", issue.description.white());
            println!("      Correção:  {}", issue.remediation.cyan());
            println!();
        }
    }
}

pub fn render_dashboard(report: &DiagnosticReport, anonymize: bool) {
    println!("{}", borders::DOUBLE_DIVIDER.cyan());
    println!("{}", "      SENTINELA DE ÁUDIO & BLUETOOTH - MONITOR EM TEMPO REAL       ".cyan().bold());
    println!("{}", borders::DOUBLE_DIVIDER.cyan());

    print!("USB BT Power: ");
    match report.hardware.bt_power_control {
        PowerControlState::On => print!("{}", "[ON - ESTÁVEL] ".green().bold()),
        PowerControlState::Auto => print!("{}", "[AUTO - INSTÁVEL] ".red().bold()),
        PowerControlState::Unknown => print!("{}", "[DESCONHECIDO] ".yellow()),
    }

    if let Some(w) = &report.wifi {
        let band_color = if w.band.is_coexistence_hazard() { "2.4 GHz".red() } else { "5 GHz".green() };
        let ssid_disp = if anonymize { sanitizer::mask_ssid(&w.ssid) } else { w.ssid.clone() };
        print!("| Wi-Fi: {} ({}) ", ssid_disp.cyan(), band_color);
    }

    println!();
    println!("{}", borders::DIVIDER.bright_black());

    for dev in &report.bluetooth_devices {
        println!("Fone: {} | Bateria: {}%", dev.name.cyan().bold(), dev.battery_percentage.unwrap_or(0));
        println!("  Codec:   {}", dev.active_codec.display_name().yellow());
        println!(
            "  HW Vol:  {}",
            if dev.hw_volume_enabled == Some(false) {
                "DESATIVADO (Seguro)".green()
            } else {
                "ATIVADO (Oscilante)".red()
            }
        );
    }

    println!("\n{}", "Pressione Ctrl+C para encerrar o monitoramento.".bright_black());
}

pub fn render_health_report(report: &crate::service::health::SystemHealthReport) {
    use crate::service::health::HealthStatus;

    println!("\n{}", borders::TOP.cyan().bold());
    println!("{}", "║          CHECKUP DE SAÚDE E ESTABILIDADE DO SISTEMA (DOCTOR)       ║".cyan().bold());
    println!("{}", borders::BOTTOM.cyan().bold());

    let overall_disp = match report.overall_status {
        HealthStatus::Optimal => "SISTEMA ÍNTEGRO & ESTÁVEL".green().bold(),
        HealthStatus::Warning => "ATENÇÃO - ALERTA DE PERFORMANCE".yellow().bold(),
        HealthStatus::Critical => "RISCO CRÍTICO DE INSTABILIDADE / TRAVAMENTO".red().bold(),
    };
    println!("\n  Status Geral de Saúde: {}\n", overall_disp);

    for (i, m) in report.metrics.iter().enumerate() {
        let (tag, color_fn): (&str, fn(&str) -> ColoredString) = match m.status {
            HealthStatus::Optimal => ("SEGURO", |s| s.green().bold()),
            HealthStatus::Warning => ("AVISO", |s| s.yellow().bold()),
            HealthStatus::Critical => ("PERIGO", |s| s.red().bold()),
        };

        println!("  [{}] [{}] {}", (i + 1), color_fn(tag), m.name.bold());
        println!("      Valor:     {}", m.value);
        println!("      Detalhes:  {}", m.details.bright_black());
        if let Some(rec) = &m.recommendation {
            println!("      Ação:      {}", rec.cyan().bold());
        }
        println!();
    }
}

pub fn render_history_list(snapshots: &[DiagnosticSnapshot]) {
    println!("\n{}", borders::TOP.cyan().bold());
    println!("{}", "║          HISTÓRICO LOCAL DE DIAGNÓSTICOS & COMPARATIVOS            ║".cyan().bold());
    println!("{}", borders::BOTTOM.cyan().bold());

    if snapshots.is_empty() {
        println!("  {}", "Nenhum relatório local armazenado no momento.".yellow());
        println!("  Execute 'bt-dell-audio checkup --save' para gerar um snapshot comparativo.\n");
        return;
    }

    println!("  Total de relatórios armazenados (política máx. 5 snapshots / 7 dias): {}\n", snapshots.len());

    for (i, snap) in snapshots.iter().enumerate() {
        let status_tag = if snap.has_critical {
            "CRÍTICO".red().bold()
        } else if snap.issues_count > 0 {
            "ALERTA".yellow().bold()
        } else {
            "ESTÁVEL".green().bold()
        };

        println!(
            "  [{}] [{}] {} ({} problemas detectados)",
            i + 1,
            status_tag,
            snap.date_formatted.cyan().bold(),
            snap.issues_count
        );
        println!("      Arquivo:   {}", snap.filename.bright_black());
        println!("      Sistema:   {} ({})", snap.os_name, snap.kernel_version.bright_black());
        println!();
    }
}
