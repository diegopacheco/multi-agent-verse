use crate::agents::{create_shared_agent, run_agent};
use crate::models::{AgentInfo, AgentRole, AgentStatus, Event, Session, SplitTasksResult, Task, TaskDefinition, TaskStatus, TaskType};
use crate::solutions::{
    create_agent_dir, create_code_dir, create_project_dir, write_event_log, write_session_state,
    write_summary, write_tasks_json,
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
    run_coordinator(shared_session.clone(), &cli_agent, &model, &base_path).await;
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
    let prompt = format!(
        r#"You are a task splitter agent. Analyze this project request and break it into independent tasks.

RULES:
1. Minimum 3 tasks required
2. Tasks that can run in parallel go to "parallel_tasks"
3. Tasks with dependencies go to "sequential_tasks" with order number
4. ALWAYS include: run.sh creation (sequential, second-to-last order) and stop.sh creation (sequential, last order)
5. Each task must be self-contained and clearly described

IMPORTANT: Output ONLY valid JSON - no markdown, no explanation, just the JSON object.

Output format:
{{
  "parallel_tasks": [
    {{"id": "1", "description": "Brief but meaningful task description"}},
    {{"id": "2", "description": "Another task that can run in parallel"}}
  ],
  "sequential_tasks": [
    {{"id": "3", "description": "Create run.sh script to start the application on port 5678", "order": 1}},
    {{"id": "4", "description": "Create stop.sh script to stop the application", "order": 2}}
  ]
}}

Project request:
{}"#,
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
        let worker_count = session.workers.len();
        let tasks = if final_state.status == AgentStatus::Done {
            parse_tasks_from_output(base_path, worker_count, user_prompt).await
        } else {
            generate_fallback_tasks(worker_count, user_prompt)
        };
        session.tasks = tasks.clone();
        let event = Event::info(
            format!("{} tasks created", session.tasks.len()),
            Some(agent_id),
        );
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
        let _ = write_tasks_json(base_path, &session.tasks).await;
        let _ = write_session_state(base_path, &session).await;
    }
}

