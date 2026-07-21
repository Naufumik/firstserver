use std::sync::Arc;
use std::sync::atomic::Ordering;
use axum::extract::{Query, Path, State};
use axum::Json;
use chrono::Utc;

use crate::{AppState, AppError};
use crate::models::task::{
    Task, CreateTaskRequest, SearchParams, UpdateTaskRequest
};

pub async fn get_tasks(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<Task>> {
    let tasks = state.tasks.read();
    Json(tasks.clone())
}

pub async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<Task>, AppError> {
    let mut tasks = state.tasks.write();
    let new_id = state.next_id.fetch_add(1, Ordering::SeqCst);

    let new_task = Task {
        id: new_id,
        title: payload.title,
        description: payload.description,
        status: crate::models::task::TaskStatus::Pending,
        created_at: Utc::now(),
        updated_at: None,
    };

    tasks.push(new_task.clone());
    Ok(Json(new_task))
}

pub async fn search_tasks(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<Task>>, AppError> {
    let tasks = state.tasks.read();

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
        Err(AppError::NotFound("Задачи по заданным критериям не найдены".to_string()))
    } else {
        Ok(Json(found))
    }
}

pub async fn get_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Task>, AppError> {
    let tasks = state.tasks.read();

    tasks.iter().find(|t| t.id == id)
        .map(|task| Json(task.clone()))
        .ok_or_else(|| AppError::NotFound(format!("Задачи с ID:{} не существует", id)))
}

pub async fn edit_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, AppError> {
    let mut tasks = state.tasks.write();

    match tasks.iter_mut().find(|t| t.id == id) {
        Some(task) => {
            task.title = payload.title;
            task.description = payload.description;
            task.status = payload.status;
            task.updated_at = Some(Utc::now());
            Ok(Json(task.clone()))
        }
        None => Err(AppError::NotFound(format!("Задачи с ID:{} не существует", id)))
    }
}

pub async fn delete_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut tasks = state.tasks.write();

    match tasks.iter().position(|t| t.id == id) {
        Some(index) => {
            let task = tasks.remove(index);
            Ok(Json(serde_json::json!({
                "message": format!("Задача \"{}\" успешно удалена", task.title)
            })))
        }
        None => Err(AppError::NotFound(format!("Задачи с ID:{} не существует", id)))
    }
}