use serde::{Serialize, Deserialize};
use crate::{Source, Price};
use crate::square::CustomerInfo;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
  pub id: String,
  #[serde(rename = "type")]
  pub type_: String,
  pub effective_date: String,
  pub new_plan_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanPhaseResponse {
  pub uid: String,
  pub ordinal: u64,
  pub order_template_id: String,
  pub plan_phase_uid: String,
}

// ==================== Subscription Search Response ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionSearchResponse {
  pub subscriptions: Vec<SubscriptionResponseObject>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchSubscriptionsRequest {
  pub query: SearchSubscriptionQuery
}

impl SearchSubscriptionsRequest {
  pub fn new(customer_id: String) -> Self {
    Self {
      query: SearchSubscriptionQuery {
        filter: SearchSubscriptionFilter {
          customer_ids: vec![customer_id]
        }
      }
    }
  }

  pub fn to_value(&self) -> serde_json::Result<serde_json::Value> {
    serde_json::to_value(self)
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchSubscriptionQuery {
  pub filter: SearchSubscriptionFilter
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchSubscriptionFilter {
  pub customer_ids: Vec<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionInfo {
  pub title: String,
  pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSubscriptionInfo {
  pub start_date: String,
  pub charged_through_date: Option<String>,
  pub canceled_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
  pub customer: Option<CustomerInfo>,
  pub subscription_info: Option<SubscriptionInfo>,
  pub user_subscription: Option<UserSubscriptionInfo>
}


















