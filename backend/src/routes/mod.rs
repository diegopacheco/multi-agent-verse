use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::{
    AgentInfo, AgentRole, AgentStatus, CreateSessionRequest, CreateSessionResponse,
    EventsResponse, LogsResponse, PreviewResponse, ProjectsResponse, RunRequest, RunResponse,
    Session, StatusResponse, TasksResponse,
};
use crate::orchestrator::{create_shared_session, run_orchestration, SharedSession};
use crate::solutions::{list_projects, read_agent_logs};

pub type SessionStore = Arc<RwLock<HashMap<String, SharedSession>>>;
pub type PreviewProcess = Arc<RwLock<Option<(u32, String)>>>;

pub fn create_session_store() -> SessionStore {
    Arc::new(RwLock::new(HashMap::new()))
}

pub fn create_preview_state() -> PreviewProcess {
    Arc::new(RwLock::new(None))
}

fn solutions_code_dir(project_name: &str) -> Result<std::path::PathBuf, String> {
    std::env::current_dir()
        .map_err(|e| e.to_string())
        .and_then(|d| {
            d.parent()
                .ok_or("Cannot get parent directory".to_string())
                .map(|p| p.join("solutions").join(project_name).join("code"))
        })
}

async fn run_stop_sh(project_name: &str) {
    if let Ok(code_dir) = solutions_code_dir(project_name) {
        let stop_sh = code_dir.join("stop.sh");
        if stop_sh.exists() {
            let _ = tokio::process::Command::new("bash")
                .arg("stop.sh")
                .current_dir(&code_dir)
                .output()
                .await;
        }
    }
}

async fn kill_port_5678() {
    let _ = tokio::process::Command::new("bash")
        .args(["-c", "lsof -ti:5678 | xargs kill -9 2>/dev/null"])
        .output()
        .await;
}

pub async fn create_session(
    req: web::Json<CreateSessionRequest>,
    store: web::Data<SessionStore>,
) -> impl Responder {
    let session_id = Uuid::new_v4().to_string();
    let task_splitter = AgentInfo::new(
        "task-splitter".to_string(),
        AgentRole::TaskSplitter,
        req.model.clone(),
    );
    let coordinator = AgentInfo::new(
        "coordinator".to_string(),
        AgentRole::Coordinator,
        req.model.clone(),
    );
    let workers: Vec<AgentInfo> = (1..=req.worker_count)
        .map(|i| AgentInfo::new(format!("worker-{}", i), AgentRole::Worker, req.model.clone()))
        .collect();
    let testers: Vec<AgentInfo> = (1..=req.tester_count)
        .map(|i| AgentInfo::new(format!("tester-{}", i), AgentRole::Tester, req.model.clone()))
        .collect();
    let session = Session {
        id: session_id.clone(),
        project_name: String::new(),
        prompt: String::new(),
        model: req.model.clone(),
        cli_agent: req.cli_agent.clone(),
        task_splitter,
        coordinator,
        workers,
        testers,
        tasks: Vec::new(),
        events: Vec::new(),
        created_at: Utc::now(),
    };
    let shared_session = create_shared_session(session);
    {
        let mut sessions = store.write().await;
        sessions.insert(session_id.clone(), shared_session);
    }
    HttpResponse::Ok().json(CreateSessionResponse { session_id })
}

pub async fn run_session(
    path: web::Path<String>,
    req: web::Json<RunRequest>,
    store: web::Data<SessionStore>,
) -> impl Responder {
    let session_id = path.into_inner();
    let shared_session = {
        let sessions = store.read().await;
        match sessions.get(&session_id) {
            Some(s) => s.clone(),
            None => return HttpResponse::NotFound().body("Session not found"),
        }
    };
    {
        let mut session = shared_session.write().await;
        session.project_name = req.project_name.clone();
        session.prompt = req.prompt.clone();
    }
    let ss = shared_session.clone();
    tokio::spawn(async move {
        run_orchestration(ss).await;
    });
    HttpResponse::Ok().json(RunResponse { ok: true })
}

pub async fn get_status(
    path: web::Path<String>,
    store: web::Data<SessionStore>,
) -> impl Responder {
    let session_id = path.into_inner();
    let sessions = store.read().await;
    let shared_session = match sessions.get(&session_id) {
        Some(s) => s,
        None => return HttpResponse::NotFound().body("Session not found"),
    };
    let session = shared_session.read().await;
    HttpResponse::Ok().json(StatusResponse {
        task_splitter: session.task_splitter.clone(),
        coordinator: session.coordinator.clone(),
        workers: session.workers.clone(),
        testers: session.testers.clone(),
        progress: session.progress(),
        elapsed_time: session.elapsed_secs(),
    })
}

