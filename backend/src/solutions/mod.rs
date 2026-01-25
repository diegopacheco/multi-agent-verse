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

pub async fn list_projects() -> Result<Vec<String>, String> {
    let solutions_path = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("Cannot get parent directory")?
        .join("solutions");
    let mut projects = Vec::new();
    let mut entries = fs::read_dir(&solutions_path)
        .await
        .map_err(|e| e.to_string())?;
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        if entry.file_type().await.map_err(|e| e.to_string())?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                projects.push(name.to_string());
            }
        }
    }
    projects.sort();
    Ok(projects)
}

pub async fn read_agent_logs(base_path: &PathBuf, agent_id: &str) -> Result<String, String> {
    let log_path = base_path.join(agent_id).join("logs.txt");
    fs::read_to_string(&log_path)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Task;

    #[test]
    fn test_task_status_done_filter() {
        let tasks = vec![
            Task::new_parallel("1".to_string(), "Task 1".to_string()),
            Task::new_parallel("2".to_string(), "Task 2".to_string()),
        ];
        let completed = tasks.iter().filter(|t| t.status == TaskStatus::Done).count();
        assert_eq!(completed, 0);
    }

    #[test]
    fn test_task_status_pending_filter() {
        let tasks = vec![
            Task::new_parallel("1".to_string(), "Task 1".to_string()),
            Task::new_sequential("2".to_string(), "Task 2".to_string(), 1),
        ];
        let pending = tasks.iter().filter(|t| t.status == TaskStatus::Pending).count();
        assert_eq!(pending, 2);
    }

    #[test]
    fn test_summary_struct_creation() {
        let summary = Summary {
            session_id: "test-session".to_string(),
            project_name: "test-project".to_string(),
            model: "opus-4-5".to_string(),
            total_tasks: 5,
            completed_tasks: 3,
            failed_tasks: 1,
            total_duration_secs: 120,
            created_at: Utc::now(),
            finished_at: Utc::now(),
        };
        assert_eq!(summary.session_id, "test-session");
        assert_eq!(summary.total_tasks, 5);
        assert_eq!(summary.completed_tasks, 3);
        assert_eq!(summary.failed_tasks, 1);
    }

    #[test]
    fn test_event_format() {
        let event = Event::info("Test message".to_string(), Some("agent-1".to_string()));
        let line = format!(
            "[{}] [{}] {} - {}",
            event.timestamp.format("%Y-%m-%d %H:%M:%S"),
            event.level,
            event.agent_id.as_deref().unwrap_or("system"),
            event.message
        );
        assert!(line.contains("INFO"));
        assert!(line.contains("agent-1"));
        assert!(line.contains("Test message"));
    }

    #[test]
    fn test_event_format_no_agent() {
        let event = Event::error("Error occurred".to_string(), None);
        let agent_str = event.agent_id.as_deref().unwrap_or("system");
        assert_eq!(agent_str, "system");
    }
}
