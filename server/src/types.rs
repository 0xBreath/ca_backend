use serde::{Serialize, Deserialize};


// ======================= Create Customer =======================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCustomerRequest {
  pub email_address: String,
  pub family_name: String,
  pub given_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Preferences {
  pub email_unsubscribed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Customer {
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


// ======================= Search Customer =======================

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchCustomerResponse {
  pub customers: Vec<Customer>,
}

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
pub struct SearchCustomerQuery {
  query: SearchCustomerFilter
}

impl SearchCustomerQuery {
  pub fn new(email: String) -> SearchCustomerQuery {
    SearchCustomerQuery {
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