pub async fn get_logs(
    path: web::Path<(String, String)>,
    store: web::Data<SessionStore>,
) -> impl Responder {
    let (session_id, agent_id) = path.into_inner();
    let sessions = store.read().await;
    let shared_session = match sessions.get(&session_id) {
        Some(s) => s,
        None => return HttpResponse::NotFound().body("Session not found"),
    };
    let session = shared_session.read().await;
    let agent = find_agent(&session, &agent_id);
    let agent = match agent {
        Some(a) => a,
        None => return HttpResponse::NotFound().body("Agent not found"),
    };
    let base_path = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("solutions")
        .join(&session.project_name);
    let logs = read_agent_logs(&base_path, &agent_id).await.unwrap_or_default();
    HttpResponse::Ok().json(LogsResponse {
        logs,
        status: agent.status,
        error: if agent.status == AgentStatus::Error {
            Some("Agent execution failed".to_string())
        } else {
            None
        },
        duration: agent.duration_secs(),
    })
}

pub async fn get_tasks(
    path: web::Path<String>,
    store: web::Data<SessionStore>,
) -> impl Responder {
    let session_id = path.into_inner();
    let sessions = store.read().await;
    let shared_session = match sessions.get(&session_id) {
        Some(s) => s,
        None => return HttpResponse::NotFound().body("Session not found"),
    };
    let session = shared_session.read().await;
    HttpResponse::Ok().json(TasksResponse {
        tasks: session.tasks.clone(),
    })
}

pub async fn get_events(
    path: web::Path<String>,
    store: web::Data<SessionStore>,
) -> impl Responder {
    let session_id = path.into_inner();
    let sessions = store.read().await;
    let shared_session = match sessions.get(&session_id) {
        Some(s) => s,
        None => return HttpResponse::NotFound().body("Session not found"),
    };
    let session = shared_session.read().await;
    HttpResponse::Ok().json(EventsResponse {
        events: session.events.clone(),
    })
}

pub async fn get_projects() -> impl Responder {
    match list_projects().await {
        Ok(projects) => HttpResponse::Ok().json(ProjectsResponse { projects }),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

pub async fn preview_start(
    path: web::Path<String>,
    preview: web::Data<PreviewProcess>,
) -> impl Responder {
    let project_name = path.into_inner();
    {
        let existing = preview.read().await;
        if let Some((pid, ref old_project)) = *existing {
            run_stop_sh(old_project).await;
            let _ = tokio::process::Command::new("kill")
                .arg(pid.to_string())
                .output()
                .await;
        }
    }
    kill_port_5678().await;
    let code_dir = match solutions_code_dir(&project_name) {
        Ok(d) => d,
        Err(e) => return HttpResponse::InternalServerError().body(e),
    };
    let run_sh = code_dir.join("run.sh");
    if !run_sh.exists() {
        return HttpResponse::BadRequest().body("run.sh not found in project code directory");
    }
    match tokio::process::Command::new("bash")
        .arg("run.sh")
        .current_dir(&code_dir)
        .spawn()
    {
        Ok(child) => {
            let pid = child.id().unwrap_or(0);
            {
                let mut state = preview.write().await;
                *state = Some((pid, project_name));
            }
            HttpResponse::Ok().json(PreviewResponse {
                ok: true,
                url: Some("http://localhost:5678/index.html".to_string()),
            })
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn preview_stop(preview: web::Data<PreviewProcess>) -> impl Responder {
    let mut state = preview.write().await;
    if let Some((pid, ref old_project)) = *state {
        run_stop_sh(old_project).await;
        let _ = tokio::process::Command::new("kill")
            .arg(pid.to_string())
            .output()
            .await;
        *state = None;
    }
    kill_port_5678().await;
    HttpResponse::Ok().json(PreviewResponse {
        ok: true,
        url: None,
    })
}

fn find_agent<'a>(session: &'a Session, agent_id: &str) -> Option<&'a AgentInfo> {
    if session.task_splitter.id == agent_id {
        return Some(&session.task_splitter);
    }
    if session.coordinator.id == agent_id {
        return Some(&session.coordinator);
    }
    if let Some(worker) = session.workers.iter().find(|w| w.id == agent_id) {
        return Some(worker);
    }
    if let Some(tester) = session.testers.iter().find(|t| t.id == agent_id) {
        return Some(tester);
    }
    None
}
