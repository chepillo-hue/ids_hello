use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::PgPool;

use crate::dtos::aeropuerto::{ActualizarAeropuertoDto, CrearAeropuertoDto};
use crate::services::aeropuerto::AeropuertoService;

pub async fn obtener_resumen_vuelos(State(pool): State<PgPool>) -> impl IntoResponse {
    match AeropuertoService::listar_resumen_vuelos(&pool).await {
        Ok(datos) => (StatusCode::OK, Json(datos)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn listar(State(pool): State<PgPool>) -> impl IntoResponse {
    match AeropuertoService::listar(&pool).await {
        Ok(datos) => (StatusCode::OK, Json(datos)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn obtener_por_codigo(
    State(pool): State<PgPool>,
    Path(codigo): Path<String>,
) -> impl IntoResponse {
    match AeropuertoService::obtener_por_codigo(&pool, &codigo).await {
        Ok(Some(aeropuerto)) => (StatusCode::OK, Json(aeropuerto)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Aeropuerto no encontrado").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn crear(
    State(pool): State<PgPool>,
    Json(payload): Json<CrearAeropuertoDto>,
) -> impl IntoResponse {
    match AeropuertoService::crear(&pool, payload).await {
        Ok(creado) => (StatusCode::CREATED, Json(creado)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

pub async fn actualizar(
    State(pool): State<PgPool>,
    Path(codigo): Path<String>,
    Json(payload): Json<ActualizarAeropuertoDto>,
) -> impl IntoResponse {
    match AeropuertoService::actualizar(&pool, &codigo, payload).await {
        Ok(Some(actualizado)) => (StatusCode::OK, Json(actualizado)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Aeropuerto no encontrado").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

pub async fn eliminar(
    State(pool): State<PgPool>,
    Path(codigo): Path<String>,
) -> impl IntoResponse {
    match AeropuertoService::eliminar(&pool, &codigo).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "Aeropuerto no encontrado").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}