use sqlx::{PgPool, Row};
use crate::dtos::aeropuerto::{Aeropuerto, CrearAeropuertoDto, ActualizarAeropuertoDto, ResumenVuelo};

pub struct AeropuertoService;

impl AeropuertoService {
    pub async fn listar_resumen_vuelos(pool: &PgPool) -> Result<Vec<ResumenVuelo>, String> {
        let query = r#"
            SELECT 
                status,
                COUNT(*) AS total_vuelos
            FROM bookings.flights
            GROUP BY status
        "#;

        let rows = sqlx::query(query)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Error en BD: {}", e))?;

        let mut resultados = Vec::new();
        for row in rows {
            resultados.push(ResumenVuelo {
                estado: row.get("status"),
                total_vuelos: row.get("total_vuelos"),
            });
        }

        Ok(resultados)
    }

    pub async fn listar(pool: &PgPool) -> Result<Vec<Aeropuerto>, String> {
        let query = r#"
            SELECT 
                airport_code,
                airport_name->>'en' AS nombre,
                city->>'en' AS ciudad
            FROM bookings.airports_data
            ORDER BY airport_code
            LIMIT 50
        "#;

        let rows = sqlx::query(query)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Error en BD: {}", e))?;

        let mut resultados = Vec::new();
        for row in rows {
            resultados.push(Aeropuerto {
                codigo: row.get("airport_code"),
                nombre: row.get("nombre"),
                ciudad: row.get("ciudad"),
            });
        }

        Ok(resultados)
    }

    pub async fn obtener_por_codigo(pool: &PgPool, codigo: &str) -> Result<Option<Aeropuerto>, String> {
        let query = r#"
            SELECT 
                airport_code,
                airport_name->>'en' AS nombre,
                city->>'en' AS ciudad
            FROM bookings.airports_data
            WHERE airport_code = $1
        "#;

        let row = sqlx::query(query)
            .bind(codigo)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("Error en BD: {}", e))?;

        Ok(row.map(|r| Aeropuerto {
            codigo: r.get("airport_code"),
            nombre: r.get("nombre"),
            ciudad: r.get("ciudad"),
        }))
    }

    pub async fn crear(pool: &PgPool, dto: CrearAeropuertoDto) -> Result<Aeropuerto, String> {
        let query = r#"
            INSERT INTO bookings.airports_data (airport_code, airport_name, city, coordinates, timezone)
            VALUES (
                $1, 
                jsonb_build_object('en', $2::text), 
                jsonb_build_object('en', $3::text), 
                '(0,0)'::point, 
                'UTC'
            )
            RETURNING airport_code, airport_name->>'en' AS nombre, city->>'en' AS ciudad
        "#;

        let row = sqlx::query(query)
            .bind(&dto.codigo)
            .bind(&dto.nombre)
            .bind(&dto.ciudad)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Error al insertar: {}", e))?;

        Ok(Aeropuerto {
            codigo: row.get("airport_code"),
            nombre: row.get("nombre"),
            ciudad: row.get("ciudad"),
        })
    }

    pub async fn actualizar(pool: &PgPool, codigo: &str, dto: ActualizarAeropuertoDto) -> Result<Option<Aeropuerto>, String> {
        let query = r#"
            UPDATE bookings.airports_data
            SET 
                airport_name = COALESCE(
                    CASE WHEN $2::text IS NOT NULL THEN jsonb_build_object('en', $2::text) END, 
                    airport_name
                ),
                city = COALESCE(
                    CASE WHEN $3::text IS NOT NULL THEN jsonb_build_object('en', $3::text) END, 
                    city
                )
            WHERE airport_code = $1
            RETURNING airport_code, airport_name->>'en' AS nombre, city->>'en' AS ciudad
        "#;

        let row = sqlx::query(query)
            .bind(codigo)
            .bind(dto.nombre)
            .bind(dto.ciudad)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("Error al actualizar: {}", e))?;

        Ok(row.map(|r| Aeropuerto {
            codigo: r.get("airport_code"),
            nombre: r.get("nombre"),
            ciudad: r.get("ciudad"),
        }))
    }

    pub async fn eliminar(pool: &PgPool, codigo: &str) -> Result<bool, String> {
        let query = "DELETE FROM bookings.airports_data WHERE airport_code = $1";

        let result = sqlx::query(query)
            .bind(codigo)
            .execute(pool)
            .await
            .map_err(|e| format!("Error al eliminar: {}", e))?;

        Ok(result.rows_affected() > 0)
    }
}