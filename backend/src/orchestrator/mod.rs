use crate::agents::{create_shared_agent, run_agent};
use crate::models::{AgentInfo, AgentRole, AgentStatus, Event, Session, Task, TaskStatus};
use crate::solutions::{
    create_agent_dir, create_project_dir, write_event_log, write_session_state, write_summary,
    write_tasks_json,
};
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedSession = Arc<RwLock<Session>>;

pub fn create_shared_session(session: Session) -> SharedSession {
    Arc::new(RwLock::new(session))
}

pub async fn run_orchestration(shared_session: SharedSession) {
    let (cli_agent, model, project_name, prompt) = {
        let session = shared_session.read().await;
        (
            session.cli_agent.clone(),
            session.model.clone(),
            session.project_name.clone(),
            session.prompt.clone(),
        )
    };
    let base_path = match create_project_dir(&project_name).await {
        Ok(p) => p,
        Err(e) => {
            let mut session = shared_session.write().await;
            session.task_splitter.status = AgentStatus::Error;
            let event = Event::error(format!("Failed to create project dir: {}", e), None);
            session.events.push(event);
            return;
        }
    };
    {
        let mut session = shared_session.write().await;
        let event = Event::info("Project directory created".to_string(), None);
        session.events.push(event.clone());
        let _ = write_event_log(&base_path, &event).await;
        let _ = write_session_state(&base_path, &session).await;
    }
    run_task_splitter(shared_session.clone(), &cli_agent, &model, &base_path, &prompt).await;
    let task_splitter_status = {
        let session = shared_session.read().await;
        session.task_splitter.status
    };
    if task_splitter_status != AgentStatus::Done {
        return;
    }
    run_coordinator(shared_session.clone(), &cli_agent, &model, &base_path).await;
    run_workers(shared_session.clone(), &cli_agent, &model, &base_path).await;
    run_testers(shared_session.clone(), &cli_agent, &model, &base_path).await;
    {
        let session = shared_session.read().await;
        let _ = write_summary(&base_path, &session).await;
        let _ = write_session_state(&base_path, &session).await;
    }
}

async fn run_task_splitter(
    shared_session: SharedSession,
    cli_agent: &str,
    model: &str,
    base_path: &PathBuf,
    user_prompt: &str,
) {
    let agent_id = {
        let session = shared_session.read().await;
        session.task_splitter.id.clone()
    };
    let worktree = match create_agent_dir(base_path, &agent_id).await {
        Ok(p) => p,
        Err(e) => {
            let mut session = shared_session.write().await;
            session.task_splitter.status = AgentStatus::Error;
            let event = Event::error(
                format!("Failed to create task-splitter dir: {}", e),
                Some(agent_id),
            );
            session.events.push(event);
            return;
        }
    };
    {
        let mut session = shared_session.write().await;
        session.task_splitter.status = AgentStatus::Running;
        session.task_splitter.started_at = Some(Utc::now());
        session.task_splitter.worktree = Some(worktree.clone());
        let event = Event::info("task-splitter started".to_string(), Some(agent_id.clone()));
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
        let _ = write_session_state(base_path, &session).await;
    }
    let worker_count = {
        let session = shared_session.read().await;
        session.workers.len()
    };
    let prompt = format!(
        "You are a task splitter agent. Analyze this project request and break it into exactly {} independent, parallel tasks.\n\n\
        IMPORTANT: Output ONLY valid JSON - no markdown, no explanation, just the JSON array.\n\n\
        Output format (JSON array):\n\
        [\n\
          {{\"id\": \"1\", \"description\": \"Brief but meaningful task description\"}},\n\
          {{\"id\": \"2\", \"description\": \"Another meaningful task description\"}}\n\
        ]\n\n\
        Project request:\n{}",
        worker_count,
        user_prompt
    );
    let agent_state = {
        let session = shared_session.read().await;
        create_shared_agent(session.task_splitter.clone())
    };
    run_agent(cli_agent, agent_state.clone(), &prompt, model, &worktree).await;
    let final_state = agent_state.lock().await;
    {
        let mut session = shared_session.write().await;
        session.task_splitter.status = final_state.status;
        session.task_splitter.finished_at = Some(Utc::now());
        let status_msg = match final_state.status {
            AgentStatus::Done => "completed",
            AgentStatus::Error => "failed",
            AgentStatus::Timeout => "timed out",
            _ => "unknown",
        };
        let event = Event::info(
            format!("task-splitter {}", status_msg),
            Some(agent_id.clone()),
        );
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
        if final_state.status == AgentStatus::Done {
            let tasks = parse_tasks_from_output(base_path, worker_count, user_prompt).await;
            session.tasks = tasks.clone();
            let event = Event::info(
                format!("{} tasks created", session.tasks.len()),
                Some(agent_id),
            );
            session.events.push(event.clone());
            let _ = write_event_log(base_path, &event).await;
            let _ = write_tasks_json(base_path, &session.tasks).await;
        }
        let _ = write_session_state(base_path, &session).await;
    }
}

