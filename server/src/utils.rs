use actix_web::{Error};
use crate::types::*;
use log::*;
use reqwest::Client;

pub struct SquareClient {
  pub client: Client,
  pub base_url: String,
  pub token: String,
  pub version: String,
  pub location_id: String,
  pub catalog_id: String,
  pub subscription_price: u64,
  pub subscription_name: String,
}

impl SquareClient {
  pub fn new() -> Self {
    Self {
      client: Client::new(),
      base_url: std::env::var("SQUARE_API_URL").unwrap_or_else(|_| "https://connect.squareupsandbox.com/v2/".to_string()),
      token: std::env::var("SQUARE_ACCESS_TOKEN").unwrap_or_else(|_| "".to_string()),
      version: std::env::var("SQUARE_API_VERSION").unwrap_or_else(|_| "2023-09-25".to_string()),
      location_id: std::env::var("SQUARE_LOCATION_ID").unwrap_or_else(|_| "".to_string()),
      catalog_id: std::env::var("SQUARE_CATALOG_ID").unwrap_or_else(|_| "".to_string()),
      subscription_price: std::env::var("SQUARE_SUBSCRIPTION_PRICE").unwrap_or_else(|_| "".to_string()).parse::<u64>().unwrap(),
      subscription_name: std::env::var("SQUARE_SUBSCRIPTION_NAME").unwrap_or_else(|_| "".to_string()),
    }
  }

  pub async fn update_customer(&self, request: CustomerRequest) -> Result<CustomerResponse, Error> {
    // POST customer search
    let search_customer_endpoint = self.base_url.clone() + "customers/search";
    let query = SearchCustomerRequest::new(request.email_address.clone()).to_value()?;
    let search_res = self.client.post(search_customer_endpoint)
      .header("Square-Version", self.version.clone())
      .bearer_auth(self.token.clone())
      .json(&query)
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST customer search to Square"))?;
    debug!("POST Square search customer: {:?}", &search_res);
    let customer_search = search_res.json::<SearchCustomerResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse SearchCustomerResponse from Square"))?;

    if customer_search.customers.is_empty() {
      // create new customer -> POST
      let create_customer_endpoint = self.base_url.clone() + "customers";
      let res = self.client.post(create_customer_endpoint)
        .header("Square-Version", self.version.clone())
        .bearer_auth(self.token.clone())
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST customer create to Square"))?;
      let res = res.json::<CustomerResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse POST response from Square"))?;
      info!("POST Square create customer: {:?}", &res);

      Ok(res)
    } else {
      // update existing customer to subscribe -> PUT
      let customer_id = customer_search.customers[0].id.clone();
      let update_customer_endpoint = format!("{}customers/{}", self.base_url.clone(), customer_id);
      debug!("update customer endpoint: {}", update_customer_endpoint);

      let res = self.client.put(update_customer_endpoint)
        .header("Square-Version", self.version.clone())
        .bearer_auth(self.token.clone())
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send PUT customer update to Square"))?;
      let res = res.json::<UpdateCustomerResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse PUT response from Square"))?;
      info!("PUT Square update customer: {:?}", &res);

      Ok(res.customer)
    }
  }

  pub async fn upsert_catalog(&self) -> Result<SubscriptionPlanResponse, Error> {
    let catalog_endpoint = self.base_url.clone() + "catalog/object";

    let request = CatalogBuilder {
      id: "#plan".to_string(),
      name: self.subscription_name.clone(),
      price: self.subscription_price.clone(),
    };
    
    // upsert catalog
    let catalog_res = self.client.post(catalog_endpoint.clone())
      .header("Square-Version", self.version.clone())
      .bearer_auth(self.token.clone())
      .json(&CatalogRequest::new(request.clone()).to_value()?)
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST catalog upsert to Square"))?;
    debug!("POST Square upsert catalog: {:?}", &catalog_res);
    let catalog = catalog_res.json::<CatalogResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse catalog upsert response from Square"))?;
    info!("Square upsert catalog: {:?}", &catalog);
    info!("Catalog ID: {}", &catalog.catalog_object.id);

    // create monthly subscription plan within catalog
    let subscription_res = self.client.post(catalog_endpoint)
      .header("Square-Version", self.version.clone())
      .bearer_auth(self.token.clone())
      .json(&catalog.subscription_plan(request).to_value()?)
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST catalog subscription plan to Square"))?;
    debug!("POST Square catalog subscription plan: {:?}", &subscription_res);
    let subscription_plan = subscription_res.json::<SubscriptionPlanResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse catalog subscription plan response from Square"))?;
    info!("Square catalog subscription plan: {:?}", &subscription_plan);

    Ok(subscription_plan)
  }

