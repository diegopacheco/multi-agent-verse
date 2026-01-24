pub mod claude;
pub mod codex;
pub mod copilot;
pub mod gemini;

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
    match cli_agent {
        "claude-code" => claude::run(state, prompt, model, worktree).await,
        "codex" => codex::run(state, prompt, model, worktree).await,
        "copilot" => copilot::run(state, prompt, model, worktree).await,
        "gemini" => gemini::run(state, prompt, model, worktree).await,
        _ => {
            let mut s = state.lock().await;
            s.status = AgentStatus::Error;
        }
    }
}
