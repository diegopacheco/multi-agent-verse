use crate::agents::SharedAgentState;
use crate::models::AgentStatus;
use chrono::Utc;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::process::Command;
use tokio::time::timeout;

const TIMEOUT_SECS: u64 = 300;

pub async fn run(state: SharedAgentState, prompt: &str, _model: &str, worktree: &PathBuf, log_dir: &PathBuf) {
    {
        let mut s = state.lock().await;
        s.status = AgentStatus::Running;
        s.started_at = Some(Utc::now());
        s.worktree = Some(worktree.clone());
    }
    let log_path = log_dir.join("logs.txt");
    let prompt_path = log_dir.join("prompt.md");
    let _ = fs::write(&prompt_path, format!("# Prompt\n\n{}", prompt)).await;
    let _ = fs::write(&log_path, format!("=== RUNNING ===\nAgent started at {}\nModel: llama3 (local)\nWorking directory: {:?}\n\nExecuting: llr3 -p <prompt>\n\nWaiting for completion...\n", Utc::now().format("%Y-%m-%d %H:%M:%S"), worktree)).await;
    let result = timeout(
        Duration::from_secs(TIMEOUT_SECS),
        Command::new("llr3")
            .args(["-p", prompt])
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::create_shared_agent;
    use crate::models::{AgentInfo, AgentRole};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_llr3_run_sets_running_status() {
        let agent = AgentInfo::new("test-llr3".to_string(), AgentRole::Worker, "llama3".to_string());
        let state = create_shared_agent(agent);
        let temp_dir = tempdir().unwrap();
        let worktree = temp_dir.path().to_path_buf();
        let log_dir = temp_dir.path().to_path_buf();
        run(state.clone(), "test prompt", "llama3", &worktree, &log_dir).await;
        let s = state.lock().await;
        assert!(s.started_at.is_some());
        assert!(s.finished_at.is_some());
    }

    #[tokio::test]
    async fn test_llr3_creates_prompt_file() {
        let agent = AgentInfo::new("test-llr3".to_string(), AgentRole::Worker, "llama3".to_string());
        let state = create_shared_agent(agent);
        let temp_dir = tempdir().unwrap();
        let worktree = temp_dir.path().to_path_buf();
        let log_dir = temp_dir.path().to_path_buf();
        run(state, "test prompt", "llama3", &worktree, &log_dir).await;
        assert!(log_dir.join("prompt.md").exists());
    }

    #[tokio::test]
    async fn test_llr3_creates_logs_file() {
        let agent = AgentInfo::new("test-llr3".to_string(), AgentRole::Worker, "llama3".to_string());
        let state = create_shared_agent(agent);
        let temp_dir = tempdir().unwrap();
        let worktree = temp_dir.path().to_path_buf();
        let log_dir = temp_dir.path().to_path_buf();
        run(state, "test prompt", "llama3", &worktree, &log_dir).await;
        assert!(log_dir.join("logs.txt").exists());
    }
}
