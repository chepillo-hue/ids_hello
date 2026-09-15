use sqlx::PgPool;
use crate::dtos::perfume::{CrearPerfumeDto, PerfumeResponseDto};

pub struct PerfumeService;

impl PerfumeService {
    pub async fn listar_todos(pool: &PgPool) -> Result<Vec<PerfumeResponseDto>, sqlx::Error> {
        sqlx::query_as!(
            PerfumeResponseDto,
            "SELECT id_perfume, sku, nombre, id_marca, id_familia, id_concentracion, genero, ml, precio_costo, precio_venta, imagen_url, activo FROM perfumes WHERE activo = TRUE ORDER BY id_perfume DESC"
        )
        .fetch_all(pool)
        .await
    }

    pub async fn crear(pool: &PgPool, dto: CrearPerfumeDto) -> Result<PerfumeResponseDto, sqlx::Error> {
        sqlx::query_as!(
            PerfumeResponseDto,
            r#"
            INSERT INTO perfumes (sku, nombre, id_marca, id_familia, id_concentracion, genero, ml, precio_costo, precio_venta, imagen_url)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id_perfume, sku, nombre, id_marca, id_familia, id_concentracion, genero, ml, precio_costo, precio_venta, imagen_url, activo
            "#,
            dto.sku,
            dto.nombre,
            dto.id_marca,
            dto.id_familia,
            dto.id_concentracion,
            dto.genero,
            dto.ml,
            dto.precio_costo,
            dto.precio_venta,
            dto.imagen_url
        )
        .fetch_one(pool)
        .await
    }
}