use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PerfumeResponseDto {
    pub id_perfume: i32,
    pub sku: String,
    pub nombre: String,
    pub id_marca: i32,
    pub id_familia: i32,
    pub id_concentracion: i32,
    pub genero: Option<String>,
    pub ml: i32,
    pub precio_costo: Decimal,
    pub precio_venta: Decimal,
    pub imagen_url: Option<String>,
    pub activo: bool,
}

#[derive(Deserialize)]
pub struct CrearPerfumeDto {
    pub sku: String,
    pub nombre: String,
    pub id_marca: i32,
    pub id_familia: i32,
    pub id_concentracion: i32,
    pub genero: Option<String>,
    pub ml: i32,
    pub precio_costo: Decimal,
    pub precio_venta: Decimal,
    pub imagen_url: Option<String>,
}