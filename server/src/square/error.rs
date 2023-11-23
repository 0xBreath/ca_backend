use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SquareResponse<T> {
    Success(T),
    Error(SquareErrorResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquareErrorResponse {
    pub errors: Vec<SquareError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquareError {
    pub category: String,
    pub code: String,
    pub detail: String,
    pub field: String,
}

impl SquareErrorResponse {
    pub fn from_value(value: serde_json::Value) -> SquareErrorResponse {
        serde_json::from_value(value).unwrap()
    }
}