async fn run_coordinator(
    shared_session: SharedSession,
    _cli_agent: &str,
    _model: &str,
    base_path: &PathBuf,
) {
    let agent_id = {
        let session = shared_session.read().await;
        session.coordinator.id.clone()
    };
    let worktree = match create_agent_dir(base_path, &agent_id).await {
        Ok(p) => p,
        Err(_) => return,
    };
    {
        let mut session = shared_session.write().await;
        session.coordinator.status = AgentStatus::Running;
        session.coordinator.started_at = Some(Utc::now());
        session.coordinator.worktree = Some(worktree.clone());
        let event = Event::info("coordinator started".to_string(), Some(agent_id.clone()));
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
        let _ = write_session_state(base_path, &session).await;
    }
    {
        let mut session = shared_session.write().await;
        let workers: Vec<String> = session.workers.iter().map(|w| w.id.clone()).collect();
        let testers: Vec<String> = session.testers.iter().map(|t| t.id.clone()).collect();
        for (i, task) in session.tasks.iter_mut().enumerate() {
            if i < workers.len() {
                task.assigned_worker = Some(workers[i].clone());
            }
            if !testers.is_empty() {
                task.assigned_tester = Some(testers[i % testers.len()].clone());
            }
        }
        session.coordinator.status = AgentStatus::Done;
        session.coordinator.finished_at = Some(Utc::now());
        let event = Event::info(
            "coordinator completed - tasks assigned".to_string(),
            Some(agent_id),
        );
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
        let _ = write_tasks_json(base_path, &session.tasks).await;
        let _ = write_session_state(base_path, &session).await;
    }
}

async fn run_workers(
    shared_session: SharedSession,
    cli_agent: &str,
    model: &str,
    base_path: &PathBuf,
) {
    let worker_tasks: Vec<(String, Vec<Task>)> = {
        let session = shared_session.read().await;
        session
            .workers
            .iter()
            .map(|w| {
                let tasks: Vec<Task> = session
                    .tasks
                    .iter()
                    .filter(|t| t.assigned_worker.as_ref() == Some(&w.id))
                    .cloned()
                    .collect();
                (w.id.clone(), tasks)
            })
            .collect()
    };
    let mut handles = Vec::new();
    for (worker_id, tasks) in worker_tasks {
        let shared = shared_session.clone();
        let cli = cli_agent.to_string();
        let m = model.to_string();
        let bp = base_path.clone();
        let handle = tokio::spawn(async move {
            run_single_worker(shared, &worker_id, &tasks, &cli, &m, &bp).await;
        });
        handles.push(handle);
    }
    for handle in handles {
        let _ = handle.await;
    }
}

