use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::PgPool;
use crate::dtos::perfume::CrearPerfumeDto;
use crate::services::perfume::PerfumeService;

pub async fn listar_perfumes_handler(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match PerfumeService::listar_todos(&pool).await {
        Ok(lista) => Ok(Json(lista)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn crear_perfume_handler(
    State(pool): State<PgPool>,
    Json(dto): Json<CrearPerfumeDto>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match PerfumeService::crear(&pool, dto).await {
        Ok(nuevo) => Ok((StatusCode::CREATED, Json(nuevo))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}