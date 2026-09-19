use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};

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

    // 2. Rutas Dinámicas de la API
    let api_routes = Router::new()
        .route("/catalogos/:tabla", get(obtener_catalogo));

    // 3. Router Principal y Archivos Estáticos
    let app = Router::new()
        .nest("/api", api_routes)
        .nest_service("/public", ServeDir::new("public"))
        .route_service("/", ServeFile::new("public/index.html"))
        .with_state(pool);

    // 4. Iniciar Servidor
    let port_str = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let port: u16 = port_str.parse().expect("PORT invalido");
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("Servidor ERP corriendo en http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Handler dinámico multicatálogo
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

    // Usamos query_scalar para deserializar directamente el arreglo JSON de Postgres
    let sql = format!("SELECT COALESCE(json_agg(t), '[]'::json) FROM ({}) t", query);

    let resultado: Value = sqlx::query_scalar(&sql)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(resultado))
} // Rebuild side menu