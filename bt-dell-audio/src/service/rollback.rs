use std::thread::sleep;
use crate::audio::wireplumber;
use crate::config::constants::timing;
use crate::domain::AppResult;

pub struct RollbackResult {
    pub wireplumber_reverted: bool,
}

pub fn execute_user_rollback() -> AppResult<RollbackResult> {
    let wireplumber_reverted = wireplumber::restore_original_config()?;
    if wireplumber_reverted {
        wireplumber::restart_wireplumber()?;
        sleep(timing::SERVICE_RELOAD_DELAY);
    }

    Ok(RollbackResult {
        wireplumber_reverted,
    })
}
