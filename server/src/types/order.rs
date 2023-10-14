use serde::{Serialize, Deserialize};
use crate::types::Price;

// ==================== Order Request ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderBuilder {
  pub location_id: String,
  pub catalog_object_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderRequest {
  pub idempotency_key: String,
  pub order: OrderRequestObject
}

impl OrderRequest {
  pub fn new(request: OrderBuilder) -> Self {
    Self {
      idempotency_key: uuid::Uuid::new_v4().to_string(),
      order: OrderRequestObject {
        location_id: request.location_id,
        state: "DRAFT".to_string(),
        line_items: vec![
          LineItemRequest {
            quantity: "1".to_string(),
            catalog_object_id: request.catalog_object_id,
          }
        ],
        discounts: None
      }
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderRequestObject {
  pub location_id: String,
  /// DRAFT
  pub state: String,
  pub line_items: Vec<LineItemRequest>,
  pub discounts: Option<Vec<DiscountRequest>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineItemRequest {
  /// u32 as String
  pub quantity: String,
  pub catalog_object_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscountRequest {
  pub catalog_object_id: String,
  /// ORDER
  pub scope: String,
}

// ==================== Order Response ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderResponse {
  pub order: OrderResponseObject
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderResponseObject {
  pub id: String,
  pub location_id: String,
  pub line_items: Vec<LineItemResponse>,
  pub discounts: Option<Vec<DiscountResponse>>,
  pub created_at: String,
  pub updated_at: String,
  pub state: String,
  pub version: u64,
  pub total_tax_money: Price,
  pub total_discount_money: Price,
  pub total_tip_money: Price,
  pub total_money: Price,
  pub total_service_charge_money: Price,
  pub net_amounts: NetAmounts,
  pub source: Source,
  pub net_amount_due: Price
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineItemResponse {
  pub uid: String,
  pub catalog_object_id: String,
  pub catalog_version: u64,
  pub quantity: String,
  pub name: String,
  pub variation_name: String,
  pub base_price_money: Price,
  pub gross_sales_money: Price,
  pub total_tax_money: Price,
  pub total_discount_money: Price,
  pub total_money: Price,
  pub variation_total_price_money: Price,
  pub applied_discounts: Vec<AppliedDiscount>,
  pub item_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppliedDiscount {
  pub uid: String,
  pub discount_uid: String,
  pub applied_money: Price,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscountResponse {
  pub uid: String,
  pub catalog_object_id: String,
  pub catalog_version: u64,
  pub name: String,
  pub percentage: String,
  pub applied_money: Price,
  #[serde(rename = "type")]
  pub type_: String,
  pub scope: String,
  pub apply_per_quantity: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetAmounts {
  pub total_money: Price,
  pub tax_money: Price,
  pub discount_money: Price,
  pub tip_money: Price,
  pub service_charge_money: Price,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
  pub name: String,
}






















