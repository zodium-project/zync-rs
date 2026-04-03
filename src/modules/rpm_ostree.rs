/// Module: rpm-ostree system update.
use serde_json::Value;
use crate::color::{RESET, _RED, _YELLOW};
use crate::config::Config;
use crate::ui::box_line;
use crate::runner::{command_exists, run_cmd, run_cmd_output, RunResult};
use crate::state::State;

pub fn run(cfg: &Config, state: &mut State) -> bool {
    if !command_exists("rpm-ostree") {
        box_line(&format!("{}✘ rpm-ostree not found{}", _RED.code(), RESET.code()));
        return false;
    }

    box_line("Checking for rpm-ostree upgrade...");

    match run_cmd(&["rpm-ostree", "upgrade"], cfg) {
        RunResult::DryRun | RunResult::Success => {}
        _ => return false,
    }

    if !cfg.dry_run {
        let json_str = match run_cmd_output(&["rpm-ostree", "status", "--json"], cfg) {
            Ok(s) => s,
            Err(_) => {
                box_line(&format!("{}✘ failed to query rpm-ostree status{}", _RED.code(), RESET.code()));
                return false;
            }
        };

        if let Ok(val) = serde_json::from_str::<Value>(&json_str) {
            let deployments = val["deployments"].as_array();
            let staged_count = deployments
                .map(|d| d.iter().filter(|v| v["staged"].as_bool().unwrap_or(false)).count())
                .unwrap_or(0);

            if staged_count > 0 {
                let booted_ver = deployment_version(&val, "booted");
                let staged_ver = deployment_version(&val, "staged");
                state.reboot_required = true;
                box_line(&format!("Upgrade staged: {booted_ver} → {staged_ver}"));
                box_line(&format!("{}◆ reboot required{}", _YELLOW.code(), RESET.code()));
            } else {
                box_line("Already up to date; no new deployment staged");
            }
        }
    }

    if cfg.maintain {
        run_cmd(&["rpm-ostree", "cleanup", "-m"], cfg);
    }

    true
}

fn deployment_version(val: &Value, field: &str) -> String {
    val["deployments"]
        .as_array()
        .and_then(|d| d.iter().find(|v| v[field].as_bool().unwrap_or(false)))
        .and_then(|v| v["version"].as_str())
        .unwrap_or("unknown")
        .to_owned()
}

pub(crate) fn json_val(json: &str) -> Option<Value> {
    serde_json::from_str(json).ok()
}