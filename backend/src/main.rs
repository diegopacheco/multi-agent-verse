mod agents;
mod models;
mod orchestrator;
mod routes;
mod solutions;

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use routes::{
    create_preview_state, create_session, create_session_store, get_events, get_logs, get_projects,
    get_status, get_tasks, preview_start, preview_stop, run_session,
};

async fn health() -> impl Responder {
    HttpResponse::Ok().body("Multi-Agent Verse Backend - OK")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let store = create_session_store();
    let preview = create_preview_state();
    println!("Starting server on http://localhost:8080");
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(store.clone()))
            .app_data(web::Data::new(preview.clone()))
            .route("/", web::get().to(health))
            .route("/api/session", web::post().to(create_session))
            .route("/api/run/{session_id}", web::post().to(run_session))
            .route("/api/status/{session_id}", web::get().to(get_status))
            .route("/api/logs/{session_id}/{agent_id}", web::get().to(get_logs))
            .route("/api/tasks/{session_id}", web::get().to(get_tasks))
            .route("/api/events/{session_id}", web::get().to(get_events))
            .route("/api/projects", web::get().to(get_projects))
            .route("/api/preview/start/{project_name}", web::post().to(preview_start))
            .route("/api/preview/stop", web::post().to(preview_stop))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
