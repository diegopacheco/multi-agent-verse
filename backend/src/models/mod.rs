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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_info_new() {
        let agent = AgentInfo::new("worker-1".to_string(), AgentRole::Worker, "opus-4-5".to_string());
        assert_eq!(agent.id, "worker-1");
        assert_eq!(agent.role, AgentRole::Worker);
        assert_eq!(agent.model, "opus-4-5");
        assert_eq!(agent.status, AgentStatus::Pending);
        assert!(agent.worktree.is_none());
        assert!(agent.started_at.is_none());
        assert!(agent.finished_at.is_none());
    }

    #[test]
    fn test_agent_info_new_with_log_dir() {
        let log_dir = PathBuf::from("/tmp/logs");
        let agent = AgentInfo::new_with_log_dir("tester-1".to_string(), AgentRole::Tester, "gpt-5.2".to_string(), log_dir.clone());
        assert_eq!(agent.id, "tester-1");
        assert_eq!(agent.role, AgentRole::Tester);
        assert_eq!(agent.log_dir, Some(log_dir));
    }

    #[test]
    fn test_agent_duration_secs_not_started() {
        let agent = AgentInfo::new("test".to_string(), AgentRole::Worker, "model".to_string());
        assert_eq!(agent.duration_secs(), 0);
    }

    #[test]
    fn test_task_new_parallel() {
        let task = Task::new_parallel("1".to_string(), "Test task".to_string());
        assert_eq!(task.id, "1");
        assert_eq!(task.description, "Test task");
        assert_eq!(task.task_type, TaskType::Parallel);
        assert!(task.order.is_none());
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.assigned_worker.is_none());
        assert!(task.assigned_tester.is_none());
    }

    #[test]
    fn test_task_new_sequential() {
        let task = Task::new_sequential("2".to_string(), "Sequential task".to_string(), 5);
        assert_eq!(task.id, "2");
        assert_eq!(task.description, "Sequential task");
        assert_eq!(task.task_type, TaskType::Sequential);
        assert_eq!(task.order, Some(5));
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_event_info() {
        let event = Event::info("Test message".to_string(), Some("agent-1".to_string()));
        assert_eq!(event.level, "INFO");
        assert_eq!(event.message, "Test message");
        assert_eq!(event.agent_id, Some("agent-1".to_string()));
    }

    #[test]
    fn test_event_error() {
        let event = Event::error("Error message".to_string(), None);
        assert_eq!(event.level, "ERROR");
        assert_eq!(event.message, "Error message");
        assert!(event.agent_id.is_none());
    }

    #[test]
    fn test_session_progress_empty_tasks() {
        let session = Session {
            id: "test".to_string(),
            project_name: "test-project".to_string(),
            prompt: "test prompt".to_string(),
            model: "opus-4-5".to_string(),
            cli_agent: "claude-code".to_string(),
            task_splitter: AgentInfo::new("task-splitter".to_string(), AgentRole::TaskSplitter, "opus-4-5".to_string()),
            coordinator: AgentInfo::new("coordinator".to_string(), AgentRole::Coordinator, "opus-4-5".to_string()),
            workers: vec![],
            testers: vec![],
            tasks: vec![],
            events: vec![],
            created_at: Utc::now(),
        };
        assert_eq!(session.progress(), 0.0);
    }

    #[test]
    fn test_session_progress_all_pending() {
        let mut session = Session {
            id: "test".to_string(),
            project_name: "test-project".to_string(),
            prompt: "test prompt".to_string(),
            model: "opus-4-5".to_string(),
            cli_agent: "claude-code".to_string(),
            task_splitter: AgentInfo::new("task-splitter".to_string(), AgentRole::TaskSplitter, "opus-4-5".to_string()),
            coordinator: AgentInfo::new("coordinator".to_string(), AgentRole::Coordinator, "opus-4-5".to_string()),
            workers: vec![],
            testers: vec![],
            tasks: vec![],
            events: vec![],
            created_at: Utc::now(),
        };
        session.tasks.push(Task::new_parallel("1".to_string(), "Task 1".to_string()));
        session.tasks.push(Task::new_parallel("2".to_string(), "Task 2".to_string()));
        assert_eq!(session.progress(), 0.0);
    }

    #[test]
    fn test_session_progress_all_done() {
        let mut session = Session {
            id: "test".to_string(),
            project_name: "test-project".to_string(),
            prompt: "test prompt".to_string(),
            model: "opus-4-5".to_string(),
            cli_agent: "claude-code".to_string(),
            task_splitter: AgentInfo::new("task-splitter".to_string(), AgentRole::TaskSplitter, "opus-4-5".to_string()),
            coordinator: AgentInfo::new("coordinator".to_string(), AgentRole::Coordinator, "opus-4-5".to_string()),
            workers: vec![],
            testers: vec![],
            tasks: vec![],
            events: vec![],
            created_at: Utc::now(),
        };
        let mut task1 = Task::new_parallel("1".to_string(), "Task 1".to_string());
        task1.status = TaskStatus::Done;
        let mut task2 = Task::new_parallel("2".to_string(), "Task 2".to_string());
        task2.status = TaskStatus::Done;
        session.tasks.push(task1);
        session.tasks.push(task2);
        assert_eq!(session.progress(), 100.0);
    }

    #[test]
    fn test_session_progress_mixed() {
        let mut session = Session {
            id: "test".to_string(),
            project_name: "test-project".to_string(),
            prompt: "test prompt".to_string(),
            model: "opus-4-5".to_string(),
            cli_agent: "claude-code".to_string(),
            task_splitter: AgentInfo::new("task-splitter".to_string(), AgentRole::TaskSplitter, "opus-4-5".to_string()),
            coordinator: AgentInfo::new("coordinator".to_string(), AgentRole::Coordinator, "opus-4-5".to_string()),
            workers: vec![],
            testers: vec![],
            tasks: vec![],
            events: vec![],
            created_at: Utc::now(),
        };
        let mut task1 = Task::new_parallel("1".to_string(), "Task 1".to_string());
        task1.status = TaskStatus::Done;
        let task2 = Task::new_parallel("2".to_string(), "Task 2".to_string());
        session.tasks.push(task1);
        session.tasks.push(task2);
        assert_eq!(session.progress(), 50.0);
    }

    #[test]
    fn test_split_tasks_result_serialization() {
        let result = SplitTasksResult {
            parallel_tasks: vec![
                TaskDefinition { id: "1".to_string(), description: "Task 1".to_string(), order: None },
            ],
            sequential_tasks: vec![
                TaskDefinition { id: "2".to_string(), description: "Task 2".to_string(), order: Some(1) },
            ],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("parallel_tasks"));
        assert!(json.contains("sequential_tasks"));
        let parsed: SplitTasksResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.parallel_tasks.len(), 1);
        assert_eq!(parsed.sequential_tasks.len(), 1);
        assert_eq!(parsed.sequential_tasks[0].order, Some(1));
    }

    #[test]
    fn test_agent_status_serialization() {
        assert_eq!(serde_json::to_string(&AgentStatus::Pending).unwrap(), "\"pending\"");
        assert_eq!(serde_json::to_string(&AgentStatus::Running).unwrap(), "\"running\"");
        assert_eq!(serde_json::to_string(&AgentStatus::Done).unwrap(), "\"done\"");
        assert_eq!(serde_json::to_string(&AgentStatus::Error).unwrap(), "\"error\"");
        assert_eq!(serde_json::to_string(&AgentStatus::Timeout).unwrap(), "\"timeout\"");
    }

    #[test]
    fn test_task_type_serialization() {
        assert_eq!(serde_json::to_string(&TaskType::Parallel).unwrap(), "\"parallel\"");
        assert_eq!(serde_json::to_string(&TaskType::Sequential).unwrap(), "\"sequential\"");
    }

    #[test]
    fn test_task_status_serialization() {
        assert_eq!(serde_json::to_string(&TaskStatus::Pending).unwrap(), "\"pending\"");
        assert_eq!(serde_json::to_string(&TaskStatus::InProgress).unwrap(), "\"inprogress\"");
        assert_eq!(serde_json::to_string(&TaskStatus::Testing).unwrap(), "\"testing\"");
        assert_eq!(serde_json::to_string(&TaskStatus::Done).unwrap(), "\"done\"");
        assert_eq!(serde_json::to_string(&TaskStatus::Failed).unwrap(), "\"failed\"");
    }
}
