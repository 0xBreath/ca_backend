use serde::{Serialize, Deserialize};
use crate::types::Price;

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceListResponse {
  pub invoices: Vec<Invoice>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Invoice {
  pub accepted_payment_methods: PaymentMethod,
  pub created_at: String,
  pub delivery_method: String,
  pub id: String,
  pub invoice_number: String,
  pub location_id: String,
  pub order_id: String,
  pub payment_requests: Vec<PaymentRequest>,
  pub primary_recipient: Recipient,
  pub public_url: String,
  pub status: String,
  pub store_payment_method_enabled: bool,
  pub subscription_id: String,
  pub timezone: String,
  pub title: String,
  pub updated_at: String,
  pub version: u64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentMethod {
  pub bank_account: bool,
  pub buy_now_pay_later: bool,
  pub card: bool,
  pub cash_app_pay: bool,
  pub square_gift_card: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentRequest {
  pub automatic_payment_source: String,
  pub card_id: String,
  pub computed_amount_money: Price,
  pub due_date: String,
  pub request_type: String,
  pub tipping_enabled: bool,
  pub total_completed_amount_money: Price,
  pub uid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recipient {
  pub customer_id: String,
  pub email_address: String,
  pub family_name: String,
  pub given_name: String,
  pub phone_number: String,
}






















