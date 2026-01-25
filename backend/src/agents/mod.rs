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
