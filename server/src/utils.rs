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
  pub plan_id: String,
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
      plan_id: std::env::var("SQUARE_PLAN_ID").unwrap_or_else(|_| "".to_string()),
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

  pub async fn upsert_catalog(&self, request: CatalogBuilder) -> Result<SubscriptionPlanResponse, Error> {
    let catalog_endpoint = self.base_url.clone() + "catalog/object";

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

  async fn latest_catalog(&self) -> Result<CatalogResponseObject, Error> {
    let list = self.list_catalogs().await?;
    let latest_catalog = list.objects.into_iter().last().unwrap();
    Ok(latest_catalog)
  }

  async fn list_catalogs(&self) -> Result<CatalogListResponse, Error> {
    let list_catalogs_endpoint = self.base_url.clone() + "catalog/list?types=SUBSCRIPTION_PLAN";

    let catalog_list_res = self.client.get(list_catalogs_endpoint)
      .header("Square-Version", self.version.clone())
      .bearer_auth(self.token.clone())
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to GET catalog list from Square")).unwrap();
    let catalog_list = catalog_list_res.json::<CatalogListResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse catalog subscription plan response from Square"))?;
    debug!("Square catalog list: {:?}", &catalog_list);

    Ok(catalog_list)
  }

  async fn get_subscription_plan(&self) -> Result<SubscriptionPlanResponseObject, Error> {
    let list_subs_endpoint = self.base_url.clone() + "catalog/list?types=SUBSCRIPTION_PLAN_VARIATION";

    let sub_list_res = self.client.get(list_subs_endpoint)
      .header("Square-Version", self.version.clone())
      .bearer_auth(self.token.clone())
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to GET subscription plan list from Square")).unwrap();
    let sub_list = sub_list_res.json::<SubscriptionPlanListResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse catalog subscription plan response from Square"))?;
    debug!("Square subscription plan list: {:?}", &sub_list);
    let plan = sub_list.objects.into_iter().find(|plan| plan.id == self.plan_id).unwrap();

    Ok(plan)
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

  pub async fn create_order_template(&self) -> Result<OrderResponse, Error> {
    let order_endpoint = self.base_url.clone() + "orders";
    let latest_catalog = self.latest_catalog().await.unwrap();
    let latest_location = self.get_location().await.unwrap();

    let catalog_object_id = latest_catalog.id;
    let request = OrderRequest::new(OrderBuilder {
      location_id: latest_location.id,
      catalog_object_id,
    });

    let res = self.client.post(order_endpoint)
      .header("Square-Version", self.version.clone())
      .bearer_auth(self.token.clone())
      .json(&request)
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST order create to Square")).unwrap();
    let order_template = res.json::<OrderResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse POST response from Square")).unwrap();
    info!("Square order template: {:?}", &order_template);

    Ok(order_template)
  }

  // todo: check if customer is subscribed before creating subscription
  pub async fn subscribe_customer(&self, request: CustomerRequest) -> Result<SubscriptionResponse, Error> {
    let subscribe_endpoint = self.base_url.clone() + "subscriptions";
    let customer: CustomerResponse = self.update_customer(request).await?;
    let location: LocationResponse = self.get_location().await?;
    let plan: SubscriptionPlanResponseObject = self.get_subscription_plan().await?;

    let res = self.client.post(subscribe_endpoint)
      .header("Square-Version", self.version.clone())
      .bearer_auth(self.token.clone())
      .json(&SubscriptionRequest::new(customer.id, location.id, plan.id))
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST subscription to Square")).unwrap();
    let subscription = res.json::<SubscriptionResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse POST subscription response from Square")).unwrap();
    debug!("Square subscription: {:?}", &subscription);

    Ok(subscription)
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
}


