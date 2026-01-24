use crate::models::{Event, Session, Summary, Task, TaskStatus};
use chrono::Utc;
use std::path::PathBuf;
use tokio::fs;

pub async fn create_project_dir(project_name: &str) -> Result<PathBuf, String> {
    let base_path = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("Cannot get parent directory")?
        .join("solutions")
        .join(project_name);
    fs::create_dir_all(&base_path)
        .await
        .map_err(|e| e.to_string())?;
    Ok(base_path)
}

pub async fn create_agent_dir(base_path: &PathBuf, agent_id: &str) -> Result<PathBuf, String> {
    let agent_path = base_path.join(agent_id);
    fs::create_dir_all(&agent_path)
        .await
        .map_err(|e| e.to_string())?;
    Ok(agent_path)
}

pub async fn create_code_dir(base_path: &PathBuf) -> Result<PathBuf, String> {
    let code_path = base_path.join("code");
    fs::create_dir_all(&code_path)
        .await
        .map_err(|e| e.to_string())?;
    Ok(code_path)
}

pub async fn write_event_log(base_path: &PathBuf, event: &Event) -> Result<(), String> {
    let log_path = base_path.join("events.log");
    let line = format!(
        "[{}] [{}] {} - {}\n",
        event.timestamp.format("%Y-%m-%d %H:%M:%S"),
        event.level,
        event.agent_id.as_deref().unwrap_or("system"),
        event.message
    );
    let existing = fs::read_to_string(&log_path).await.unwrap_or_default();
    fs::write(&log_path, format!("{}{}", existing, line))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn write_tasks_json(base_path: &PathBuf, tasks: &[Task]) -> Result<(), String> {
    let tasks_path = base_path.join("tasks.json");
    let json = serde_json::to_string_pretty(tasks).map_err(|e| e.to_string())?;
    fs::write(&tasks_path, json)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn write_session_state(base_path: &PathBuf, session: &Session) -> Result<(), String> {
    let state_path = base_path.join("session.json");
    let json = serde_json::to_string_pretty(session).map_err(|e| e.to_string())?;
    fs::write(&state_path, json)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn write_summary(base_path: &PathBuf, session: &Session) -> Result<(), String> {
    let summary_path = base_path.join("summary.json");
    let completed = session
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Done)
        .count();
    let failed = session
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Failed)
        .count();
    let summary = Summary {
        session_id: session.id.clone(),
        project_name: session.project_name.clone(),
        model: session.model.clone(),
        total_tasks: session.tasks.len(),
        completed_tasks: completed,
        failed_tasks: failed,
        total_duration_secs: session.elapsed_secs(),
        created_at: session.created_at,
        finished_at: Utc::now(),
    };
    let json = serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())?;
    fs::write(&summary_path, json)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn read_agent_logs(base_path: &PathBuf, agent_id: &str) -> Result<String, String> {
    let log_path = base_path.join(agent_id).join("logs.txt");
    fs::read_to_string(&log_path)
        .await
        .map_err(|e| e.to_string())
}
