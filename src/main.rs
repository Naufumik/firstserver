use std::sync::Arc;
use axum::{
    routing::{get},
    Router,
};

mod state;
mod error;
mod models;
mod handlers;

pub use state::AppState;
pub use error::AppError;

use handlers::tasks::{get_tasks, create_task, search_tasks, get_task, edit_task, delete_task};

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/todos", get(get_tasks).post(create_task))
        .route("/todos/search", get(search_tasks))
        .route("/todos/{id}", get(get_task).put(edit_task).delete(delete_task))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Сервер запущен на http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}