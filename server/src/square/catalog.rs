use crate::CoachingPackage;
use crate::{Price, Pricing};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionCatalogBuilder {
    pub id: String,
    pub name: String,
    pub price: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachingCatalogBuilder {
    pub id: String,
    pub name: String,
    pub single_session_price: u64,
    pub three_session_price: u64,
    pub six_session_price: u64,
    pub ten_session_price: u64,
}

// ======================= Subscription Plan Request =======================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Phase {
    pub uid: Option<String>,
    pub cadence: String,
    pub ordinal: Option<u32>,
    pub periods: Option<u32>,
    pub pricing: Pricing,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionPlanData {
    pub name: String,
    pub all_items: Option<bool>,
    pub subscription_plan_variations: Option<Vec<SubscriptionPlanResponseObject>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ItemData {
    // request fields
    pub abbreviation: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub variations: Option<Vec<Variation>>,
    // response fields
    pub description_html: Option<String>,
    pub description_plaintext: Option<String>,
    pub is_archived: Option<bool>,
    pub is_taxable: Option<bool>,
    pub product_type: Option<String>,
    pub tax_ids: Option<Vec<String>>,
    pub channels: Option<Vec<String>>,
    pub ecom_available: Option<bool>,
    pub ecom_visibility: Option<String>,
    pub image_ids: Option<Vec<String>>,
    pub visibility: Option<String>,
    pub skip_modifier_screen: Option<bool>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Variation {
    // request fields
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub item_variation_data: ItemVariationData,
    // response fields
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub present_at_all_locations: Option<bool>,
    pub version: Option<u64>,
    pub is_deleted: Option<bool>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ItemVariationData {
    // request fields,
    pub name: String,
    pub price_money: Price,
    pub pricing_type: String,
    pub item_id: String,
    // response fields
    pub sellable: Option<bool>,
    pub stockable: Option<bool>,
    pub channels: Option<Vec<String>>,
    pub location_overrides: Option<Vec<LocationOverride>>,
    pub ordinal: Option<u64>,
    pub subscription_plan_ids: Option<Vec<String>>,
    pub track_inventory: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationOverride {
    pub location_id: String,
    pub track_inventory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPlanVariationData {
    pub name: String,
    pub phases: Vec<Phase>,
    pub subscription_plan_id: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CatalogRequestObject {
    pub present_at_all_locations: Option<bool>,
    /// SUBSCRIPTION_PLAN or ITEM
    #[serde(rename = "type")]
    pub type_: String,
    pub id: String,
    pub subscription_plan_data: Option<SubscriptionPlanData>,
    pub subscription_plan_variation_data: Option<SubscriptionPlanVariationData>,
    pub item_data: Option<ItemData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogRequest {
    pub object: CatalogRequestObject,
    pub idempotency_key: String,
}

impl CatalogRequest {
    pub fn new_subscription_catalog(request: SubscriptionCatalogBuilder) -> Self {
        Self {
            object: CatalogRequestObject {
                present_at_all_locations: Some(true),
                type_: "SUBSCRIPTION_PLAN".to_string(),
                id: request.id,
                subscription_plan_data: Some(SubscriptionPlanData {
                    name: request.name,
                    all_items: Some(true),
                    subscription_plan_variations: None,
                }),
                ..Default::default()
            },
            idempotency_key: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Map item variations to [`CoachingPackage`] names
    pub fn new_coaching_catalog(request: CoachingCatalogBuilder) -> Self {
        Self {
            object: CatalogRequestObject {
                present_at_all_locations: Some(true),
                type_: "ITEM".to_string(),
                id: request.id.clone(),
                item_data: Some(ItemData {
                    abbreviation: Some("Coaching".to_string()),
                    description: Some("Coaching with Oriana".to_string()),
                    name: Some(request.name),
                    variations: Some(vec![
                        Variation {
                            id: "#1_session".to_string(),
                            type_: "ITEM_VARIATION".to_string(),
                            item_variation_data: ItemVariationData {
                                name: CoachingPackage::Single.name(),
                                price_money: Price {
                                    amount: request.single_session_price,
                                    currency: "USD".to_string(),
                                },
                                pricing_type: "FIXED_PRICING".to_string(),
                                item_id: request.id.clone(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        Variation {
                            id: "#3_session".to_string(),
                            type_: "ITEM_VARIATION".to_string(),
                            item_variation_data: ItemVariationData {
                                name: CoachingPackage::Three.name(),
                                price_money: Price {
                                    amount: request.three_session_price,
                                    currency: "USD".to_string(),
                                },
                                pricing_type: "FIXED_PRICING".to_string(),
                                item_id: request.id.clone(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        Variation {
                            id: "#6_session".to_string(),
                            type_: "ITEM_VARIATION".to_string(),
                            item_variation_data: ItemVariationData {
                                name: CoachingPackage::Six.name(),
                                price_money: Price {
                                    amount: request.six_session_price,
                                    currency: "USD".to_string(),
                                },
                                pricing_type: "FIXED_PRICING".to_string(),
                                item_id: request.id.clone(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        Variation {
                            id: "#10_session".to_string(),
                            type_: "ITEM_VARIATION".to_string(),
                            item_variation_data: ItemVariationData {
                                name: CoachingPackage::Ten.name(),
                                price_money: Price {
                                    amount: request.ten_session_price,
                                    currency: "USD".to_string(),
                                },
                                pricing_type: "FIXED_PRICING".to_string(),
                                item_id: request.id,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    ]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            idempotency_key: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn to_value(&self) -> serde_json::Result<serde_json::Value> {
        serde_json::to_value(self)
    }
}

// ======================= Subscription Plan Response =======================

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogResponse {
    pub catalog_object: CatalogResponseObject,
    pub id_mappings: Vec<IdMapping>,
}

impl CatalogResponse {
    pub fn subscription_plan(&self, request: SubscriptionCatalogBuilder) -> CatalogRequest {
        CatalogRequest {
            object: CatalogRequestObject {
                present_at_all_locations: Some(true),
                type_: "SUBSCRIPTION_PLAN_VARIATION".to_string(),
                id: request.id,
                subscription_plan_variation_data: Some(SubscriptionPlanVariationData {
                    name: request.name,
                    phases: vec![Phase {
                        cadence: "MONTHLY".to_string(),
                        pricing: Pricing {
                            type_: "STATIC".to_string(),
                            price: None,
                            price_money: Some(Price {
                                amount: request.price,
                                currency: "USD".to_string(),
                            }),
                        },
                        ..Default::default()
                    }],
                    subscription_plan_id: self.catalog_object.id.clone(),
                }),
                ..Default::default()
            },
            idempotency_key: uuid::Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogResponseObject {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: String,
    pub updated_at: String,
    pub created_at: String,
    pub version: u64,
    pub present_at_all_locations: bool,
    pub is_deleted: bool,
    pub subscription_plan_data: Option<SubscriptionPlanData>,
    pub item_data: Option<ItemData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdMapping {
    pub client_object_id: String,
    pub object_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionPlanResponse {
    pub catalog_object: SubscriptionPlanResponseObject,
    pub id_mappings: Vec<IdMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPlanResponseObject {
    pub created_at: String,
    pub id: String,
    pub is_deleted: bool,
    pub present_at_all_locations: bool,
    pub subscription_plan_variation_data: SubscriptionPlanVariationData,
    #[serde(rename = "type")]
    pub type_: String,
    pub updated_at: String,
    pub version: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionPlanListResponse {
    pub objects: Vec<SubscriptionPlanResponseObject>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogListResponse {
    pub objects: Vec<CatalogResponseObject>,
}
