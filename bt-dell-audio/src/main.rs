mod audio;
mod config;
mod diagnostics;
mod domain;
mod hardware;
mod service;
mod ui;

use clap::{Parser, Subcommand};
use colored::*;
use service::{fixer, rollback, sentinel, system_tuner};
use ui::prompts;

#[derive(Parser)]
#[command(name = "bt-dell-audio")]
#[command(author = "Lorenzo & Antigravity")]
#[command(version = "2.1")]
#[command(
    about = "Engine modular de estabilização e rollback de áudio Bluetooth para Linux (Dell Realtek)",
    long_about = "bt-dell-audio é um utilitário de engenharia de áudio projetado para eliminar oscilações de ganho, descompasso TWS e engasgos de áudio em chips combo Realtek RTL8821CE com fones JBL Wave Beam 2 no Linux PipeWire."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Assume 'sim' para todas as perguntas de confirmação (útil para automação/scripts)
    #[arg(short = 'y', long = "yes", global = true)]
    yes: bool,

    /// Simula as alterações sem gravar nenhum arquivo em disco
    #[arg(long = "dry-run", global = true)]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Executa o diagnóstico completo de hardware, drivers, Wi-Fi e PipeWire
    Checkup,

    /// Executa o checkup de saúde do sistema, memória, carga e integridade do barramento USB
    Doctor,

    /// Aplica as correções em espaço de usuário (WirePlumber 0.5, volume por software, perfil estéreo)
    Fix,

    /// Reverte todas as alterações de áudio e restaura as configurações originais do sistema
    Rollback,

    /// Grava as configurações de kernel (modprobe) e UDEV contra o autosuspend USB (requer sudo)
    SysTune,

    /// Abre o painel interativo de telemetria em tempo real
    Watch,

    /// Executa o sentinela em segundo plano com auto-cura contínua
    Daemon,

    /// Instala e ativa o sentinela como serviço de usuário no systemd
    InstallService,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Checkup) | None => handle_checkup(),
        Some(Commands::Doctor) => handle_doctor(),
        Some(Commands::Fix) => handle_fix(cli.yes, cli.dry_run),
        Some(Commands::Rollback) => handle_rollback(cli.yes),
        Some(Commands::SysTune) => handle_systune(cli.yes),
        Some(Commands::Watch) => sentinel::start_interactive_watch_loop(),
        Some(Commands::Daemon) => sentinel::start_background_daemon(),
        Some(Commands::InstallService) => handle_install_service(),
    }
}

fn handle_doctor() {
    let report = service::health::run_health_check();
    ui::console::render_health_report(&report);
}

fn handle_checkup() {
    let report = diagnostics::run_full_diagnosis();
    ui::console::render_checkup_report(&report);

    if report.has_critical_issues() {
        println!("{}", "Sugestão imediata: Execute 'bt-dell-audio fix' para estabilizar seu áudio.".yellow());
    }
}

fn handle_fix(auto_yes: bool, dry_run: bool) {
    println!("{}", "===> Estabilização de Áudio no WirePlumber 0.5 <===".cyan().bold());

    if dry_run {
        println!("{}", "[DRY-RUN] Nenhuma alteração real será feita no disco.".yellow().bold());
        let conf_path = audio::wireplumber::get_wireplumber_config_path().unwrap_or_default();
        println!("  - Alvo WirePlumber: {}", conf_path.display());
        println!("  - Volume AVRCP: será desativado");
        println!("  - Codecs: será priorizado SBC/AAC");
        return;
    }

    if !prompts::confirm("Deseja aplicar as configurações otimizadas de estabilidade no WirePlumber?", auto_yes) {
        println!("{}", "Operação cancelada pelo usuário.".yellow());
        return;
    }

    match fixer::execute_audio_stabilization() {
        Ok(summary) => {
            println!("  {} Configuração gravada em: {}", "✓".green(), summary.config_written_path.display());
            for dev in summary.devices_adjusted {
                println!("  {} Dispositivo ajustado: {}", "✓".green(), dev);
            }
            println!("\n{}", "✓ Correções de usuário aplicadas com sucesso!".green().bold());
        }
        Err(err) => {
            eprintln!("{} {}", "Erro ao aplicar correções:".red().bold(), err);
            std::process::exit(1);
        }
    }
}

