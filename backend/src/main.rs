mod agents;
mod models;
mod orchestrator;
mod routes;
mod solutions;

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use routes::{
    create_session, create_session_store, get_events, get_logs, get_status, get_tasks, run_session,
};

async fn health() -> impl Responder {
    HttpResponse::Ok().body("Multi-Agent Verse Backend - OK")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let store = create_session_store();
    println!("Starting server on http://localhost:8080");
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(store.clone()))
            .route("/", web::get().to(health))
            .route("/api/session", web::post().to(create_session))
            .route("/api/run/{session_id}", web::post().to(run_session))
            .route("/api/status/{session_id}", web::get().to(get_status))
            .route("/api/logs/{session_id}/{agent_id}", web::get().to(get_logs))
            .route("/api/tasks/{session_id}", web::get().to(get_tasks))
            .route("/api/events/{session_id}", web::get().to(get_events))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
