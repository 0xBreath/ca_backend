use serde_repr::{Serialize_repr, Deserialize_repr};

#[derive(Debug, Clone, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum CoachingPackage {
  Single,
  Three,
  Six,
  Ten
}

impl CoachingPackage {
  pub fn name(&self) -> String {
    match self {
      CoachingPackage::Single => "Single Session".to_string(),
      CoachingPackage::Three => "Package of 3".to_string(),
      CoachingPackage::Six => "Package of 6".to_string(),
      CoachingPackage::Ten => "Package of 10".to_string()
    }
  }
}