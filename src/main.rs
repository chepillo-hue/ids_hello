use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    // 1. Rutas de la API
    let api_routes = Router::new()
        .route("/api/catalogos/perfumes", get(obtener_perfumes));

    // 2. Servir 'static/index.html' en la raíz "/" y archivos estáticos
    let app = Router::new()
        .nest("", api_routes)
        // Servir index.html cuando entren a "/"
        .route_service("/", ServeFile::new("public/index.html")) 
        // Servir cualquier otro archivo estático (CSS, JS, imágenes dentro de static)
        .fallback_service(ServeDir::new("public"));

    // 3. Obtener Puerto (local o Render)
    let port_str = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let port: u16 = port_str.parse().expect("PORT debe ser un número válido");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Servidor iniciado en http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Ejemplo temporal para verificar que responda la API
async fn obtener_perfumes() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!([
        {
            "id_perfume": 1,
            "sku": "PERF-001",
            "nombre": "Acqua Di Gio",
            "genero": "Hombre",
            "precio_costo": "850.00",
            "precio_venta": "1950.00",
            "imagen_url": "https://images.unsplash.com/photo-1523293182086-7651a899d37f?w=200"
        }
    ]))
}