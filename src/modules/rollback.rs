/// Module: rpm-ostree rollback.
use crate::config::Config;
use crate::color::{BOLD, DIM, RESET, _CYAN, _GREEN, _RED, _YELLOW};
use crate::runner::{command_exists, run_cmd, run_cmd_output, RunResult};
use crate::state::State;
use crate::ui::{bare_prompt, box_line, info_box, section, section_end};
use crate::modules::rpm_ostree::json_val;

pub fn run(cfg: &Config, state: &mut State) -> bool {
    if !command_exists("rpm-ostree") {
        box_line(&format!("{}✘ rpm-ostree not found{}", _RED.code(), RESET.code()));
        return false;
    }

    let json_str = match run_cmd_output(&["rpm-ostree", "status", "--json"], cfg) {
        Ok(s) => s,
        Err(_) => {
            box_line(&format!("{}✘ failed to query rpm-ostree status{}", _RED.code(), RESET.code()));
            return false;
        }
    };

    let val = match json_val(&json_str) {
        Some(v) => v,
        None => {
            box_line(&format!("{}✘ failed to parse rpm-ostree status JSON{}", _RED.code(), RESET.code()));
            return false;
        }
    };

    let deployments = match val["deployments"].as_array() {
        Some(d) => d,
        None => {
            box_line(&format!("{}✘ no deployments found in rpm-ostree status{}", _RED.code(), RESET.code()));
            return false;
        }
    };

    if deployments.len() < 2 {
        box_line(&format!("{}◆ no previous deployment available to roll back to{}", _YELLOW.code(), RESET.code()));
        return false;
    }

    let booted_idx = deployments
        .iter()
        .position(|d| d["booted"].as_bool().unwrap_or(false))
        .unwrap_or(0);
    let target_idx = if booted_idx == 0 { 1 } else { 0 };

    let current_version  = deployments[booted_idx]["version"].as_str().unwrap_or("unknown");
    let previous_version = deployments[target_idx]["version"].as_str().unwrap_or("unknown");

    let from_line = format!("{}from{}  {current_version}", DIM.code(), RESET.code());
    let to_line   = format!("{}to{}     {previous_version}", DIM.code(), RESET.code());
    info_box(
        &format!("{cy}{bold}* Rollback{r}", cy = _CYAN.code(), bold = BOLD.code(), r = RESET.code()),
        10,
        &[&from_line, &to_line],
    );

    let ans = bare_prompt("proceed with rollback? [y/N]");
    if !matches!(ans.trim(), "y" | "Y") {
        println!("  rollback cancelled");
        return true;
    }

    section("Rolling back");
    match run_cmd(&["sudo", "rpm-ostree", "rollback"], cfg) {
        RunResult::DryRun | RunResult::Success => {}
        RunResult::Failed(s) => {
            box_line(&format!("{}✘ rpm-ostree rollback failed (exit {:?}){}", _RED.code(), s.code(), RESET.code()));
            section_end();
            return false;
        }
        RunResult::Error(e) => {
            box_line(&format!("{}✘ rpm-ostree rollback error: {e}{}", _RED.code(), RESET.code()));
            section_end();
            return false;
        }
    }

    state.reboot_required = true;
    box_line(&format!("{}✔ rollback staged successfully{}", _GREEN.code(), RESET.code()));
    box_line(&format!("{}◆ reboot required to boot into previous deployment{}", _YELLOW.code(), RESET.code()));
    section_end();
    true
}