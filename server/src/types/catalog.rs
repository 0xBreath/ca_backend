use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogBuilder {
  pub name: String,
  pub price: u64,
  pub id: String
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Price {
  // amount is smallest denomination of currency, so cents for USD
  pub amount: u64,
  pub currency: String
}



// ======================= Subscription Plan Request =======================

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Pricing {
  pub price: Price,
  /// STATIC
  #[serde(rename = "type")]
  pub type_: String
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Phase {
  pub uid: Option<String>,
  pub cadence: String,
  pub ordinal: u32,
  pub periods: Option<u32>,
  pub pricing: Pricing
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionPlanData {
  pub name: String,
  pub all_items: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionPlanVariationData {
  pub name: String,
  pub phases: Vec<Phase>,
  pub subscription_plan_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogRequestObject {
  pub present_at_all_locations: Option<bool>,
  /// SUBSCRIPTION_PLAN
  #[serde(rename = "type")]
  pub type_: String,
  pub id: String,
  pub subscription_plan_data: Option<SubscriptionPlanData>,
  pub subscription_plan_variation_data: Option<SubscriptionPlanVariationData>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogRequest {
  pub object: CatalogRequestObject,
  pub idempotency_key: String,
}

impl CatalogRequest {
  pub fn new(request: CatalogBuilder) -> Self {
    Self {
      object: CatalogRequestObject {
        present_at_all_locations: Some(true),
        type_: "SUBSCRIPTION_PLAN".to_string(),
        id: request.id,
        subscription_plan_data: Some(SubscriptionPlanData {
          name: request.name,
          all_items: Some(true),
        }),
        subscription_plan_variation_data: None
      },
      idempotency_key: uuid::Uuid::new_v4().to_string(),
    }
  }

  pub fn to_value(&self) -> serde_json::Result<serde_json::Value> {
    serde_json::to_value(self)
  }
}

// ======================= Subscription Plan Response =======================

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogResponse {
  pub catalog_object: CatalogResponseObject,
  pub id_mappings: Vec<IdMapping>
}

impl CatalogResponse {
  pub fn subscription_plan(&self, request: CatalogBuilder) -> CatalogRequest {
    CatalogRequest {
      object: CatalogRequestObject {
        present_at_all_locations: Some(true),
        type_: "SUBSCRIPTION_PLAN_VARIATION".to_string(),
        id: request.id,
        subscription_plan_data: None,
        subscription_plan_variation_data: Some(SubscriptionPlanVariationData {
          name: request.name,
          phases: vec![
            Phase {
              cadence: "MONTHLY".to_string(),
              ordinal: 0,
              pricing: Pricing {
                type_: "STATIC".to_string(),
                price: Price {
                  amount: request.price,
                  currency: "USD".to_string()
                },
              },
              ..Default::default()
            }
          ],
          subscription_plan_id: self.catalog_object.id.clone(),
        })
      },
      idempotency_key: uuid::Uuid::new_v4().to_string(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogResponseObject {
  #[serde(rename = "type")]
  pub type_: String,
  pub id: String,
  pub updated_at: String,
  pub created_at: String,
  pub version: u64,
  pub present_at_all_locations: bool,
  pub subscription_plan_data: SubscriptionPlanData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdMapping {
  client_object_id: String,
  object_id: String
}