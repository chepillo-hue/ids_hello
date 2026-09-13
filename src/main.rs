mod dtos;
mod handlers;
mod services;

//use axum::{
//    routing::{get, post},
//    Router,
//};
use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = "postgres://postgres:mi_clave_secreta@localhost:5432/demo";

    println!("Conectando a la base de datos PostgreSQL en Docker...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    let app = Router::new()
        .route("/api/vuelos", get(handlers::aeropuerto::obtener_resumen_vuelos))
        .route(
            "/api/aeropuertos",
            get(handlers::aeropuerto::listar).post(handlers::aeropuerto::crear),
        )
        .route(
            "/api/aeropuertos/:codigo",
            get(handlers::aeropuerto::obtener_por_codigo)
                .put(handlers::aeropuerto::actualizar)
                .delete(handlers::aeropuerto::eliminar),
        )
        .fallback_service(ServeDir::new("public"))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Servidor Web iniciado en http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}