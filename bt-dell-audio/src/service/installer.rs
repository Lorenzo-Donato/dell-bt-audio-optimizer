use std::fs;
use std::path::PathBuf;
use std::process::Command;
use crate::config::templates::SYSTEMD_USER_SERVICE;
use crate::domain::{AppError, AppResult};

pub fn install_user_service() -> AppResult<PathBuf> {
    let home = std::env::var("HOME").map_err(|_| AppError::EnvVarMissing("HOME".to_string()))?;
    let service_dir = PathBuf::from(home).join(".config/systemd/user");

    fs::create_dir_all(&service_dir).map_err(|e| AppError::Io {
        path: service_dir.clone(),
        source: e,
    })?;

    let current_exe = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "bt-dell-audio".to_string());

    let unit_content = SYSTEMD_USER_SERVICE.replace("{{EXEC_PATH}}", &current_exe);
    let unit_path = service_dir.join("bt-dell-audio.service");

    fs::write(&unit_path, unit_content).map_err(|e| AppError::Io {
        path: unit_path.clone(),
        source: e,
    })?;

    let _ = Command::new("systemctl").args(["--user", "daemon-reload"]).status();
    let _ = Command::new("systemctl").args(["--user", "enable", "--now", "bt-dell-audio"]).status();

    Ok(unit_path)
}

#[allow(dead_code)]
pub fn uninstall_user_service() -> AppResult<()> {
    let home = std::env::var("HOME").map_err(|_| AppError::EnvVarMissing("HOME".to_string()))?;
    let unit_path = PathBuf::from(home).join(".config/systemd/user/bt-dell-audio.service");

    let _ = Command::new("systemctl").args(["--user", "stop", "bt-dell-audio"]).status();
    let _ = Command::new("systemctl").args(["--user", "disable", "bt-dell-audio"]).status();

    if unit_path.exists() {
        let _ = fs::remove_file(&unit_path);
    }

    let _ = Command::new("systemctl").args(["--user", "daemon-reload"]).status();
    Ok(())
}
