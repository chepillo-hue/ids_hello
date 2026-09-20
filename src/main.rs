use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};

// Estructura para recibir el JSON de la transacción POST
#[derive(Debug, Deserialize)]
struct NuevoPerfumeInput {
    sku: String,
    nombre: String,
    marca: String,
    genero: Option<String>,
    familia: Option<String>,
    concentracion: Option<String>,
    precio_costo: f64,
    precio_venta: f64,
    stock: i32,
}

#[derive(Serialize)]
struct RespuestaTransaccion {
    mensaje: String,
    sku: String,
}

#[tokio::main]
async fn main() {
    // 1. Conexión a Base de Datos
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ids_hello_db".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("No se pudo conectar a PostgreSQL");

    // 2. Rutas Dinámicas de la API (Soporta GET y POST)
    let api_routes = Router::new()
        .route("/catalogos/:tabla", get(obtener_catalogo))
        .route("/catalogos/perfumes", post(crear_perfume));

    // 3. Router Principal y Archivos Estáticos
    let app = Router::new()
        .nest("/api", api_routes)
        .nest_service("/public", ServeDir::new("public"))
        .route_service("/", ServeFile::new("public/index.html"))
        .with_state(pool);

    // 4. Iniciar Servidor en 0.0.0.0 para Render
    let port_str = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let port: u16 = port_str.parse().expect("PORT invalido");
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("Servidor ERP corriendo en http://0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Handler para recibir las transacciones POST e insertar perfumes
async fn crear_perfume(
    State(pool): State<PgPool>,
    Json(payload): Json<NuevoPerfumeInput>,
) -> Result<Json<RespuestaTransaccion>, String> {

    // Inserción directa en la tabla de perfumes
    let query = "
        INSERT INTO perfumes (sku, nombre, genero, precio_costo, precio_venta, stock)
        VALUES ($1, $2, $3, $4, $5, $6)
    ";

    sqlx::query(query)
        .bind(&payload.sku)
        .bind(&payload.nombre)
        .bind(payload.genero.unwrap_or_else(|| "Unisex".to_string()))
        .bind(payload.precio_costo)
        .bind(payload.precio_venta)
        .bind(payload.stock)
        .execute(&pool)
        .await
        .map_err(|e| format!("Error al insertar en la BD: {}", e))?;

    Ok(Json(RespuestaTransaccion {
        mensaje: "Perfume registrado correctamente".to_string(),
        sku: payload.sku,
    }))
}

// Handler dinámico multicatálogo (GET)
async fn obtener_catalogo(
    Path(tabla): Path<String>,
    State(pool): State<PgPool>,
) -> Result<Json<Value>, String> {
    let query = match tabla.as_str() {
        "marcas" => "SELECT id_marca, nombre, pais_origen FROM marcas ORDER BY nombre",
        "familias" => "SELECT id_familia, nombre, descripcion FROM familias_olfativas ORDER BY nombre",
        "concentraciones" => "SELECT id_concentracion, nombre, siglas FROM concentraciones ORDER BY id_concentracion",
        "clientes" => "SELECT id_cliente, nombre, telefono, email, tipo FROM clientes ORDER BY nombre",
        "proveedores" => "SELECT id_proveedor, razon_social, rfc, contacto, telefono, email FROM proveedores ORDER BY razon_social",
        "ventas" => "
            SELECT v.id_venta, COALESCE(c.nombre, 'Venta General') as cliente, 
                   v.fecha_venta, v.metodo_pago, v.total, v.estado 
            FROM ventas_encabezado v
            LEFT JOIN clientes c ON v.id_cliente = c.id_cliente
            ORDER BY v.fecha_venta DESC",
        "compras" => "
            SELECT c.id_compra, p.razon_social as proveedor, c.folio_factura, 
                   c.fecha_compra, c.total, c.estado 
            FROM compras_encabezado c
            LEFT JOIN proveedores p ON c.id_proveedor = p.id_proveedor
            ORDER BY c.fecha_compra DESC",
        _ => "
            SELECT p.sku, p.nombre, p.genero, p.precio_costo, p.precio_venta, p.stock,
                   m.nombre as marca, f.nombre as familia, c.nombre as concentracion
            FROM perfumes p
            LEFT JOIN marcas m ON p.id_marca = m.id_marca
            LEFT JOIN familias_olfativas f ON p.id_familia = f.id_familia
            LEFT JOIN concentraciones c ON p.id_concentracion = c.id_concentracion
            ORDER BY p.nombre",
    };

    let sql = format!("SELECT COALESCE(json_agg(t), '[]'::json) FROM ({}) t", query);

    let resultado: Value = sqlx::query_scalar(&sql)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(resultado))
} //qPd