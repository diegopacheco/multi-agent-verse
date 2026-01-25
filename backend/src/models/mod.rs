use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Pending,
    Running,
    Done,
    Error,
    Timeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentRole {
    TaskSplitter,
    Coordinator,
    Worker,
    Tester,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub role: AgentRole,
    pub model: String,
    pub status: AgentStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_dir: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Testing,
    Done,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskType {
    Parallel,
    Sequential,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub task_type: TaskType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_worker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_tester: Option<String>,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitTasksResult {
    pub parallel_tasks: Vec<TaskDefinition>,
    pub sequential_tasks: Vec<TaskDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDefinition {
    pub id: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub project_name: String,
    pub prompt: String,
    pub model: String,
    pub cli_agent: String,
    pub task_splitter: AgentInfo,
    pub coordinator: AgentInfo,
    pub workers: Vec<AgentInfo>,
    pub testers: Vec<AgentInfo>,
    pub tasks: Vec<Task>,
    pub events: Vec<Event>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub model: String,
    pub cli_agent: String,
    pub worker_count: usize,
    pub tester_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRequest {
    pub prompt: String,
    pub project_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResponse {
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub task_splitter: AgentInfo,
    pub coordinator: AgentInfo,
    pub workers: Vec<AgentInfo>,
    pub testers: Vec<AgentInfo>,
    pub progress: f64,
    pub elapsed_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsResponse {
    pub logs: String,
    pub status: AgentStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub duration: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksResponse {
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsResponse {
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub session_id: String,
    pub project_name: String,
    pub model: String,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub total_duration_secs: i64,
    pub created_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectsResponse {
    pub projects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl AgentInfo {
    pub fn new(id: String, role: AgentRole, model: String) -> Self {
        Self {
            id,
            role,
            model,
            status: AgentStatus::Pending,
            worktree: None,
            log_dir: None,
            started_at: None,
            finished_at: None,
        }
    }

    pub fn new_with_log_dir(id: String, role: AgentRole, model: String, log_dir: PathBuf) -> Self {
        Self {
            id,
            role,
            model,
            status: AgentStatus::Pending,
            worktree: None,
            log_dir: Some(log_dir),
            started_at: None,
            finished_at: None,
        }
    }

    pub fn duration_secs(&self) -> i64 {
        match (self.started_at, self.finished_at) {
            (Some(start), Some(end)) => (end - start).num_seconds(),
            (Some(start), None) => (Utc::now() - start).num_seconds(),
            _ => 0,
        }
    }
}

impl Task {
    pub fn new_parallel(id: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            description,
            task_type: TaskType::Parallel,
            order: None,
            assigned_worker: None,
            assigned_tester: None,
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn new_sequential(id: String, description: String, order: u32) -> Self {
        let now = Utc::now();
        Self {
            id,
            description,
            task_type: TaskType::Sequential,
            order: Some(order),
            assigned_worker: None,
            assigned_tester: None,
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Event {
    pub fn info(message: String, agent_id: Option<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            agent_id,
            message,
        }
    }

    pub fn error(message: String, agent_id: Option<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level: "ERROR".to_string(),
            agent_id,
            message,
        }
    }
}

impl Session {
    pub fn progress(&self) -> f64 {
        if self.tasks.is_empty() {
            return 0.0;
        }
        let total_weight: f64 = self
            .tasks
            .iter()
            .map(|t| match t.status {
                TaskStatus::Pending => 0.0,
                TaskStatus::InProgress => 0.33,
                TaskStatus::Testing => 0.66,
                TaskStatus::Done => 1.0,
                TaskStatus::Failed => 1.0,
            })
            .sum();
        (total_weight / self.tasks.len() as f64) * 100.0
    }

    pub fn elapsed_secs(&self) -> i64 {
        (Utc::now() - self.created_at).num_seconds()
    }
}