async fn run_coordinator(
    shared_session: SharedSession,
    cli_agent: &str,
    model: &str,
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
    let code_path = match create_code_dir(base_path).await {
        Ok(p) => p,
        Err(e) => {
            let mut session = shared_session.write().await;
            let event = Event::error(format!("Failed to create code dir: {}", e), None);
            session.events.push(event);
            return;
        }
    };
    {
        let mut session = shared_session.write().await;
        session.coordinator.status = AgentStatus::Running;
        session.coordinator.started_at = Some(Utc::now());
        session.coordinator.worktree = Some(worktree.clone());
        let event = Event::info("coordinator started - orchestration loop beginning".to_string(), Some(agent_id.clone()));
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
        let _ = write_session_state(base_path, &session).await;
    }
    let mut log_lines = Vec::new();
    log_lines.push(format!("[{}] Coordinator orchestration started", Utc::now().format("%H:%M:%S")));
    let (parallel_tasks, sequential_tasks) = {
        let session = shared_session.read().await;
        let parallel: Vec<Task> = session.tasks.iter()
            .filter(|t| t.task_type == TaskType::Parallel)
            .cloned()
            .collect();
        let mut sequential: Vec<Task> = session.tasks.iter()
            .filter(|t| t.task_type == TaskType::Sequential)
            .cloned()
            .collect();
        sequential.sort_by_key(|t| t.order.unwrap_or(0));
        (parallel, sequential)
    };
    log_lines.push(format!("[{}] Found {} parallel tasks and {} sequential tasks",
        Utc::now().format("%H:%M:%S"), parallel_tasks.len(), sequential_tasks.len()));
    {
        let mut session = shared_session.write().await;
        let workers: Vec<String> = session.workers.iter().map(|w| w.id.clone()).collect();
        let testers: Vec<String> = session.testers.iter().map(|t| t.id.clone()).collect();
        let mut worker_idx = 0;
        for task in session.tasks.iter_mut() {
            if task.task_type == TaskType::Parallel && !workers.is_empty() {
                task.assigned_worker = Some(workers[worker_idx % workers.len()].clone());
                log_lines.push(format!("[{}] Parallel task #{} -> {}",
                    Utc::now().format("%H:%M:%S"), task.id, workers[worker_idx % workers.len()]));
                worker_idx += 1;
            }
            if !testers.is_empty() {
                task.assigned_tester = Some(testers[task.id.parse::<usize>().unwrap_or(0) % testers.len()].clone());
            }
        }
        let event = Event::info(
            format!("Assigned {} parallel tasks to {} workers", parallel_tasks.len(), workers.len()),
            Some(agent_id.clone()),
        );
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
        let _ = write_tasks_json(base_path, &session.tasks).await;
        let _ = write_session_state(base_path, &session).await;
    }
    log_lines.push(format!("[{}] PHASE 1: Executing parallel tasks", Utc::now().format("%H:%M:%S")));
    {
        let mut session = shared_session.write().await;
        let event = Event::info("PHASE 1: Starting parallel task execution".to_string(), Some(agent_id.clone()));
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
    }
    let updated_parallel_tasks: Vec<Task> = {
        let session = shared_session.read().await;
        session.tasks.iter()
            .filter(|t| t.task_type == TaskType::Parallel)
            .cloned()
            .collect()
    };
    run_parallel_tasks(shared_session.clone(), cli_agent, model, base_path, &code_path, &updated_parallel_tasks).await;
    log_lines.push(format!("[{}] PHASE 1 complete: All parallel tasks finished", Utc::now().format("%H:%M:%S")));
    {
        let mut session = shared_session.write().await;
        let completed = session.tasks.iter()
            .filter(|t| t.task_type == TaskType::Parallel && (t.status == TaskStatus::Testing || t.status == TaskStatus::Done))
            .count();
        let event = Event::info(
            format!("PHASE 1 complete: {}/{} parallel tasks succeeded", completed, parallel_tasks.len()),
            Some(agent_id.clone()),
        );
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
    }
    log_lines.push(format!("[{}] PHASE 2: Executing sequential tasks", Utc::now().format("%H:%M:%S")));
    {
        let mut session = shared_session.write().await;
        let event = Event::info("PHASE 2: Starting sequential task execution".to_string(), Some(agent_id.clone()));
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
    }
    let updated_sequential_tasks: Vec<Task> = {
        let session = shared_session.read().await;
        let mut seq: Vec<Task> = session.tasks.iter()
            .filter(|t| t.task_type == TaskType::Sequential)
            .cloned()
            .collect();
        seq.sort_by_key(|t| t.order.unwrap_or(0));
        seq
    };
    run_sequential_tasks(shared_session.clone(), cli_agent, model, base_path, &code_path, &updated_sequential_tasks).await;
    log_lines.push(format!("[{}] PHASE 2 complete: All sequential tasks finished", Utc::now().format("%H:%M:%S")));
    log_lines.push(format!("[{}] PHASE 3: Running testers", Utc::now().format("%H:%M:%S")));
    {
        let mut session = shared_session.write().await;
        let event = Event::info("PHASE 3: Starting tester validation".to_string(), Some(agent_id.clone()));
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
    }
    run_testers_internal(shared_session.clone(), cli_agent, model, base_path, &code_path).await;
    log_lines.push(format!("[{}] PHASE 3 complete: All testing finished", Utc::now().format("%H:%M:%S")));
    {
        let mut session = shared_session.write().await;
        let completed = session.tasks.iter().filter(|t| t.status == TaskStatus::Done).count();
        let failed = session.tasks.iter().filter(|t| t.status == TaskStatus::Failed).count();
        let total = session.tasks.len();
        log_lines.push(format!("[{}] Orchestration complete: {}/{} succeeded, {} failed",
            Utc::now().format("%H:%M:%S"), completed, total, failed));
        session.coordinator.status = AgentStatus::Done;
        session.coordinator.finished_at = Some(Utc::now());
        let event = Event::info(
            format!("Coordinator finished - {}/{} tasks completed, {} failed", completed, total, failed),
            Some(agent_id),
        );
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
        let log_content = log_lines.join("\n");
        let log_path = worktree.join("logs.txt");
        let _ = tokio::fs::write(&log_path, &log_content).await;
        let _ = write_tasks_json(base_path, &session.tasks).await;
        let _ = write_session_state(base_path, &session).await;
    }
}

