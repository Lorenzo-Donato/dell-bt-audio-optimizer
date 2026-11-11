use std::thread::sleep;
use crate::audio::pipewire;
use crate::config::constants::{audio, timing};
use crate::diagnostics::run_full_diagnosis;
use crate::hardware::usb;
use crate::service::circuit_breaker::CircuitBreaker;
use crate::ui::console;

pub fn start_interactive_watch_loop(anonymize: bool) {
    loop {
        // Clear screen with standard ANSI escape
        print!("\x1B[2J\x1B[1;1H");

        let report = run_full_diagnosis();
        console::render_dashboard(&report, anonymize);

        sleep(timing::WATCH_REFRESH_INTERVAL);
    }
}

pub fn start_background_daemon() {
    println!("[SENTINEL] Serviço sentinela ativo com airbag de proteção e circuit breaker.");

    let mut airbag = CircuitBreaker::default_airbag();

    loop {
        if !airbag.can_execute() {
            if let Some(remaining) = airbag.remaining_cooldown() {
                eprintln!(
                    "[AIRBAG DISPARADO] Proteção do sistema ativa após falhas repetidas. Pausando por {}s para prevenir travamentos.",
                    remaining.as_secs()
                );
            }
            sleep(timing::DAEMON_POLL_INTERVAL);
            continue;
        }

        // 1. Maintain USB power control in 'on' state on dynamically discovered Bluetooth controller
        if let Some(dev) = usb::find_bluetooth_usb_device()
            && dev.power_control.is_power_saving_active()
        {
            if let Err(e) = usb::set_usb_power_control_on(&dev.sysfs_path) {
                eprintln!("[SENTINEL AVISO] Falha ao ajustar power/control: {}. Registrando no airbag.", e);
                airbag.record_failure();
            } else {
                airbag.record_success();
            }
        }

        // 2. Auto-heal any fallback to choked SBC-XQ or invalid state
        let devices = pipewire::probe_bluetooth_devices();
        for dev in devices {
            if dev.active_codec.is_high_bandwidth_risk()
                && let Some(card_name) = &dev.card_name
            {
                println!("[SENTINEL] Detectado codec SBC-XQ com risco de sobrecarga em '{}'. Auto-recuperando...", dev.name);
                if let Err(e) = pipewire::set_card_profile(card_name, audio::CODEC_SBC) {
                    eprintln!("[SENTINEL AVISO] Falha ao recuperar perfil de áudio: {}. Registrando no airbag.", e);
                    airbag.record_failure();
                } else {
                    airbag.record_success();
                }
            }
        }

        sleep(timing::DAEMON_POLL_INTERVAL);
    }
}
