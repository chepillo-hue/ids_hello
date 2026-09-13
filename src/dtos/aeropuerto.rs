use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Aeropuerto {
    pub codigo: String,
    pub nombre: String,
    pub ciudad: String,
}

#[derive(Deserialize)]
pub struct CrearAeropuertoDto {
    pub codigo: String,
    pub nombre: String,
    pub ciudad: String,
}

#[derive(Deserialize)]
pub struct ActualizarAeropuertoDto {
    pub nombre: Option<String>,
    pub ciudad: Option<String>,
}

#[derive(Serialize)]
pub struct ResumenVuelo {
    pub estado: String,
    pub total_vuelos: i64,
}