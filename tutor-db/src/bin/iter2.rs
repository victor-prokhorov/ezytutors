use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use sqlx::PgPool;
use std::{
    env, io,
    sync::{Arc, Mutex},
};

#[path = "../iter2/handlers.rs"]
mod handlers;
#[path = "../iter2/routes.rs"]
mod routes;
use routes::*;
#[path = "../iter2/state.rs"]
mod state;
use state::AppState;
#[path = "../iter2/models.rs"]
mod models;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    // for dev environment to easy loadig .env
    dotenv().ok();
    let databse_url = env::var("DATABASE_URL").expect("set DATABASE_URL in .env");
    let db_pool = PgPool::connect(&databse_url).await.unwrap();
    let shared_data = web::Data::new(AppState {
        health_check_response: String::new(),
        visit_count: Mutex::new(0),
        db: db_pool,
    });
    let app = move || {
        App::new()
            .app_data(Arc::clone(&shared_data))
            .configure(general_routes)
            .configure(course_routes)
    };
    HttpServer::new(app).bind("127.0.0.1:3000")?.run().await
}
