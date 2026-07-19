use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use axum::response::IntoResponse;
use axum::routing::delete;
use axum::{
    extract::{Query, Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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

#[derive(Deserialize)]
struct SearchParams {
    title: Option<String>,
    status: Option<TaskStatus>,
}

#[derive(Deserialize)]
struct UpdateTaskRequest {
    title: String,
    description: String,
    status: TaskStatus,
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
        .route("/todos/search", get(search_tasks))
        .route("/todos/{id}", get(get_task))
        .route("/todos/{id}", put(edit_task))
        .route("/todos/{id}", delete(delete_task))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_tasks(State(state): State<Arc<AppState>>) -> Json<Vec<Task>> {
    let tasks = state.tasks.lock().unwrap();
    Json(tasks.clone())
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


async fn search_tasks(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let tasks = state.tasks.lock().unwrap();

    let found: Vec<Task> = tasks
        .iter()
        .filter(|t| {
            let title_match = params.title.as_ref()
                .map(|search| t.title.to_lowercase().contains(&search.to_lowercase()))
                .unwrap_or(true);
            
            let status_match = params.status.as_ref()
                .map(|s| &t.status == s)
                .unwrap_or(true);
            
            title_match && status_match
        })
        .cloned()
        .collect();

    if found.is_empty() {
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Not found",
                "message": "Задачи по заданным критериям не найдены"
            }))
        ).into_response()
    } else {
        (StatusCode::OK, Json(found)).into_response()
    }
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


async fn edit_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateTaskRequest>,
) -> impl IntoResponse{
    let mut tasks = state.tasks.lock().unwrap();

    match tasks.iter_mut().find(|t| t.id == id) {
        Some(task) => {
            task.title = payload.title;
            task.description = payload.description;
            task.status = payload.status;
            task.updated_at = Some(
                SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
            );
            (StatusCode::OK, Json(task.clone())).into_response()
        },
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


async fn delete_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse{
    let mut tasks = state.tasks.lock().unwrap();

    match tasks.iter().position(|t| t.id == id) {
        Some(index) => {
            let task = tasks.remove(index);
            (StatusCode::OK, Json(json!({
                    "message": format!("Задача \"{}\" успешно удалена", task.title)
                }))
            ).into_response()},
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