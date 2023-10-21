use actix_web::{Error};
use crate::*;
use log::*;
use reqwest::Client;

pub struct SquareClient {
  pub client: Client,
  pub base_url: String,
  pub token: String,
  pub version: String,
  pub app_id: String,
  pub location_id: String,
  pub catalog_id: String,
  pub subscription_price: u64,
  pub subscription_name: String,
  pub redirect_url: String,
}

impl SquareClient {
  pub fn new() -> Self {
    Self {
      client: Client::new(),
      base_url: std::env::var("SQUARE_API_URL").unwrap_or_else(|_| "https://connect.squareupsandbox.com/v2/".to_string()),
      token: std::env::var("SQUARE_ACCESS_TOKEN").unwrap_or_else(|_| "".to_string()),
      version: std::env::var("SQUARE_API_VERSION").unwrap_or_else(|_| "2023-10-18".to_string()),
      app_id: std::env::var("SQUARE_APP_ID").unwrap_or_else(|_| "".to_string()),
      location_id: std::env::var("SQUARE_LOCATION_ID").unwrap_or_else(|_| "".to_string()),
      catalog_id: std::env::var("SQUARE_CATALOG_ID").unwrap_or_else(|_| "".to_string()),
      subscription_price: std::env::var("SQUARE_SUBSCRIPTION_PRICE").unwrap_or_else(|_| "".to_string()).parse::<u64>().unwrap(),
      subscription_name: std::env::var("SQUARE_SUBSCRIPTION_NAME").unwrap_or_else(|_| "".to_string()),
      redirect_url: std::env::var("SQUARE_REDIRECT_URL").unwrap_or_else(|_| "".to_string()),
    }
  }

  pub async fn update_customer(&self, request: CustomerRequest) -> Result<CustomerResponse, Error> {
    // POST customer search
    let search_customer_endpoint = self.base_url.clone() + "v2/customers/search";
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
      let create_customer_endpoint = self.base_url.clone() + "v2/customers";
      let res = self.client.post(create_customer_endpoint)
        .header("Square-Version", self.version.clone())
        .bearer_auth(self.token.clone())
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST customer create to Square"))?;
      let res = res.json::<CustomerResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse POST response from Square"))?;
      debug!("POST Square create customer: {:?}", &res);

      Ok(res)
    } else {
      // update existing customer to subscribe -> PUT
      let customer_id = customer_search.customers[0].id.clone();
      let update_customer_endpoint = format!("{}v2/customers/{}", self.base_url.clone(), customer_id);
      info!("update customer endpoint: {}", update_customer_endpoint);

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

  /// After creating a catalog, use the result.catalog_object.subscription_plan_variation_data.subscription_plan_id`
  /// to set as the SQUARE_CATALOG_ID in the env
  pub async fn upsert_catalog(&self) -> Result<SubscriptionPlanResponse, Error> {
    let catalog_endpoint = self.base_url.clone() + "v2/catalog/object";

    let request = CatalogBuilder {
      id: "#plan".to_string(),
      name: self.subscription_name.clone(),
      price: self.subscription_price,
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
    debug!("Square catalog subscription plan: {:?}", &subscription_plan);

    Ok(subscription_plan)
  }

  async fn get_catalog(&self) -> Result<CatalogResponseObject, Error> {
    let list_catalogs_endpoint = self.base_url.clone() + "v2/catalog/list?types=SUBSCRIPTION_PLAN";

    let catalog_list_res = self.client.get(list_catalogs_endpoint)
      .header("Square-Version", self.version.clone())
      .bearer_auth(self.token.clone())
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to GET catalog list from Square")).unwrap();
    let catalog_list = catalog_list_res.json::<CatalogListResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse catalog subscription plan response from Square"))?;
    debug!("Square catalog list: {:?}", &catalog_list);

    // this is SUBSCRIPTION_PLAN catalog.catalog_object.subscription_plan_variation_data.subscription_plan_id
    // which should match SQUARE_CATALOG_ID in env
    let catalog = catalog_list.objects.into_iter().find(|plan| plan.id == self.catalog_id).unwrap();
    debug!("Square catalog: {:?}", &catalog);
    Ok(catalog)
  }

  async fn get_location(&self) -> Result<LocationResponse, Error> {
    let location_endpoint = self.base_url.clone() + "v2/locations";

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
    let list_subs_endpoint = self.base_url.clone() + "v2/subscriptions/search";
    let list_res = self.client.post(list_subs_endpoint)
      .header("Square-Version", self.version.clone())
      .bearer_auth(self.token.clone())
      .header("Content-Type", "application/json")
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST subscription search to Square")).unwrap();
    let list = list_res.json::<SubscriptionSearchResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse POST subscription search response from Square")).unwrap();
    debug!("Square subscription list: {:?}", &list);

    let mut subs = Vec::<SubscriptionResponse>::new();
    for sub in list.subscriptions.into_iter() {
      let retrieve_endpoint = self.base_url.clone() + "v2/subscriptions/" + &*sub.id + "?include=actions";
      info!("Sub ID: {}", &sub.id);

      let res = self.client.get(retrieve_endpoint)
        .header("Square-Version", self.version.clone())
        .bearer_auth(self.token.clone())
        .header("Content-Type", "application/json")
        .send()
        .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to GET subscription from Square")).unwrap();
      let sub = res.json::<SubscriptionResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse GET retrieve subscription response from Square")).unwrap();
      debug!("Square subscription: {:?}", &sub);
      subs.push(sub);
    }

    Ok(subs)
  }

  pub async fn subscribe(&self) -> Result<CheckoutInfo, Error> {
    let checkout_endpoint = self.base_url.clone() + "v2/online-checkout/payment-links";
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
        name: self.subscription_name.to_string(),
        price: self.subscription_price,
        location_id,
        subscription_plan_id,
        redirect_url: self.redirect_url.clone(),
      }))
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to send POST subscription checkout to Square"))?;
    let checkout = res.json::<CheckoutResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse POST subscription checkout response from Square"))?;
    info!("Square subscription checkout: {:?}", &checkout);

