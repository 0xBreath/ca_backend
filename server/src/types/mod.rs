pub mod card;
pub mod catalog;
pub mod customer;
pub mod checkout;
pub mod location;
pub mod subscription;
pub mod invoice;

pub use card::*;
pub use catalog::*;
pub use customer::*;
pub use checkout::*;
pub use location::*;
pub use subscription::*;
pub use invoice::*;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
  pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Price {
  /// Amount is smallest denomination of currency, so cents for USD
  /// Ex: 1295 for $12.95
  pub amount: u64,
  pub currency: String
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pricing {
  /// STATIC
  #[serde(rename = "type")]
  pub type_: String,
  pub price: Option<Price>,
  pub price_money: Option<Price>
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomerEmailInfo {
  pub email_address: String,
  pub family_name: String,
  pub given_name: String,
}