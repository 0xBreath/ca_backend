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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Address {
  /// Street address
  pub address_line_1: Option<String>,
  /// Apartment or suite number
  pub address_line_2: Option<String>,
  /// City
  pub locality: Option<String>,
  /// State
  pub administrative_district_level_1: Option<String>,
  pub postal_code: Option<String>,
  pub country: Option<String>,
}