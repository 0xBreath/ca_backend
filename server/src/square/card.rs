use serde::{Serialize, Deserialize};
use crate::{Address, CustomerRequest};

pub struct CardBuilder {
  pub customer: CustomerRequest,
  pub customer_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardRequest {
  pub idempotency_key: String,
  /// card token (from seller account) or payment_id
  /// ccof:customer-card-id-ok
  pub source_id: String,
  pub card: CardRequestObject
}

impl CardRequest {
  pub fn new(request: CardBuilder) -> Self {
    Self {
      idempotency_key: uuid::Uuid::new_v4().to_string(),
      source_id: "ccof:customer-card-id-ok".to_string(),
      card: CardRequestObject {
        billing_address: request.customer.address,
        cardholder_name: format!("{} {}", request.customer.given_name, request.customer.family_name),
        customer_id: request.customer_id,
        reference_id: request.customer.email_address,
      }
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardRequestObject {
  pub billing_address: Address,
  /// Pull from [`CustomerResponse`](crate::CustomerResponse) first and last name
  pub cardholder_name: String,
  /// Pull from [`CustomerResponse`](crate::CustomerResponse) id
  pub customer_id: String,
  pub reference_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardResponse {
  pub card: CardResponseObject
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardResponseObject {
  pub id: String,
  pub billing_address: Address,
  pub fingerprint: Option<String>,
  pub bin: String,
  /// ex: VISA, AMERICAN_EXPRESS
  pub card_brand: String,
  /// CREDIT
  pub card_type: String,
  /// First and last name
  pub cardholder_name: String,
  pub customer_id: String,
  pub enabled: bool,
  pub exp_month: u8,
  pub exp_year: u16,
  pub last_4: String,
  pub merchant_id: Option<String>,
  /// NOT_PREPAID
  pub prepaid_type: String,
  /// External tracking id
  pub reference_id: String,
  pub version: u64,
}
/*
https://developer.squareup.com/docs/checkout-api/subscription-plan-checkout
*/