async fn run_single_worker(
    shared_session: SharedSession,
    worker_id: &str,
    tasks: &[Task],
    cli_agent: &str,
    model: &str,
    base_path: &PathBuf,
) {
    let worktree = match create_agent_dir(base_path, worker_id).await {
        Ok(p) => p,
        Err(_) => return,
    };
    {
        let mut session = shared_session.write().await;
        if let Some(worker) = session.workers.iter_mut().find(|w| w.id == worker_id) {
            worker.status = AgentStatus::Running;
            worker.started_at = Some(Utc::now());
            worker.worktree = Some(worktree.clone());
        }
        let event = Event::info(format!("{} started", worker_id), Some(worker_id.to_string()));
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
        let _ = write_session_state(base_path, &session).await;
    }
    for task in tasks {
        {
            let mut session = shared_session.write().await;
            if let Some(t) = session.tasks.iter_mut().find(|t| t.id == task.id) {
                t.status = TaskStatus::InProgress;
                t.updated_at = Utc::now();
            }
            let event = Event::info(
                format!("{} started task #{}", worker_id, task.id),
                Some(worker_id.to_string()),
            );
            session.events.push(event.clone());
            let _ = write_event_log(base_path, &event).await;
            let _ = write_session_state(base_path, &session).await;
        }
        let prompt = format!(
            "You are a worker agent. Complete this task:\n\nTask ID: {}\nDescription: {}",
            task.id, task.description
        );
        let agent_info =
            AgentInfo::new(worker_id.to_string(), AgentRole::Worker, model.to_string());
        let agent_state = create_shared_agent(agent_info);
        run_agent(cli_agent, agent_state.clone(), &prompt, model, &worktree).await;
        let final_state = agent_state.lock().await;
        {
            let mut session = shared_session.write().await;
            if let Some(t) = session.tasks.iter_mut().find(|t| t.id == task.id) {
                if final_state.status == AgentStatus::Done {
                    t.status = TaskStatus::Testing;
                } else {
                    t.status = TaskStatus::Failed;
                }
                t.updated_at = Utc::now();
            }
            let status_msg = if final_state.status == AgentStatus::Done {
                "completed"
            } else {
                "failed"
            };
            let event = Event::info(
                format!("{} {} task #{}", worker_id, status_msg, task.id),
                Some(worker_id.to_string()),
            );
            session.events.push(event.clone());
            let _ = write_event_log(base_path, &event).await;
            let _ = write_tasks_json(base_path, &session.tasks).await;
            let _ = write_session_state(base_path, &session).await;
        }
    }
    {
        let mut session = shared_session.write().await;
        if let Some(worker) = session.workers.iter_mut().find(|w| w.id == worker_id) {
            worker.status = AgentStatus::Done;
            worker.finished_at = Some(Utc::now());
        }
        let event = Event::info(
            format!("{} finished all tasks", worker_id),
            Some(worker_id.to_string()),
        );
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
        let _ = write_session_state(base_path, &session).await;
    }
}

async fn run_testers(
    shared_session: SharedSession,
    cli_agent: &str,
    model: &str,
    base_path: &PathBuf,
) {
    let tester_tasks: Vec<(String, Vec<Task>)> = {
        let session = shared_session.read().await;
        session
            .testers
            .iter()
            .map(|t| {
                let tasks: Vec<Task> = session
                    .tasks
                    .iter()
                    .filter(|task| {
                        task.assigned_tester.as_ref() == Some(&t.id)
                            && task.status == TaskStatus::Testing
                    })
                    .cloned()
                    .collect();
                (t.id.clone(), tasks)
            })
            .collect()
    };
    let mut handles = Vec::new();
    for (tester_id, tasks) in tester_tasks {
        let shared = shared_session.clone();
        let cli = cli_agent.to_string();
        let m = model.to_string();
        let bp = base_path.clone();
        let handle = tokio::spawn(async move {
            run_single_tester(shared, &tester_id, &tasks, &cli, &m, &bp).await;
        });
        handles.push(handle);
    }
    for handle in handles {
        let _ = handle.await;
    }
}

