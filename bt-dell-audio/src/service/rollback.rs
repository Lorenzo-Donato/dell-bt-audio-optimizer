use std::thread::sleep;
use crate::audio::wireplumber;
use crate::config::constants::timing;
use crate::domain::AppResult;
use crate::service::installer;

pub struct RollbackResult {
    pub wireplumber_reverted: bool,
    pub service_uninstalled: bool,
}

pub fn execute_user_rollback() -> AppResult<RollbackResult> {
    let wireplumber_reverted = wireplumber::restore_original_config()?;
    if wireplumber_reverted {
        wireplumber::restart_wireplumber()?;
        sleep(timing::SERVICE_RELOAD_DELAY);
    }

    let service_uninstalled = installer::uninstall_user_service().is_ok();

    Ok(RollbackResult {
        wireplumber_reverted,
        service_uninstalled,
    })
}