async fn run_parallel_tasks(
    shared_session: SharedSession,
    cli_agent: &str,
    model: &str,
    base_path: &PathBuf,
    code_path: &PathBuf,
    parallel_tasks: &[Task],
) {
    let worker_tasks: Vec<(String, Vec<Task>)> = {
        let session = shared_session.read().await;
        session
            .workers
            .iter()
            .map(|w| {
                let tasks: Vec<Task> = parallel_tasks
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
        if tasks.is_empty() {
            continue;
        }
        let shared = shared_session.clone();
        let cli = cli_agent.to_string();
        let m = model.to_string();
        let bp = base_path.clone();
        let cp = code_path.clone();
        let handle = tokio::spawn(async move {
            run_single_worker(shared, &worker_id, &tasks, &cli, &m, &bp, &cp).await;
        });
        handles.push(handle);
    }
    for handle in handles {
        let _ = handle.await;
    }
}

async fn run_sequential_tasks(
    shared_session: SharedSession,
    cli_agent: &str,
    model: &str,
    base_path: &PathBuf,
    code_path: &PathBuf,
    sequential_tasks: &[Task],
) {
    let workers: Vec<String> = {
        let session = shared_session.read().await;
        session.workers.iter().map(|w| w.id.clone()).collect()
    };
    if workers.is_empty() || sequential_tasks.is_empty() {
        return;
    }
    let worker_id = workers[0].clone();
    for task in sequential_tasks {
        {
            let mut session = shared_session.write().await;
            if let Some(t) = session.tasks.iter_mut().find(|t| t.id == task.id) {
                t.assigned_worker = Some(worker_id.clone());
            }
            let event = Event::info(
                format!("Sequential task #{} assigned to {}", task.id, worker_id),
                Some("coordinator".to_string()),
            );
            session.events.push(event.clone());
            let _ = write_event_log(base_path, &event).await;
        }
        run_single_worker(shared_session.clone(), &worker_id, &[task.clone()], cli_agent, model, base_path, code_path).await;
    }
}

async fn run_testers_internal(
    shared_session: SharedSession,
    cli_agent: &str,
    model: &str,
    base_path: &PathBuf,
    code_path: &PathBuf,
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
        let cp = code_path.clone();
        let handle = tokio::spawn(async move {
            run_single_tester(shared, &tester_id, &tasks, &cli, &m, &bp, &cp).await;
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
    code_path: &PathBuf,
) {
    let log_dir = match create_agent_dir(base_path, worker_id).await {
        Ok(p) => p,
        Err(_) => return,
    };
    {
        let mut session = shared_session.write().await;
        if let Some(worker) = session.workers.iter_mut().find(|w| w.id == worker_id) {
            worker.status = AgentStatus::Running;
            worker.started_at = Some(Utc::now());
            worker.worktree = Some(code_path.clone());
        }
        let event = Event::info(format!("{} started", worker_id), Some(worker_id.to_string()));
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
        let _ = write_session_state(base_path, &session).await;
    }
    if tasks.is_empty() {
        let log_path = log_dir.join("logs.txt");
        let _ = tokio::fs::write(&log_path, "No tasks assigned to this worker.\n").await;
        let mut session = shared_session.write().await;
        if let Some(worker) = session.workers.iter_mut().find(|w| w.id == worker_id) {
            worker.status = AgentStatus::Done;
            worker.finished_at = Some(Utc::now());
        }
        let event = Event::info(format!("{} finished - no tasks assigned", worker_id), Some(worker_id.to_string()));
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
        let _ = write_session_state(base_path, &session).await;
        return;
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
            r#"You are a worker agent. Complete this task.

Task ID: {}
Description: {}

REQUIREMENTS:
- Write all code to the current directory
- Create clean, working code that fulfills the task description
- Do not create unnecessary files
- If creating run.sh: start the app on port 5678 serving /index.html
- If creating stop.sh: properly terminate the process started by run.sh
- Update overall progress when done"#,
            task.id, task.description
        );
        let agent_info =
            AgentInfo::new_with_log_dir(worker_id.to_string(), AgentRole::Worker, model.to_string(), log_dir.clone());
        let agent_state = create_shared_agent(agent_info);
        run_agent(cli_agent, agent_state.clone(), &prompt, model, code_path).await;
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


async fn run_single_tester(
    shared_session: SharedSession,
    tester_id: &str,
    tasks: &[Task],
    cli_agent: &str,
    model: &str,
    base_path: &PathBuf,
    code_path: &PathBuf,
) {
    let log_dir = match create_agent_dir(base_path, tester_id).await {
        Ok(p) => p,
        Err(_) => return,
    };
    {
        let mut session = shared_session.write().await;
        if let Some(tester) = session.testers.iter_mut().find(|t| t.id == tester_id) {
            tester.status = AgentStatus::Running;
            tester.started_at = Some(Utc::now());
            tester.worktree = Some(code_path.clone());
        }
        let event = Event::info(format!("{} started", tester_id), Some(tester_id.to_string()));
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
        let _ = write_session_state(base_path, &session).await;
    }
    if tasks.is_empty() {
        let log_path = log_dir.join("logs.txt");
        let _ = tokio::fs::write(&log_path, "No tasks assigned to this tester.\n").await;
        let mut session = shared_session.write().await;
        if let Some(tester) = session.testers.iter_mut().find(|t| t.id == tester_id) {
            tester.status = AgentStatus::Done;
            tester.finished_at = Some(Utc::now());
        }
        let event = Event::info(format!("{} finished - no tasks to test", tester_id), Some(tester_id.to_string()));
        session.events.push(event.clone());
        let _ = write_event_log(base_path, &event).await;
        let _ = write_session_state(base_path, &session).await;
        return;
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
            r#"You are a tester agent. Validate and test the work done for this task.

Task ID: {}
Description: {}

TESTING PHASES:

1. MANUAL TESTING:
   - Check if all required files exist
   - Verify run.sh starts the application correctly
   - Verify the application responds on port 5678
   - Verify /index.html loads correctly
   - Test core functionality described in the task

2. AUTOMATED TESTING:
   - Create a tests/ directory if it does not exist
   - Generate automated tests for the task functionality
   - Write tests to tests/test_{}.sh or tests/test_{}.js
   - Execute the tests and verify they pass

3. CLEANUP TESTING:
   - Verify stop.sh properly terminates the application
   - Ensure no zombie processes are left running

Report your findings and fix any issues you find. Mark the task as PASSED only if all tests succeed."#,
            task.id, task.description, task.id, task.id
        );
        let agent_info =
            AgentInfo::new_with_log_dir(tester_id.to_string(), AgentRole::Tester, model.to_string(), log_dir.clone());
        let agent_state = create_shared_agent(agent_info);
        run_agent(cli_agent, agent_state.clone(), &prompt, model, code_path).await;
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
        if let Some(start) = logs.find('{') {
            let json_str = if let Some(end) = logs.rfind('}') {
                logs[start..=end].to_string()
            } else {
                logs[start..].to_string()
            };
            if let Ok(parsed) = serde_json::from_str::<SplitTasksResult>(&json_str) {
                let mut tasks = Vec::new();
                for def in parsed.parallel_tasks {
                    tasks.push(Task::new_parallel(def.id, def.description));
                }
                for def in parsed.sequential_tasks {
                    let order = def.order.unwrap_or(1);
                    tasks.push(Task::new_sequential(def.id, def.description, order));
                }
                if tasks.len() >= 3 {
                    return tasks;
                }
            }
            if let Some(arr_start) = logs.find('[') {
                let arr_json = if let Some(arr_end) = logs.rfind(']') {
                    logs[arr_start..=arr_end].to_string()
                } else {
                    format!("{}]", logs[arr_start..].trim_end())
                };
                if let Ok(parsed) = serde_json::from_str::<Vec<TaskDefinition>>(&arr_json) {
                    let tasks: Vec<Task> = parsed
                        .iter()
                        .map(|def| Task::new_parallel(def.id.clone(), def.description.clone()))
                        .collect();
                    if !tasks.is_empty() {
                        return ensure_minimum_tasks(tasks, user_prompt);
                    }
                }
            }
        }
    }
    generate_fallback_tasks(count, user_prompt)
}

fn ensure_minimum_tasks(mut tasks: Vec<Task>, user_prompt: &str) -> Vec<Task> {
    let has_run_sh = tasks.iter().any(|t| t.description.to_lowercase().contains("run.sh"));
    let has_stop_sh = tasks.iter().any(|t| t.description.to_lowercase().contains("stop.sh"));
    let next_id = tasks.len() + 1;
    if !has_run_sh {
        tasks.push(Task::new_sequential(
            format!("{}", next_id),
            "Create run.sh script to start the application on port 5678 serving /index.html".to_string(),
            100,
        ));
    }
    if !has_stop_sh {
        tasks.push(Task::new_sequential(
            format!("{}", next_id + 1),
            "Create stop.sh script to stop the application started by run.sh".to_string(),
            101,
        ));
    }
    if tasks.len() < 3 {
        let prompt_words: Vec<&str> = user_prompt.split_whitespace().take(10).collect();
        let short_prompt = prompt_words.join(" ");
        tasks.insert(0, Task::new_parallel(
            "0".to_string(),
            format!("Implement main functionality: {}", short_prompt),
        ));
    }
    tasks
}

fn generate_fallback_tasks(count: usize, user_prompt: &str) -> Vec<Task> {
    let prompt_words: Vec<&str> = user_prompt.split_whitespace().take(10).collect();
    let short_prompt = if prompt_words.len() > 5 {
        format!("{}...", prompt_words[..5].join(" "))
    } else {
        prompt_words.join(" ")
    };
    let parallel_count = count.saturating_sub(2).max(1);
    let mut tasks: Vec<Task> = (1..=parallel_count)
        .map(|i| {
            let desc = match i {
                1 => format!("Setup project structure and implement core logic for: {}", short_prompt),
                2 => format!("Add UI components for: {}", short_prompt),
                3 => format!("Implement data handling for: {}", short_prompt),
                4 => format!("Add error handling for: {}", short_prompt),
                _ => format!("Additional feature {} for: {}", i, short_prompt),
            };
            Task::new_parallel(format!("{}", i), desc)
        })
        .collect();
    tasks.push(Task::new_sequential(
        format!("{}", parallel_count + 1),
        "Create run.sh script to start the application on port 5678 serving /index.html".to_string(),
        1,
    ));
    tasks.push(Task::new_sequential(
        format!("{}", parallel_count + 2),
        "Create stop.sh script to stop the application started by run.sh".to_string(),
        2,
    ));
    tasks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_shared_session() {
        let session = Session {
            id: "test-id".to_string(),
            project_name: "test-project".to_string(),
            prompt: "test prompt".to_string(),
            model: "opus-4".to_string(),
            cli_agent: "claude".to_string(),
            task_splitter: AgentInfo::new("ts".to_string(), AgentRole::TaskSplitter, "opus-4".to_string()),
            coordinator: AgentInfo::new("coord".to_string(), AgentRole::Coordinator, "opus-4".to_string()),
            workers: vec![],
            testers: vec![],
            tasks: vec![],
            events: vec![],
            created_at: Utc::now(),
        };
        let shared = create_shared_session(session);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let session = rt.block_on(async { shared.read().await.clone() });
        assert_eq!(session.project_name, "test-project");
    }

    #[test]
    fn test_generate_fallback_tasks_minimum_count() {
        let tasks = generate_fallback_tasks(3, "build a calculator app");
        assert!(tasks.len() >= 3);
    }

    #[test]
    fn test_generate_fallback_tasks_includes_run_sh() {
        let tasks = generate_fallback_tasks(5, "build a web app");
        let has_run_sh = tasks.iter().any(|t| t.description.contains("run.sh"));
        assert!(has_run_sh);
    }

    #[test]
    fn test_generate_fallback_tasks_includes_stop_sh() {
        let tasks = generate_fallback_tasks(5, "build a web app");
        let has_stop_sh = tasks.iter().any(|t| t.description.contains("stop.sh"));
        assert!(has_stop_sh);
    }

    #[test]
    fn test_generate_fallback_tasks_short_prompt() {
        let tasks = generate_fallback_tasks(3, "test");
        assert!(!tasks.is_empty());
    }

    #[test]
    fn test_generate_fallback_tasks_long_prompt() {
        let tasks = generate_fallback_tasks(3, "this is a very long prompt with many words to test truncation");
        let first_task = &tasks[0];
        assert!(first_task.description.contains("..."));
    }
}
