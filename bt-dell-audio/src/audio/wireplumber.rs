use std::fs;
use std::path::PathBuf;
use std::process::Command;
use crate::config::constants::paths;
use crate::config::templates::WIREPLUMBER_BLUETOOTH_CONF;
use crate::domain::{AppError, AppResult};

pub fn get_wireplumber_config_path() -> AppResult<PathBuf> {
    let home = std::env::var("HOME").map_err(|_| AppError::EnvVarMissing("HOME".to_string()))?;
    Ok(PathBuf::from(home)
        .join(paths::WIREPLUMBER_CONFIG_DIR_REL)
        .join(paths::WIREPLUMBER_BT_TWEAKS_FILE))
}

pub fn is_autoswitch_profile_enabled() -> bool {
    if let Ok(path) = get_wireplumber_config_path()
        && let Ok(content) = fs::read_to_string(path)
    {
        return content.contains("bluetooth.autoswitch-to-headset-profile = true");
    }
    false
}

pub fn write_optimized_config() -> AppResult<PathBuf> {
    let target_path = get_wireplumber_config_path()?;
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::Io {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    // Create backup once if needed
    let backup_path = target_path.with_extension(format!("conf{}", paths::WIREPLUMBER_BACKUP_SUFFIX));
    if target_path.exists() && !backup_path.exists() {
        let _ = fs::copy(&target_path, &backup_path);
    }

    fs::write(&target_path, WIREPLUMBER_BLUETOOTH_CONF).map_err(|e| AppError::Io {
        path: target_path.clone(),
        source: e,
    })?;

    Ok(target_path)
}

pub fn restore_original_config() -> AppResult<bool> {
    let target_path = get_wireplumber_config_path()?;
    let backup_path = target_path.with_extension(format!("conf{}", paths::WIREPLUMBER_BACKUP_SUFFIX));

    if backup_path.exists() {
        fs::copy(&backup_path, &target_path).map_err(|e| AppError::Io {
            path: target_path.clone(),
            source: e,
        })?;
        let _ = fs::remove_file(&backup_path);
        Ok(true)
    } else if target_path.exists() {
        fs::remove_file(&target_path).map_err(|e| AppError::Io {
            path: target_path.clone(),
            source: e,
        })?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn restart_wireplumber() -> AppResult<()> {
    let status = Command::new("systemctl")
        .args(["--user", "restart", "wireplumber"])
        .status()
        .map_err(|e| AppError::CommandExecution {
            command: "systemctl --user restart wireplumber".to_string(),
            details: e.to_string(),
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::CommandExecution {
            command: "systemctl --user restart wireplumber".to_string(),
            details: format!("Código de saída: {:?}", status.code()),
        })
    }
}
