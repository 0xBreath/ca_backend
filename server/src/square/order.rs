use crate::square::{Price, Source};
use serde::{Deserialize, Serialize};

pub struct SearchOrdersRequestBuilder {
    pub location_ids: Vec<String>,
    pub customer_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchOrdersRequest {
    pub location_ids: Vec<String>,
    pub query: Option<SearchOrdersQuery>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchOrdersQuery {
    pub filter: SearchOrdersFilter,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchOrdersFilter {
    pub customer_filter: CustomerFilter,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerFilter {
    pub customer_ids: Vec<String>,
}

impl SearchOrdersRequest {
    pub fn new(request: SearchOrdersRequestBuilder) -> Self {
        match &request.customer_ids {
            None => Self {
                location_ids: request.location_ids,
                query: None,
            },
            Some(customer_ids) => Self {
                location_ids: request.location_ids,
                query: Some(SearchOrdersQuery {
                    filter: SearchOrdersFilter {
                        customer_filter: CustomerFilter {
                            customer_ids: customer_ids.clone(),
                        },
                    },
                }),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchOrdersResponse {
    pub orders: Vec<OrderObject>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderObject {
    pub id: String,
    pub location_id: String,
    pub line_items: Vec<LineItem>,
    pub fulfillments: Vec<Fulfillment>,
    pub discounts: Vec<Discount>,
    pub created_at: String,
    pub updated_at: String,
    pub state: String,
    pub version: u64,
    pub total_tax_money: Price,
    pub total_discount_money: Price,
    pub total_tip_money: Price,
    pub total_money: Price,
    pub tenders: Vec<Tender>,
    pub total_service_charge_money: Price,
    pub net_amounts: NetAmounts,
    pub source: Source,
    pub customer_id: String,
    pub net_amount_due_money: Price,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tender {
    pub id: String,
    pub location_id: String,
    pub transaction_id: String,
    pub created_at: String,
    pub amount_money: Price,
    #[serde(rename = "type")]
    pub type_: String,
    pub card_details: CardDetails,
    pub tip_money: Price,
    pub payment_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardDetails {
    pub status: String,
    pub card: Card,
    pub entry_method: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    pub card_brand: String,
    pub last_4: String,
    pub fingerprint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fulfillment {
    pub uid: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineItem {
    pub uid: String,
    pub catalog_object_id: Option<String>,
    pub catalog_version: Option<u64>,
    pub quantity: String,
    pub name: String,
    pub variation_name: Option<String>,
    pub base_price_money: Price,
    pub gross_sales_money: Price,
    pub total_tax_money: Price,
    pub total_service_charge_money: Price,
    pub total_discount_money: Price,
    pub total_money: Price,
    pub variation_total_price_money: Price,
    pub item_type: Option<String>,
    pub applied_discounts: Vec<AppliedDiscount>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppliedDiscount {
    pub uid: String,
    pub discount_uid: String,
    pub applied_money: Price,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Discount {
    pub uid: String,
    pub name: String,
    pub percentage: String,
    pub applied_money: Price,
    #[serde(rename = "type")]
    pub type_: String,
    pub scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetAmounts {
    pub total_money: Price,
    pub tax_money: Price,
    pub discount_money: Price,
    pub tip_money: Price,
    pub service_charge_money: Price,
}
