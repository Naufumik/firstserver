use std::sync::Arc;
use axum::{
    routing::{get},
    Router,
};
use dotenvy::dotenv;

mod state;
mod error;
mod models;
mod handlers;

pub use state::AppState;
pub use error::AppError;

use handlers::tasks::{get_tasks, create_task, search_tasks, get_task, edit_task, delete_task};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let app_state = AppState::new(&database_url).await?;
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id SERIAL PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ
        )
        "#
    ).execute(&app_state.db).await?;

    let shared_state = Arc::new(app_state);

    let app = Router::new()
        .route("/todos", get(get_tasks).post(create_task))
        .route("/todos/search", get(search_tasks))
        .route("/todos/{id}", get(get_task).put(edit_task).delete(delete_task))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Сервер запущен на http://0.0.0.0:3000");
    axum::serve(listener, app).await?;
    
    Ok(())
}