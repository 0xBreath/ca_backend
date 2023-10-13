use serde::{Serialize, Deserialize};

// ======================= Create Customer Request =======================

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerRequest {
  pub email_address: String,
  pub family_name: String,
  pub given_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Preferences {
  pub email_unsubscribed: bool,
}

// ======================= Create Customer Response =======================

#[derive(Debug, Serialize, Deserialize)]
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