  async fn get_catalog(&self) -> Result<CatalogResponseObject, Error> {
    let list_catalogs_endpoint = self.base_url.clone() + "catalog/list?types=SUBSCRIPTION_PLAN";

    let catalog_list_res = self.client.get(list_catalogs_endpoint)
      .header("Square-Version", self.version.clone())
      .bearer_auth(self.token.clone())
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to GET catalog list from Square")).unwrap();
    let catalog_list = catalog_list_res.json::<CatalogListResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse catalog subscription plan response from Square"))?;
    debug!("Square catalog list: {:?}", &catalog_list);

    let catalog = catalog_list.objects.into_iter().find(|plan| plan.id == self.catalog_id).unwrap();
    info!("Square catalog: {:?}", &catalog);
    Ok(catalog)
  }

  async fn get_location(&self) -> Result<LocationResponse, Error> {
    let location_endpoint = self.base_url.clone() + "locations";

    let res = self.client.get(location_endpoint)
      .header("Square-Version", self.version.clone())
      .bearer_auth(self.token.clone())
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to GET location from Square")).unwrap();
    let location_list = res.json::<LocationListResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse location response from Square"))?;
    debug!("Square location: {:?}", &location_list);
    // todo: error handle
    let location = location_list.locations.into_iter().find(|location| location.id == self.location_id).unwrap();
    Ok(location)
  }

  pub async fn list_subscriptions(&self) -> Result<Vec<SubscriptionResponse>, Error> {
    let list_subs_endpoint = self.base_url.clone() + "subscriptions/search";
    let list_res = self.client.post(list_subs_endpoint)
      .header("Square-Version", self.version.clone())
      .bearer_auth(self.token.clone())
      .header("Content-Type", "application/json")
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST subscription search to Square")).unwrap();
    let list = list_res.json::<SubscriptionSearchResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse POST subscription search response from Square")).unwrap();
    debug!("Square subscription list: {:?}", &list);

    let mut subscriptions = Vec::<SubscriptionResponse>::new();
    for sub in list.subscriptions.into_iter() {
      let retrieve_endpoint = self.base_url.clone() + "subscriptions/" + &*sub.id + "?include=actions";
      info!("Sub ID: {}", &sub.id);

      let res = self.client.get(retrieve_endpoint)
        .header("Square-Version", self.version.clone())
        .bearer_auth(self.token.clone())
        .header("Content-Type", "application/json")
        .send()
        .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to GET subscription from Square")).unwrap();
      let sub = res.json::<SubscriptionResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse GET retrieve subscription response from Square")).unwrap();
      debug!("Square subscription: {:?}", &sub);
      subscriptions.push(sub);
    }

    Ok(subscriptions)
  }

  pub async fn subscribe(&self) -> Result<serde_json::Value, Error> {
    let checkout_endpoint = self.base_url.clone() + "online-checkout/payment-links";
    let subscription_plan_id = self.get_catalog().await?.subscription_plan_data
      .subscription_plan_variations.unwrap()
      .get(0).unwrap()
      .id.clone();
    let location_id = self.get_location().await?.id;

    let res = self.client.post(checkout_endpoint)
      .header("Square-Version", self.version.clone())
      .bearer_auth(self.token.clone())
      .header("Content-Type", "application/json")
      .json(&CheckoutRequest::new(CheckoutBuilder {
        name: "Premium".to_string(),
        price: self.subscription_price.clone(),
        location_id,
        subscription_plan_id,
      }))
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST subscription checkout to Square"))?;
    let checkout = res.json::<serde_json::Value>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse POST subscription checkout response from Square"))?;

    Ok(checkout)
  }
}


