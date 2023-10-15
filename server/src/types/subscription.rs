use serde::{Serialize, Deserialize};
use crate::{Source, Price};

// ==================== Subscription Request ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionRequest {
  pub idempotency_key: String,
  pub customer_id: String,
  pub location_id: String,
  pub plan_variation_id: String,
  pub phases: Option<Vec<PlanPhaseRequest>>
}

impl SubscriptionRequest {
  pub fn new(customer: String, location: String, plan: String) -> Self {
    Self {
      idempotency_key: uuid::Uuid::new_v4().to_string(),
      customer_id: customer.to_string(),
      location_id: location.to_string(),
      plan_variation_id: plan.to_string(),
      phases: None
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanPhaseRequest {
  pub ordinal: u64,
  pub order_template_id: String,
}

// ==================== Subscription Response ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionResponse {
  pub subscription: SubscriptionResponseObject
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionResponseObject {
  pub actions: Option<Vec<Action>>,
  pub buyer_self_management_token: String,
  pub canceled_date: Option<String>,
  pub card_id: Option<String>,
  pub charged_through_date: Option<String>,
  pub created_at: String,
  pub customer_id: String,
  pub id: String,
  pub invoice_ids: Option<Vec<String>>,
  pub location_id: String,
  pub order_template_id: String,
  pub phases: Option<Vec<PlanPhaseResponse>>,
  pub plan_variation_id: String,
  pub source: Source,
  pub start_date: String,
  pub status: String,
  pub timezone: String,
  pub version: u64,
  pub tax_percentage: Option<String>,
  pub price_override_money: Option<Price>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
  pub id: String,
  #[serde(rename = "type")]
  pub type_: String,
  pub effective_date: String,
  pub new_plan_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanPhaseResponse {
  pub uid: String,
  pub ordinal: u64,
  pub order_template_id: String,
  pub plan_phase_uid: String,
}

// ==================== Subscription Search Response ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionSearchResponse {
  pub subscriptions: Vec<SubscriptionResponseObject>
}


