async fn run_single_tester(
    shared_session: SharedSession,
    tester_id: &str,
    tasks: &[Task],
    cli_agent: &str,
    model: &str,
    base_path: &PathBuf,
) {
    let worktree = match create_agent_dir(base_path, tester_id).await {
        Ok(p) => p,
        Err(_) => return,
    };
    {
        let mut session = shared_session.write().await;
        if let Some(tester) = session.testers.iter_mut().find(|t| t.id == tester_id) {
            tester.status = AgentStatus::Running;
            tester.started_at = Some(Utc::now());
            tester.worktree = Some(worktree.clone());
        }
        let event = Event::info(format!("{} started", tester_id), Some(tester_id.to_string()));
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
        let _ = write_session_state(base_path, &session).await;
    }
    for task in tasks {
        {
            let mut session = shared_session.write().await;
            let event = Event::info(
                format!("{} testing task #{}", tester_id, task.id),
                Some(tester_id.to_string()),
            );
            session.events.push(event.clone());
            let _ = write_event_log(base_path, &event).await;
        }
        let prompt = format!(
            "You are a tester agent. Validate and test the work done for this task:\n\nTask ID: {}\nDescription: {}",
            task.id, task.description
        );
        let agent_info =
            AgentInfo::new(tester_id.to_string(), AgentRole::Tester, model.to_string());
        let agent_state = create_shared_agent(agent_info);
        run_agent(cli_agent, agent_state.clone(), &prompt, model, &worktree).await;
        let final_state = agent_state.lock().await;
        {
            let mut session = shared_session.write().await;
            if let Some(t) = session.tasks.iter_mut().find(|t| t.id == task.id) {
                if final_state.status == AgentStatus::Done {
                    t.status = TaskStatus::Done;
                } else {
                    t.status = TaskStatus::Failed;
                }
                t.updated_at = Utc::now();
            }
            let status_msg = if final_state.status == AgentStatus::Done {
                "approved"
            } else {
                "rejected"
            };
            let event = Event::info(
                format!("{} {} task #{}", tester_id, status_msg, task.id),
                Some(tester_id.to_string()),
            );
            session.events.push(event.clone());
            let _ = write_event_log(base_path, &event).await;
            let _ = write_tasks_json(base_path, &session.tasks).await;
            let _ = write_session_state(base_path, &session).await;
        }
    }
    {
        let mut session = shared_session.write().await;
        if let Some(tester) = session.testers.iter_mut().find(|t| t.id == tester_id) {
            tester.status = AgentStatus::Done;
            tester.finished_at = Some(Utc::now());
        }
        let event = Event::info(
            format!("{} finished all tests", tester_id),
            Some(tester_id.to_string()),
        );
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
        let _ = write_session_state(base_path, &session).await;
    }
}

async fn parse_tasks_from_output(base_path: &PathBuf, count: usize, user_prompt: &str) -> Vec<Task> {
    let log_path = base_path.join("task-splitter").join("logs.txt");
    if let Ok(logs) = tokio::fs::read_to_string(&log_path).await {
        if let Some(start) = logs.find('[') {
            if let Some(end) = logs.rfind(']') {
                let json_str = &logs[start..=end];
                if let Ok(parsed) = serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
                    let tasks: Vec<Task> = parsed
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let id = v.get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&format!("{}", i + 1))
                                .to_string();
                            let desc = v.get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Implement feature")
                                .to_string();
                            Task::new(id, desc)
                        })
                        .collect();
                    if !tasks.is_empty() {
                        return tasks;
                    }
                }
            }
        }
    }
    generate_fallback_tasks(count, user_prompt)
}

fn generate_fallback_tasks(count: usize, user_prompt: &str) -> Vec<Task> {
    let prompt_words: Vec<&str> = user_prompt.split_whitespace().take(10).collect();
    let short_prompt = if prompt_words.len() > 5 {
        format!("{}...", prompt_words[..5].join(" "))
    } else {
        prompt_words.join(" ")
    };
    (1..=count)
        .map(|i| {
            let desc = match i {
                1 => format!("Setup project structure for: {}", short_prompt),
                2 => format!("Implement core logic for: {}", short_prompt),
                3 => format!("Add UI components for: {}", short_prompt),
                4 => format!("Implement data handling for: {}", short_prompt),
                5 => format!("Add error handling for: {}", short_prompt),
                _ => format!("Additional feature {} for: {}", i, short_prompt),
            };
            Task::new(format!("{}", i), desc)
        })
        .collect()
}
