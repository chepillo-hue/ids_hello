use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};

// Estructuras de Datos (DTOs)
#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Perfume {
    pub id_perfume: i32,
    pub sku: String,
    pub nombre: String,
    pub genero: String,
    pub precio_costo: f64,
    pub precio_venta: f64,
    pub imagen_url: Option<String>,
}

#[derive(Deserialize)]
pub struct CrearPerfume {
    pub sku: String,
    pub nombre: String,
    pub genero: String,
    pub precio_costo: f64,
    pub precio_venta: f64,
    pub imagen_url: Option<String>,
}

#[tokio::main]
async fn main() {
    // 1. Conexión a Base de Datos (PostgreSQL vía DATABASE_URL)
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/perfumeria_db".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("No se pudo conectar a PostgreSQL");

    // 2. Rutas del API (Sin anteponer /api internamente)
    let api_routes = Router::new()
        .route("/catalogos/perfumes", get(obtener_perfumes).post(crear_perfume))
        .route("/catalogos/perfumes/:id", put(actualizar_perfume).delete(eliminar_perfume));

    // 3. Router Principal y Archivos Estáticos
    let app = Router::new()
        .nest("/api", api_routes) // El prefijo /api se aplica aquí para todo el grupo
        .nest_service("/public", ServeDir::new("public"))
        .route_service("/", ServeFile::new("public/index.html"))
        .with_state(pool);

    // 4. Servidor en Puerto Dinámico (Render / Local)
    let port_str = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let port: u16 = port_str.parse().expect("PORT debe ser un número válido");
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("Servidor corriendo exitosamente en http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// --- HANDLERS DEL CRUD ---

// GET: Obtener todos los perfumes
async fn obtener_perfumes(State(pool): State<PgPool>) -> Result<Json<Vec<Perfume>>, StatusCode> {
    let perfumes = sqlx::query_as::<_, Perfume>("SELECT * FROM perfumes ORDER BY id_perfume DESC")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(perfumes))
}

// POST: Crear nuevo perfume
async fn crear_perfume(
    State(pool): State<PgPool>,
    Json(payload): Json<CrearPerfume>,
) -> Result<(StatusCode, Json<Perfume>), StatusCode> {
    let perfume = sqlx::query_as::<_, Perfume>(
        r#"
        INSERT INTO perfumes (sku, nombre, genero, precio_costo, precio_venta, imagen_url)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(&payload.sku)
    .bind(&payload.nombre)
    .bind(&payload.genero)
    .bind(payload.precio_costo)
    .bind(payload.precio_venta)
    .bind(&payload.imagen_url)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(perfume)))
}

// PUT: Actualizar perfume existente
async fn actualizar_perfume(
    Path(id): Path<i32>,
    State(pool): State<PgPool>,
    Json(payload): Json<CrearPerfume>,
) -> Result<Json<Perfume>, StatusCode> {
    let perfume = sqlx::query_as::<_, Perfume>(
        r#"
        UPDATE perfumes
        SET sku = $1, nombre = $2, genero = $3, precio_costo = $4, precio_venta = $5, imagen_url = $6
        WHERE id_perfume = $7
        RETURNING *
        "#,
    )
    .bind(&payload.sku)
    .bind(&payload.nombre)
    .bind(&payload.genero)
    .bind(payload.precio_costo)
    .bind(payload.precio_venta)
    .bind(&payload.imagen_url)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(perfume))
}

// DELETE: Eliminar perfume por ID
async fn eliminar_perfume(
    Path(id): Path<i32>,
    State(pool): State<PgPool>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM perfumes WHERE id_perfume = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}