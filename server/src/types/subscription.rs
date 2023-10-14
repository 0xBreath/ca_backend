use serde::{Serialize, Deserialize};

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
  pub id: String,
  pub location_id: String,
  pub customer_id: String,
  pub start_date: String,
  /// ACTIVE
  pub status: String,
  pub version: u64,
  pub created_at: String,
  /// UTC
  pub timezone: String,
  pub source: Source,
  pub phases: Option<Vec<PlanPhaseResponse>>,
  pub plan_variation_id: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanPhaseResponse {
  pub uid: String,
  pub ordinal: u64,
  pub order_template_id: String,
  pub plan_phase_uid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
  pub name: String,
}