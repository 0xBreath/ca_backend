use actix_web::{Error, HttpResponse};
use crate::types::*;
use log::*;
use reqwest::Client;

pub async fn update_customer(client: &Client, request: CustomerRequest) -> Result<HttpResponse, Error> {
  let base_url = std::env::var("SQUARE_API_URL").unwrap_or_else(|_| "https://connect.squareupsandbox.com/v2/".to_string());
  let token = std::env::var("SQUARE_ACCESS_TOKEN").unwrap_or_else(|_| "".to_string());
  let version = std::env::var("SQUARE_API_VERSION").unwrap_or_else(|_| "2023-09-25".to_string());

  // POST customer search
  let search_customer_endpoint = base_url.clone() + "customers/search";
  let query = SearchCustomerRequest::new(request.email_address.clone()).to_value()?;
  let search_res = client.post(search_customer_endpoint)
    .header("Square-Version", version.clone())
    .bearer_auth(token.clone())
    .json(&query)
    .send()
    .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST customer search to Square"))?;
  debug!("POST Square search customer: {:?}", &search_res);
  let customer_search = search_res.json::<SearchCustomerResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse SearchCustomerResponse from Square"))?;

  if customer_search.customers.is_empty() {
    // create new customer -> POST
    let create_customer_endpoint = base_url.clone() + "customers";
    let res = client.post(create_customer_endpoint)
      .header("Square-Version", version.clone())
      .bearer_auth(token.clone())
      .header("Content-Type", "application/json")
      .json(&request)
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST customer create to Square"))?;
    let res = res.json::<CustomerResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse POST response from Square"))?;
    info!("POST Square create customer: {:?}", &res);

    Ok(HttpResponse::Ok().json(request))
  } else {
    // update existing customer to subscribe -> PUT
    let customer_id = customer_search.customers[0].id.clone();
    let update_customer_endpoint = format!("{}customers/{}", base_url.clone(), customer_id);
    debug!("update customer endpoint: {}", update_customer_endpoint);

    let res = client.put(update_customer_endpoint)
      .header("Square-Version", version.clone())
      .bearer_auth(token.clone())
      .header("Content-Type", "application/json")
      .json(&request)
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send PUT customer update to Square"))?;
    let res = res.json::<CustomerResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse PUT response from Square"))?;
    info!("PUT Square update customer: {:?}", &res);

    Ok(HttpResponse::Ok().json(request))
  }
}

pub async fn upsert_catalog(client: &Client, request: CatalogBuilder) -> Result<HttpResponse, Error> {
  let base_url = std::env::var("SQUARE_API_URL").unwrap_or_else(|_| "https://connect.squareupsandbox.com/v2/".to_string());
  let token = std::env::var("SQUARE_ACCESS_TOKEN").unwrap_or_else(|_| "".to_string());
  let version = std::env::var("SQUARE_API_VERSION").unwrap_or_else(|_| "2023-09-25".to_string());
  let catalog_endpoint = base_url.clone() + "catalog/object";

  // upsert catalog
  let catalog_res = client.post(catalog_endpoint.clone())
    .header("Square-Version", version.clone())
    .bearer_auth(token.clone())
    .json(&CatalogRequest::new(request.clone()).to_value()?)
    .send()
    .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST catalog upsert to Square"))?;
  debug!("POST Square upsert catalog: {:?}", &catalog_res);
  let catalog = catalog_res.json::<CatalogResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse catalog upsert response from Square"))?;
  info!("Square upsert catalog: {:?}", &catalog);
  info!("Catalog ID: {}", &catalog.catalog_object.id);

  // create monthly subscription plan within catalog
  let subscription_res = client.post(catalog_endpoint)
    .header("Square-Version", version.clone())
    .bearer_auth(token.clone())
    .json(&catalog.subscription_plan(request).to_value()?)
    .send()
    .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST catalog subscription plan to Square"))?;
  debug!("POST Square catalog subscription plan: {:?}", &subscription_res);
  let subscription_plan_res = subscription_res.json::<serde_json::Value>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse catalog subscription plan response from Square"))?;
  info!("Square catalog subscription plan: {:?}", &subscription_plan_res);

  Ok(HttpResponse::Ok().json(catalog))
}

// pub async fn order_subscription(client: &Client, request: OrderBuilder) -> Result<HttpResponse, Error> {
//   let base_url = std::env::var("SQUARE_API_URL").unwrap_or_else(|_| "https://connect.squareupsandbox.com/v2/".to_string());
//   let token = std::env::var("SQUARE_ACCESS_TOKEN").unwrap_or_else(|_| "".to_string());
//   let version = std::env::var("SQUARE_API_VERSION").unwrap_or_else(|_| "2023-09-25".to_string());
//   let order_endpoint = base_url.clone() + "orders";
//
//   // create order
//   let res = client.post(order_endpoint.clone())
//     .header("Square-Version", version.clone())
//     .bearer_auth(token.clone())
//     .json(&OrderRequest::new(request.clone()).to_value()?)
//     .send()
//     .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST order to Square"))?;
//   debug!("POST Square order: {:?}", &res);
//   let order = res.json::<OrderResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse order response from Square"))?;
//   info!("Square order: {:?}", &order);
//   info!("Order ID: {}", &order.order.id);
//
//   Ok(HttpResponse::Ok().json(order))
// }