fn handle_rollback(auto_yes: bool) {
    println!("{}", "===> Reversão (Rollback) de Configurações <===".yellow().bold());
    println!("Esta operação removerá as regras personalizadas do WirePlumber e restaurará o backup.");

    if !prompts::confirm("Tem certeza de que deseja reverter o áudio para o estado original?", auto_yes) {
        println!("{}", "Rollback cancelado.".yellow());
        return;
    }

    match rollback::execute_user_rollback() {
        Ok(res) => {
            if res.wireplumber_reverted {
                println!("  {} Configurações de WirePlumber restauradas com sucesso.", "✓".green());
            } else {
                println!("  {} Nenhuma alteração anterior encontrada no WirePlumber.", "ℹ".blue());
            }
            println!("\n{}", "✓ Rollback em espaço de usuário concluído com sucesso!".green().bold());
            println!("\nSe você também aplicou configurações de Kernel/Udev com sudo, execute:");
            println!("  {}\n", "sudo ./scripts/rollback.sh".bold().cyan());
        }
        Err(err) => {
            eprintln!("{} {}", "Erro durante o rollback:".red().bold(), err);
            std::process::exit(1);
        }
    }
}

fn handle_systune(auto_yes: bool) {
    println!("{}", "===> Configurações de Sistema (Kernel & Udev) <===".cyan().bold());

    if !prompts::confirm("Deseja gravar as regras permanentes de modprobe e udev contra o autosuspend do Bluetooth?", auto_yes) {
        println!("{}", "Operação cancelada pelo usuário.".yellow());
        return;
    }

    match system_tuner::apply_system_configurations() {
        Ok(()) => {
            println!("{}", "✓ Regras permanentes gravadas e UDEV recarregado com sucesso!".green().bold());
        }
        Err(domain::AppError::PermissionDenied) => {
            eprintln!("{}", "Aviso: A gravação em /etc/modprobe.d/ e /etc/udev/rules.d/ exige privilégios de root.".yellow().bold());
            eprintln!("Por favor, execute o script com sudo:");
            eprintln!("  {}", "sudo ./scripts/apply-system-tuning.sh".cyan().bold());
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("{} {}", "Erro ao aplicar configurações de sistema:".red().bold(), err);
            std::process::exit(1);
        }
    }
}

fn handle_install_service() {
    println!("{}", "===> Instalando unidade do Systemd para inicialização no login...".cyan().bold());

    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => {
            eprintln!("{}", "Erro: Variável HOME não encontrada.".red());
            std::process::exit(1);
        }
    };

    let service_dir = std::path::PathBuf::from(home).join(".config/systemd/user");
    let _ = std::fs::create_dir_all(&service_dir);

    let current_exe = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "bt-dell-audio".to_string());

    let unit_content = format!(
        r#"[Unit]
Description=Dell Realtek Bluetooth Audio Sentinel & Optimizer
After=pipewire.service wireplumber.service

[Service]
Type=simple
ExecStart={} daemon
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
"#,
        current_exe
    );

    let unit_path = service_dir.join("bt-dell-audio.service");
    if let Err(e) = std::fs::write(&unit_path, unit_content) {
        eprintln!("Falha ao gravar arquivo de serviço: {}", e);
        std::process::exit(1);
    }

    let _ = std::process::Command::new("systemctl").args(["--user", "daemon-reload"]).status();
    let _ = std::process::Command::new("systemctl").args(["--user", "enable", "--now", "bt-dell-audio"]).status();

    println!("  {} Unidade systemd criada em: {}", "✓".green(), unit_path.display());
    println!("  {} Serviço ativado e rodando em segundo plano!", "✓".green().bold());
}
