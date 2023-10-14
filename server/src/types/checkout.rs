use serde::{Serialize, Deserialize};
use crate::{Source, Price};

pub struct CheckoutBuilder {
  pub name: String,
  pub price: u64,
  pub location_id: String,
  pub subscription_plan_id: String,
}

// ======================= Subscribe Request =======================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutRequest {
  pub idempotency_key: String,
  pub quick_pay: QuickPay,
  pub checkout_options: CheckoutOptions
}

impl CheckoutRequest {
  pub fn new(request: CheckoutBuilder) -> Self {
    Self {
      idempotency_key: uuid::Uuid::new_v4().to_string(),
      quick_pay: QuickPay {
        name: request.name,
        price_money: Price {
          amount: request.price,
          currency: "USD".to_string()
        },
        location_id: request.location_id,
      },
      checkout_options: CheckoutOptions {
        subscription_plan_id: request.subscription_plan_id,
      }
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickPay {
  pub name: String,
  pub price_money: Price,
  pub location_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutOptions {
  pub subscription_plan_id: String,
}

// ======================= Subscribe Response =======================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutResponse {
  pub payment_link: PaymentLink,
  pub related_resources: RelatedResources
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentLink {
  pub id: String,
  pub version: u64,
  pub description: String,
  pub order_id: String,
  pub checkout_options: CheckoutOptions,
  pub url: String,
  pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedResources {
  pub orders: Vec<Order>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
  pub id: String,
  pub location_id: String,
  pub source: Source,
  pub line_items: Vec<LineItem>,
  pub fulfillments: Vec<Fulfillment>,
  pub net_amounts: Vec<NetAmounts>,
  pub created_at: String,
  pub updated_at: String,
  /// DRAFT
  pub state: String,
  pub version: u64,
  pub total_money: Price,
  pub total_tax_money: Price,
  pub total_discount_money: Price,
  pub total_tip_money: Price,
  pub total_service_charge_money: Price,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
  pub uid: String,
  pub name: String,
  pub quantity: String,
  /// ITEM
  pub item_type: String,
  pub base_price_money: Price,
  pub variation_total_price_money: Price,
  pub gross_sales_money: Price,
  pub total_tax_money: Price,
  pub total_discount_money: Price,
  pub total_money: Price,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fulfillment {
  pub uid: String,
  /// DIGITAL
  #[serde(rename = "type")]
  pub type_: String,
  /// PROPOSED
  pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetAmounts {
  pub total_money: Price,
  pub tax_money: Price,
  pub discount_money: Price,
  pub tip_money: Price,
  pub service_charge_money: Price,
}






















