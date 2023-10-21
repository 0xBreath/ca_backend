use serde::{Serialize, Deserialize};
use crate::Address;

// ======================= Create Customer Request =======================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerRequest {
  pub email_address: String,
  pub family_name: String,
  pub given_name: String,
  pub address: Address
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
  pub email_unsubscribed: bool,
}

// ======================= Create Customer Response =======================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerResponse {
  pub created_at: String,
  pub creation_source: String,
  pub email_address: String,
  pub family_name: String,
  pub given_name: String,
  pub id: String,
  pub preferences: Preferences,
  pub updated_at: String,
  pub version: u64,
  pub cards: Option<Vec<Card>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
  pub id: String,
  pub card_brand: String,
  pub last_4: String,
  pub exp_month: u8,
  pub exp_year: u16,
  pub cardholder_name: String,
  pub billing_address: Address,
}

// ======================= Update Customer Response =======================

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCustomerResponse {
  pub customer: CustomerResponse
}

// ======================= Search Customer Request =======================

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchCustomerExact {
  exact: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchCustomerEmailAddress {
  email_address: SearchCustomerExact
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchCustomerFilter {
  filter: SearchCustomerEmailAddress
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchCustomerRequest {
  query: SearchCustomerFilter
}

impl SearchCustomerRequest {
  pub fn new(email: String) -> Self {
    Self {
      query: SearchCustomerFilter {
        filter: SearchCustomerEmailAddress {
          email_address: SearchCustomerExact {
            exact: email
          }
        }
      }
    }
  }

  pub fn to_value(&self) -> serde_json::Result<serde_json::Value> {
    serde_json::to_value(self)
  }
}

// ======================= Search Customer Response =======================

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchCustomerResponse {
  pub customers: Vec<CustomerResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerListResponse {
  pub customers: Vec<CustomerResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerInfo {
  pub email_address: String,
  pub family_name: String,
  pub given_name: String,
  pub cards: Option<Vec<CardInfo>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardInfo {
  pub card_brand: String,
  pub last_4: String,
  pub exp_month: u8,
  pub exp_year: u16,
  pub cardholder_name: String,
}