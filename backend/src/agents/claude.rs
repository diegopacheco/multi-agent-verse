use crate::agents::SharedAgentState;
use crate::models::AgentStatus;
use chrono::Utc;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::process::Command;
use tokio::time::timeout;

const TIMEOUT_SECS: u64 = 300;

pub async fn run(state: SharedAgentState, prompt: &str, model: &str, worktree: &PathBuf) {
    {
        let mut s = state.lock().await;
        s.status = AgentStatus::Running;
        s.started_at = Some(Utc::now());
        s.worktree = Some(worktree.clone());
    }
    let log_path = worktree.join("logs.txt");
    let prompt_path = worktree.join("prompt.md");
    let _ = fs::write(&prompt_path, format!("# Prompt\n\n{}", prompt)).await;
    let result = timeout(
        Duration::from_secs(TIMEOUT_SECS),
        Command::new("claude")
            .args(["-p", prompt, "--model", model, "--dangerously-skip-permissions"])
            .current_dir(worktree)
            .output(),
    )
    .await;
    let mut s = state.lock().await;
    s.finished_at = Some(Utc::now());
    match result {
        Ok(Ok(output)) => {
            let logs = format!(
                "=== STDOUT ===\n{}\n=== STDERR ===\n{}\n=== EXIT CODE: {} ===\n",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
                output.status.code().unwrap_or(-1)
            );
            let _ = fs::write(&log_path, logs).await;
            if output.status.success() {
                s.status = AgentStatus::Done;
            } else {
                s.status = AgentStatus::Error;
            }
        }
        Ok(Err(e)) => {
            let _ = fs::write(&log_path, format!("Error: {}", e)).await;
            s.status = AgentStatus::Error;
        }
        Err(_) => {
            let _ = fs::write(&log_path, "Timeout after 5 minutes").await;
            s.status = AgentStatus::Timeout;
        }
    }
}
