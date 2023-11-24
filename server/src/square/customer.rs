use crate::square::{SquareErrorResponse, SquareResponse};
use crate::Address;
use serde::{Deserialize, Serialize};

// ======================= Create Customer Request =======================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerRequest {
    pub email_address: String,
    pub family_name: String,
    pub given_name: String,
    pub address: Address,
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
    pub cards: Option<Vec<Card>>,
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
    pub customer: CustomerResponse,
}

// ======================= Search Customer Request =======================

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchCustomerExact {
    exact: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchCustomerEmailAddress {
    email_address: SearchCustomerExact,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchCustomerFilter {
    filter: SearchCustomerEmailAddress,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchCustomerRequest {
    query: SearchCustomerFilter,
}

impl SearchCustomerRequest {
    pub fn new(email: String) -> Self {
        Self {
            query: SearchCustomerFilter {
                filter: SearchCustomerEmailAddress {
                    email_address: SearchCustomerExact { exact: email },
                },
            },
        }
    }

    pub fn to_value(&self) -> serde_json::Result<serde_json::Value> {
        serde_json::to_value(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCustomerAttributeRequest {
    pub idempotency_key: String,
    pub custom_attribute: CustomAttribute,
}

impl UpdateCustomerAttributeRequest {
    pub fn new(key: String, value: u8) -> Self {
        Self {
            idempotency_key: uuid::Uuid::new_v4().to_string(),
            custom_attribute: CustomAttribute {
                key,
                value: value.to_string(),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomAttribute {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCustomAttributeRequest {
    pub idempotency_key: String,
    pub custom_attribute_definition: CustomAttributeDefinitionRequest,
}

impl CreateCustomAttributeRequest {
    pub fn new(key: String) -> Self {
        Self {
            idempotency_key: uuid::Uuid::new_v4().to_string(),
            custom_attribute_definition: CustomAttributeDefinitionRequest {
                key,
                schema: CustomAttributeSchemaObject {
                    reference: "https://developer-production-s.squarecdn.com/schemas/v1/common.json#squareup.common.Number".to_string(),
                },
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomAttributeDefinitionRequest {
    pub key: String,
    pub schema: CustomAttributeSchemaObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAttributeSchemaObject {
    /// "https://developer-production-s.squarecdn.com/schemas/v1/common.json#squareup.common.Number"
    #[serde(rename = "$ref")]
    pub reference: String,
}

// ======================= Search Customer Response =======================

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub cards: Option<Vec<CardInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardInfo {
    pub card_brand: String,
    pub last_4: String,
    pub exp_month: u8,
    pub exp_year: u16,
    pub cardholder_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAttributeResponses {
    pub sessions: SquareResponse<CreateCustomAttributeResponse>,
    pub sessions_credited: SquareResponse<CreateCustomAttributeResponse>,
    pub sessions_debited: SquareResponse<CreateCustomAttributeResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCustomAttributeResponse {
    pub custom_attribute_definition: CustomAttributeDefinitionResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAttributeDefinitionResponse {
    pub key: String,
    pub version: u64,
    pub updated_at: String,
    pub schema: CustomAttributeSchemaObject,
    pub created_at: String,
    pub visibility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerAttributesResponse {
    pub sessions: CustomerAttributeResponse,
    pub sessions_credited: CustomerAttributeResponse,
    pub sessions_debited: CustomerAttributeResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerAttributeResponse {
    pub custom_attribute: CustomAttributeResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAttributeResponse {
    pub key: String,
    pub version: u64,
    pub updated_at: String,
    pub value: String,
    pub created_at: String,
    pub visibility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSessions {
    pub email: Option<String>,
    pub sessions: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeltaSessions {
    Reset,
    Increment(u8),
    Decrement(u8),
}

impl DeltaSessions {
    pub fn delta(&self, existing: Option<u8>) -> u8 {
        match existing {
            None => match self {
                DeltaSessions::Reset => 0,
                _ => self.checked_delta(0),
            },
            Some(existing) => match self {
                DeltaSessions::Reset => 0,
                _ => self.checked_delta(existing),
            },
        }
    }

    fn checked_delta(&self, existing: u8) -> u8 {
        match self {
            DeltaSessions::Reset => 0,
            DeltaSessions::Increment(delta) => {
                let res = (existing as i16) + (*delta as i16);
                if res < 0 {
                    0
                } else {
                    res as u8
                }
            }
            DeltaSessions::Decrement(delta) => {
                let res = (existing as i16) - (*delta as i16);
                if res < 0 {
                    0
                } else {
                    res as u8
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsInfo {
    pub email: String,
    pub sessions: u8,
    pub sessions_credited: u8,
    pub sessions_debited: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsInfoUpdate {
    pub sessions: DeltaSessions,
    pub sessions_credited: DeltaSessions,
    pub sessions_debited: DeltaSessions,
}

pub enum SessionAttribute {
    Sessions,
    SessionsCredited,
    SessionsDebited,
}

impl SessionAttribute {
    pub fn key(&self) -> String {
        match self {
            SessionAttribute::Sessions => "sessions".to_string(),
            SessionAttribute::SessionsCredited => "sessions_credited".to_string(),
            SessionAttribute::SessionsDebited => "sessions_debited".to_string(),
        }
    }
}
