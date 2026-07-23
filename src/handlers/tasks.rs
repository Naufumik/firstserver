use std::sync::Arc;
use axum::extract::{Query, Path, State};
use axum::Json;
use chrono::Utc;
use sqlx::query_as;

use crate::{AppState, AppError};
use crate::models::task::{
    Task, CreateTaskRequest, SearchParams, UpdateTaskRequest, TaskStatus
};

pub async fn get_tasks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Task>>, AppError> {
    let tasks = query_as::<_, Task>("SELECT * FROM tasks ORDER BY id")
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::InternalError(format!("DB error: {}", e)))?;

    Ok(Json(tasks))
}

pub async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<Task>, AppError> {
    let now = Utc::now();
    
    let new_task = query_as::<_, Task>(
        r#"
        INSERT INTO tasks (title, description, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, title, description, status, created_at, updated_at
        "#
    )
    .bind(payload.title)
    .bind(payload.description)
    .bind(TaskStatus::Pending)
    .bind(now)
    .bind(None::<chrono::DateTime<Utc>>)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::InternalError(format!("DB error: {}", e)))?;

    Ok(Json(new_task))
}

pub async fn search_tasks(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<Task>>, AppError> {
    let tasks = query_as::<_, Task>(
        r#"
        SELECT * FROM tasks
        WHERE ($1::TEXT IS NULL OR title ILIKE '%' || $1::TEXT || '%')
          AND ($2::TEXT IS NULL OR status = $2::TEXT)
        ORDER BY id
        "#
    )
    .bind(params.title.as_deref())
    .bind(params.status.as_ref().map(|s| match s {
        TaskStatus::Pending => "pending",
        TaskStatus::InProgress => "in_progress",
        TaskStatus::Done => "done",
    }))
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::InternalError(format!("DB error: {}", e)))?;

    if tasks.is_empty() {
        Err(AppError::NotFound("Задачи по заданным критериям не найдены".to_string()))
    } else {
        Ok(Json(tasks))
    }
}

pub async fn get_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Task>, AppError> {
    let task = query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::InternalError(format!("DB error: {}", e)))?;

    match task {
        Some(t) => Ok(Json(t)),
        None => Err(AppError::NotFound(format!("Задачи с ID:{} не существует", id))),
    }
}

pub async fn edit_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, AppError> {
    let now = Utc::now();
    
    let updated_task = query_as::<_, Task>(
        r#"
        UPDATE tasks
        SET title = $1, description = $2, status = $3, updated_at = $4
        WHERE id = $5
        RETURNING id, title, description, status, created_at, updated_at
        "#
    )
    .bind(payload.title)
    .bind(payload.description)
    .bind(payload.status)
    .bind(now)
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::InternalError(format!("DB error: {}", e)))?;

    match updated_task {
        Some(task) => Ok(Json(task)),
        None => Err(AppError::NotFound(format!("Задачи с ID:{} не существует", id))),
    }
}

pub async fn delete_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::InternalError(format!("DB error: {}", e)))?;

    if result.rows_affected() == 0 {
        Err(AppError::NotFound(format!("Задачи с ID:{} не существует", id)))
    } else {
        Ok(Json(serde_json::json!({
            "message": format!("Задача с ID:{} успешно удалена", id)
        })))
    }
}