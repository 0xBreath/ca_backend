use serde::{Serialize, Deserialize};

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
        // todo: annual subscription discount
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
  pub discounts: Option<Vec<Discount>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineItemRequest {
  /// u32 as String
  pub quantity: String,
  pub catalog_object_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Discount {
  pub catalog_object_id: String,
  /// ORDER
  pub scope: String,
}

// ==================== Order Response ====================

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct OrderResponse {
//   pub order: OrderResponseObject
// }
//
// pub struct OrderResponseObject {
//   pub id: String,
//   pub location_id: String,
//   pub line_items: Vec<LineItemResponse>,
// }
//
// pub struct LineItemResponse {
//   pub uid: String,
//   pub catalog_object_id: String,
//   pub catalog_version: u64,
//   pub quantity: String,
//   pub name: String,
//   pub variation_name: String,
// }