use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use axum::response::IntoResponse;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize)]
enum TaskStatus {
    Pending,
    InProgress,
    Done,
}

#[derive(Clone, Serialize, Deserialize)]
struct Task {
    id: i32,
    title: String,
    description: String,
    status: TaskStatus,
    created_at: u64,
    updated_at: Option<u64>,
}

#[derive(Deserialize)]
struct CreateTaskRequest {
    title: String,
    description: String,
}

struct AppState {
    tasks: Mutex<Vec<Task>>,
    next_id: AtomicI32,
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState {
        tasks: Mutex::new(Vec::new()),
        next_id: AtomicI32::new(1),
    });

    let app = Router::new()
        .route("/todos", get(get_tasks))
        .route("/todos", post(create_task))
        .route("/todos/{id}", get(get_task))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_tasks(State(state): State<Arc<AppState>>) -> Json<Vec<Task>> {
    let tasks = state.tasks.lock().unwrap();
    Json(tasks.clone())
}

async fn get_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse{
    let tasks = state.tasks.lock().unwrap();

    match tasks.iter().find(|t| t.id == id) {
        Some(task) => {
            (StatusCode::OK, Json(task.clone()) ).into_response()},
        None => {
            (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "Task not found",
                    "message": format!("Задачи с ID:{} не существует",id)
                }))
            ).into_response()
        }
    }
}

async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTaskRequest>,
) -> Json<Task> {
    let mut tasks = state.tasks.lock().unwrap();
    let new_id = state.next_id.fetch_add(1, Ordering::SeqCst);

    let new_task = Task {
        id: new_id,
        title: payload.title,
        description: payload.description,
        status: TaskStatus::Pending,
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        updated_at: None,
    };

    tasks.push(new_task.clone());
    Json(new_task)
}