    let checkout_info = CheckoutInfo {
      url: checkout.payment_link.url,
      amount: checkout.related_resources.orders.get(0).unwrap().net_amount_due_money.amount,
    };
    debug!("Square subscription checkout info: {:?}", &checkout_info);

    Ok(checkout_info)
  }

  /// Provides customer name and card, but not email
  pub async fn list_customers(&self) -> Result<CustomerListResponse, Error> {
    let endpoint = self.base_url.clone() + "v2/customers?limit=10&sort_field=CREATED_AT&sort_order=DESC";
    let res = self.client.get(endpoint)
      .header("Square-Version", self.version.clone())
      .bearer_auth(self.token.clone())
      .header("Content-Type", "application/json")
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to GET customers from Square")).unwrap();
    let list = res.json::<CustomerListResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse GET customers response from Square"))?;
    debug!("Square customer list: {:?}", &list);
    Ok(list)
  }

  pub async fn list_invoices(&self) -> Result<InvoiceListResponse, Error> {
    let endpoint = self.base_url.clone() + "v2/invoices?location_id=" + &*self.location_id;
    let res = self.client.get(endpoint)
      .header("Square-Version", self.version.clone())
      .bearer_auth(self.token.clone())
      .header("Content-Type", "application/json")
      .send()
      .await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to GET invoices from Square")).unwrap();
    let list = res.json::<InvoiceListResponse>().await.map_err(|_| actix_web::error::ErrorBadRequest("Failed to parse GET invoices response from Square")).unwrap();
    Ok(list)
  }

  // get customer email via invoices endpoint
  pub async fn email_list(&self) -> Result<Vec<CustomerEmailInfo>, Error> {
    let res = self.list_invoices().await?;
    let emails: Vec<CustomerEmailInfo> = res.invoices.into_iter().map(|invoice| CustomerEmailInfo {
      email_address: invoice.primary_recipient.email_address,
      family_name: invoice.primary_recipient.family_name,
      given_name: invoice.primary_recipient.given_name,
    }).collect();
    // filter out duplicate email_address
    let emails: Vec<CustomerEmailInfo> = emails.into_iter().fold(Vec::new(), |mut acc, email| {
      if !acc.iter().any(|e| e.email_address == email.email_address) {
        acc.push(email);
      }
      acc
    });

    Ok(emails)
  }
}