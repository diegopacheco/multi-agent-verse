pub mod claude;
pub mod codex;
pub mod copilot;
pub mod gemini;
pub mod llr3;

use crate::models::{AgentInfo, AgentStatus};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type SharedAgentState = Arc<Mutex<AgentInfo>>;

pub fn create_shared_agent(agent: AgentInfo) -> SharedAgentState {
    Arc::new(Mutex::new(agent))
}

pub async fn run_agent(
    cli_agent: &str,
    state: SharedAgentState,
    prompt: &str,
    model: &str,
    worktree: &PathBuf,
) {
    let log_dir = {
        let s = state.lock().await;
        s.log_dir.clone().unwrap_or_else(|| worktree.clone())
    };
    match cli_agent {
        "claude-code" => claude::run(state, prompt, model, worktree, &log_dir).await,
        "codex" => codex::run(state, prompt, model, worktree, &log_dir).await,
        "copilot" => copilot::run(state, prompt, model, worktree, &log_dir).await,
        "gemini" => gemini::run(state, prompt, model, worktree, &log_dir).await,
        "llr3" => llr3::run(state, prompt, model, worktree, &log_dir).await,
        _ => {
            let mut s = state.lock().await;
            s.status = AgentStatus::Error;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AgentRole;

    #[test]
    fn test_create_shared_agent() {
        let agent = AgentInfo::new("worker-1".to_string(), AgentRole::Worker, "opus-4-5".to_string());
        let shared = create_shared_agent(agent);
        assert!(Arc::strong_count(&shared) == 1);
    }

    #[tokio::test]
    async fn test_shared_agent_lock() {
        let agent = AgentInfo::new("worker-1".to_string(), AgentRole::Worker, "opus-4-5".to_string());
        let shared = create_shared_agent(agent);
        let locked = shared.lock().await;
        assert_eq!(locked.id, "worker-1");
        assert_eq!(locked.status, AgentStatus::Pending);
    }

    #[tokio::test]
    async fn test_run_agent_unknown_cli_sets_error() {
        let agent = AgentInfo::new("test".to_string(), AgentRole::Worker, "model".to_string());
        let shared = create_shared_agent(agent);
        let worktree = PathBuf::from("/tmp");
        run_agent("unknown-cli", shared.clone(), "prompt", "model", &worktree).await;
        let locked = shared.lock().await;
        assert_eq!(locked.status, AgentStatus::Error);
    }

    #[test]
    fn test_shared_agent_state_type() {
        let agent = AgentInfo::new("test".to_string(), AgentRole::Tester, "model".to_string());
        let shared: SharedAgentState = create_shared_agent(agent);
        assert!(Arc::strong_count(&shared) >= 1);
    }
}
