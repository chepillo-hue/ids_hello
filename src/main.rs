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
use std::time::Duration;
use tower_http::services::{ServeDir, ServeFile};

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
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ids_hello_db".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(300))
        .connect(&database_url)
        .await
        .expect("No se pudo conectar a PostgreSQL");

    let api_routes = Router::new()
        .route("/catalogos/:tabla", get(obtener_catalogo))
        .route("/catalogos/perfumes", post(crear_perfume))
        .route("/seed", post(ejecutar_seed))
        .route("/seed-millones", post(ejecutar_seed_millones))
        .route("/conteo", get(obtener_conteo));

    let app = Router::new()
        .nest("/api", api_routes)
        .nest_service("/public", ServeDir::new("public"))
        .route_service("/", ServeFile::new("public/index.html"))
        .with_state(pool);

    let port_str = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let port: u16 = port_str.parse().expect("PORT invalido");
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("Servidor ERP corriendo en http://0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Endpoint para consultar cuántas ventas y compras existen guardadas
async fn obtener_conteo(
    State(pool): State<PgPool>,
) -> Result<Json<Value>, String> {
    let total_ventas: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM ventas_encabezado")
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let total_compras: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM compras_encabezado")
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(serde_json::json!({
        "total_ventas": total_ventas.0,
        "total_compras": total_compras.0
    })))
}

// Endpoint para disparar 1 millón de registros
async fn ejecutar_seed_millones(
    State(pool): State<PgPool>,
) -> Result<Json<Value>, String> {
    println!("Iniciando generación de 1,000,000 de registros en segundo plano...");

    tokio::spawn(async move {
        if let Err(e) = proceso_carga_millones(pool).await {
            eprintln!("Error durante la carga masiva en segundo plano: {}", e);
        }
    });

    Ok(Json(serde_json::json!({
        "status": "processing",
        "mensaje": "Proceso de generación de 1,000,000 de registros iniciado en segundo plano. Monitorea los Logs en Render."
    })))
}

async fn proceso_carga_millones(pool: PgPool) -> Result<(), String> {
    println!("Desactivando synchronous_commit para optimizar velocidad...");
    sqlx::raw_sql("SET synchronous_commit = OFF;")
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let lotes = 10;
    let registros_por_lote = 100_000;

    for lote in 1..=lotes {
        println!("Insertando lote {}/{} ({} registros de ventas)...", lote, lotes, registros_por_lote);

        let sql_lote = format!(r#"
            INSERT INTO ventas_encabezado (id_cliente, fecha_venta, metodo_pago, total)
            SELECT 
                (1 + floor(random() * 3))::INT,
                NOW() - (random() * interval '365 days'),
                (ARRAY['Efectivo', 'Tarjeta', 'Transferencia'])[1 + floor(random() * 3)::INT],
                (50 + (random() * 500))::NUMERIC(10,2)
            FROM generate_series(1, {});
        "#, registros_por_lote);

        sqlx::raw_sql(&sql_lote)
            .execute(&pool)
            .await
            .map_err(|e| format!("Error en el lote {}: {}", lote, e))?;
    }

    sqlx::raw_sql("SET synchronous_commit = ON;")
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    println!("¡Generación masiva de 1,000,000 de ventas completada con éxito!");
    Ok(())
}

async fn ejecutar_seed(
    State(pool): State<PgPool>,
) -> Result<Json<Value>, String> {
    println!("Recibida petición de carga masiva. Iniciando hilo en segundo plano...");

    tokio::spawn(async move {
        if let Err(e) = proceso_carga_masiva(pool).await {
            eprintln!("Error durante la carga masiva en segundo plano: {}", e);
        }
    });

    Ok(Json(serde_json::json!({
        "status": "processing",
        "mensaje": "Proceso de generación masiva iniciado en segundo plano. Puedes monitorear el avance en los Logs de Render."
    })))
}

async fn proceso_carga_masiva(pool: PgPool) -> Result<(), String> {
    println!("Iniciando inserción masiva de registros en la base de datos...");

    sqlx::raw_sql("SET synchronous_commit = OFF;")
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let sql_masivo = r#"
    DO $$
    DECLARE
        i INT;
        v_id_compra INT;
        v_id_venta INT;
        v_metodos TEXT[] := ARRAY['Efectivo', 'Tarjeta', 'Transferencia'];
    BEGIN
        FOR i IN 1..2000 LOOP
            INSERT INTO compras_encabezado (id_proveedor, folio_factura, fecha_compra, total)
            VALUES (
                (1 + floor(random() * 2))::INT,
                'FACT-2026-' || LPAD(i::text, 5, '0'),
                NOW() - (random() * interval '365 days'),
                0
            ) RETURNING id_compra INTO v_id_compra;

            INSERT INTO compras_detalle (id_compra, id_perfume, cantidad, precio_unitario, subtotal)
            VALUES (
                v_id_compra,
                (1 + floor(random() * 5))::INT,
                (10 + floor(random() * 50))::INT,
                100.00,
                2200.00
            );
        END LOOP;

        FOR i IN 1..10000 LOOP
            INSERT INTO ventas_encabezado (id_cliente, fecha_venta, metodo_pago, total)
            VALUES (
                (1 + floor(random() * 3))::INT,
                NOW() - (random() * interval '180 days'),
                v_metodos[1 + floor(random() * 3)::INT],
                350.00
            ) RETURNING id_venta INTO v_id_venta;

            INSERT INTO ventas_detalle (id_venta, id_perfume, cantidad, precio_unitario, subtotal)
            VALUES (
                v_id_venta,
                (1 + floor(random() * 5))::INT,
                (1 + floor(random() * 3))::INT,
                175.00,
                350.00
            );
        END LOOP;
    END $$;
    "#;

    sqlx::raw_sql(sql_masivo)
        .execute(&pool)
        .await
        .map_err(|e| format!("Error ejecutando la carga masiva: {}", e))?;

    sqlx::raw_sql("SET synchronous_commit = ON;")
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    println!("¡Generación masiva completada con éxito!");
    Ok(())
}

async fn crear_perfume(
    State(pool): State<PgPool>,
    Json(payload): Json<NuevoPerfumeInput>,
) -> Result<Json<RespuestaTransaccion>, String> {
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
            ORDER BY v.fecha_venta DESC LIMIT 100",
        "compras" => "
            SELECT c.id_compra, p.razon_social as proveedor, c.folio_factura, 
                   c.fecha_compra, c.total, c.estado 
            FROM compras_encabezado c
            LEFT JOIN proveedores p ON c.id_proveedor = p.id_proveedor
            ORDER BY c.fecha_compra DESC LIMIT 100",
        _ => "
            SELECT p.sku, p.nombre, p.genero, p.precio_costo, p.precio_venta, p.stock,
                   m.nombre as marca, f.nombre as familia, c.nombre as concentracion
            FROM perfumes p
            LEFT JOIN marcas m ON p.id_marca = m.id_marca
            LEFT JOIN familias_olfativas f ON p.id_familia = f.id_familia
            LEFT JOIN concentraciones c ON p.id_concentracion = c.id_concentracion
            ORDER BY p.nombre LIMIT 100",
    };

    let sql = format!("SELECT COALESCE(json_agg(t), '[]'::json) FROM ({}) t", query);

    let resultado: Value = sqlx::query_scalar(&sql)
        .fetch_one(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(resultado))
}