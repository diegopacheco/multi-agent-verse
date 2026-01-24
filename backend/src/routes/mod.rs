use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::{
    AgentInfo, AgentRole, AgentStatus, CreateSessionRequest, CreateSessionResponse,
    EventsResponse, LogsResponse, RunRequest, RunResponse, Session, StatusResponse,
    TasksResponse,
};
use crate::orchestrator::{create_shared_session, run_orchestration, SharedSession};
use crate::solutions::read_agent_logs;

pub type SessionStore = Arc<RwLock<HashMap<String, SharedSession>>>;

pub fn create_session_store() -> SessionStore {
    Arc::new(RwLock::new(HashMap::new()))
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
