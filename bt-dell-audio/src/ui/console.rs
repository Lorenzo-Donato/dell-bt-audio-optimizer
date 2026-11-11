use colored::*;
use crate::audio::wireplumber;
use crate::domain::{DiagnosticReport, IssueSeverity, PowerControlState};
use super::theme::{borders, symbols};

pub fn render_checkup_report(report: &DiagnosticReport) {
    println!("\n{}", borders::TOP.cyan().bold());
    println!("{}", "║        CHECKUP COMPLETO DE ÁUDIO, BLUETOOTH E WI-FI (DELL)        ║".cyan().bold());
    println!("{}", borders::BOTTOM.cyan().bold());

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
        println!("  {:<25} {}", "Rede Conectada (SSID):".bright_black(), w.ssid.cyan().bold());
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
            println!("  {} {} [{}]", symbols::BULLET.cyan(), dev.name.cyan().bold(), dev.mac_address.bright_black());
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

pub fn render_dashboard(report: &DiagnosticReport) {
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
        print!("| Wi-Fi: {} ({}) ", w.ssid.cyan(), band_color);
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
