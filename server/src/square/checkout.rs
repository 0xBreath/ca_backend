use serde::{Serialize, Deserialize};
use crate::{Source, Price};
use crate::{Address, SubscriptionPlanResponseObject};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEmailRequest {
  pub email: Option<String>
}

pub struct CheckoutBuilder {
  pub name: String,
  pub price: u64,
  pub location_id: String,
  pub subscription_plan_id: String,
  pub redirect_url: String,
  pub buyer_email: Option<String>,
}

// ======================= Subscribe Request =======================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutRequest {
  pub idempotency_key: String,
  pub quick_pay: QuickPay,
  pub pre_populated_data: Option<PrePopulatedData>,
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
      pre_populated_data: Some(PrePopulatedData {
        buyer_email: request.buyer_email,
        ..Default::default()
      }),
      checkout_options: CheckoutOptions {
        subscription_plan_id: request.subscription_plan_id,
        redirect_url: Some(request.redirect_url),
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CheckoutOptions {
  pub subscription_plan_id: String,
  pub redirect_url: Option<String>,
}

// ======================= Subscribe Response =======================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutResponse {
  pub payment_link: PaymentLink,
  pub related_resources: RelatedResources
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentLink {
  pub checkout_options: CheckoutOptions,
  pub pre_populated_data: Option<PrePopulatedData>,
  pub created_at: String,
  pub id: String,
  pub long_url: String,
  pub order_id: String,
  pub url: String,
  pub version: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrePopulatedData {
  pub buyer_email: Option<String>,
  pub buy_phone_number: Option<String>,
  pub buyer_address: Option<Address>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedResources {
  pub orders: Vec<Order>,
  pub subscription_plans: Vec<SubscriptionPlanResponseObject>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
  pub created_at: String,
  pub fulfillments: Vec<Fulfillment>,
  pub id: String,
  pub line_items: Vec<LineItem>,
  pub location_id: String,
  pub net_amount_due_money: Price,
  pub net_amounts: NetAmounts,
  pub source: Source,
  pub state: String,
  pub total_discount_money: Price,
  pub total_money: Price,
  pub total_service_charge_money: Price,
  pub total_tax_money: Price,
  pub total_tip_money: Price,
  pub updated_at: String,
  pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
  pub base_price_money: Price,
  pub gross_sales_money: Price,
  pub item_type: String,
  pub name: String,
  pub quantity: String,
  pub total_discount_money: Price,
  pub total_money: Price,
  pub total_service_charge_money: Price,
  pub total_tax_money: Price,
  pub uid: String,
  pub variation_total_price_money: Price,
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
  pub discount_money: Price,
  pub service_charge_money: Price,
  pub tax_money: Price,
  pub tip_money: Price,
  pub total_money: Price,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutInfo {
  pub url: String,
  /// Amount is in cents
  pub amount: u64
}