use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationListResponse {
  pub locations: Vec<LocationResponse>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationResponse {
  pub id: String,
  pub name: String,
  pub address: Address,
  /// UTC
  pub timezone: String,
  /// CREDIT_CARD_PROCESSING, AUTOMATIC_TRANSFERS
  pub capabilities: Vec<String>,
  /// ACTICE
  pub status: String,
  pub created_at: String,
  pub merchant_id: String,
  pub country: String,
  pub language_code: String,
  pub currency: String,
  pub business_name: String,
  /// PHYSICAL
  #[serde(rename = "type")]
  pub type_: String,
  pub business_hours: BusinessHours,
  pub mcc: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BusinessHours {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
  pub address_line_1: String,
  pub locality: String,
  pub administrative_district_level_1: String,
  pub postal_code: String,
  pub country: String,
}