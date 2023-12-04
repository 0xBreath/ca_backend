use crate::*;
use actix_web::Error;
use log::*;
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;

pub struct SquareClient {
    pub client: Client,
    pub base_url: String,
    pub token: String,
    pub version: String,
    pub app_id: String,
    pub location_id: String,
    pub subscription_catalog_id: String,
    pub subscription_price: u64,
    pub subscription_name: String,
    pub redirect_url: String,
}

impl SquareClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: std::env::var("SQUARE_API_URL")
                .unwrap_or_else(|_| "https://connect.squareupsandbox.com/v2/".to_string()),
            token: std::env::var("SQUARE_ACCESS_TOKEN").unwrap_or_else(|_| "".to_string()),
            version: std::env::var("SQUARE_API_VERSION")
                .unwrap_or_else(|_| "2023-10-18".to_string()),
            app_id: std::env::var("SQUARE_APP_ID").unwrap_or_else(|_| "".to_string()),
            location_id: std::env::var("SQUARE_LOCATION_ID").unwrap_or_else(|_| "".to_string()),
            subscription_catalog_id: std::env::var("SQUARE_SUBSCRIPTION_CATALOG_ID")
                .unwrap_or_else(|_| "".to_string()),
            subscription_price: std::env::var("SQUARE_SUBSCRIPTION_PRICE")
                .unwrap_or_else(|_| "".to_string())
                .parse::<u64>()
                .unwrap(),
            subscription_name: std::env::var("SQUARE_SUBSCRIPTION_NAME")
                .unwrap_or_else(|_| "".to_string()),
            redirect_url: std::env::var("SQUARE_REDIRECT_URL").unwrap_or_else(|_| "".to_string()),
        }
    }

    async fn handle_response<T: DeserializeOwned>(
        &self,
        res: reqwest::Response,
        error_msg: &str,
    ) -> Result<SquareResponse<T>, Error> {
        match res.status() {
            StatusCode::OK => {
                let res: T = res.json::<T>().await.map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("{}: {:?}", error_msg, e))
                })?;
                Ok(SquareResponse::Success(res))
            }
            _ => {
                error!("Unhandled status code: {:?}", res.status());
                let error = res.json::<SquareErrorResponse>().await.map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("{}: {:?}", error_msg, e))
                })?;
                Ok(SquareResponse::Error(error))
            }
        }
    }

    pub async fn get_customer(
        &self,
        request: UserEmailRequest,
    ) -> Result<Option<CustomerResponse>, Error> {
        if request.email.is_none() {
            return Ok(None);
        }
        let search_customer_endpoint = self.base_url.clone() + "v2/customers/search";
        let query = SearchCustomerRequest::new(request.clone().email.unwrap()).to_value()?;
        let search_res = self
            .client
            .post(search_customer_endpoint)
            .header("Square-Version", self.version.clone())
            .bearer_auth(self.token.clone())
            .json(&query)
            .send()
            .await
            .map_err(|_| {
                actix_web::error::ErrorBadRequest("Failed to send POST customer search to Square")
            })?;
        debug!("get_customer search response: {:?}", &search_res.status());
        match self
            .handle_response::<SearchCustomerResponse>(
                search_res,
                "Failed to parse SearchCustomerResponse from Square in get_customer",
            )
            .await
        {
            Ok(customer_search) => match customer_search {
                SquareResponse::Error(error) => {
                    error!("Failed to get customer: {:?}", &error);
                    Ok(None)
                }
                SquareResponse::Success(res) => {
                    if res.customers.is_empty() {
                        Ok(None)
                    } else {
                        let customer = res.customers[0].clone();
                        Ok(Some(customer))
                    }
                }
            },
            Err(e) => {
                error!("{:?}", &e);
                Ok(None)
            }
        }
    }

    pub async fn get_customer_info(
        &self,
        request: UserEmailRequest,
    ) -> Result<Option<CustomerInfo>, Error> {
        let customer = self.get_customer(request).await?;
        match customer {
            None => Ok(None),
            Some(customer) => {
                let info = CustomerInfo {
                    email_address: customer.email_address,
                    family_name: customer.family_name,
                    given_name: customer.given_name,
                    cards: customer.cards.map(|cards| {
                        cards
                            .into_iter()
                            .map(|card| CardInfo {
                                card_brand: card.card_brand,
                                last_4: card.last_4,
                                exp_month: card.exp_month,
                                exp_year: card.exp_year,
                                cardholder_name: card.cardholder_name,
                            })
                            .collect()
                    }),
                };
                Ok(Some(info))
            }
        }
    }

    #[allow(dead_code)]
    pub async fn update_customer(
        &self,
        request: CustomerRequest,
    ) -> Result<SquareResponse<CustomerResponse>, Error> {
        // POST customer search
        let search_customer_endpoint = self.base_url.clone() + "v2/customers/search";
        let query = SearchCustomerRequest::new(request.email_address.clone()).to_value()?;
        let search_res = self
            .client
            .post(search_customer_endpoint)
            .header("Square-Version", self.version.clone())
            .bearer_auth(self.token.clone())
            .json(&query)
            .send()
            .await
            .map_err(|_| {
                actix_web::error::ErrorBadRequest("Failed to send POST customer search to Square")
            })?;
        debug!(
            "update_customer search response: {:?}",
            &search_res.status()
        );
        let customer_search = self
            .handle_response::<SearchCustomerResponse>(
                search_res,
                "Failed to parse SearchCustomerResponse from Square",
            )
            .await?;

        match customer_search {
            SquareResponse::Error(error) => Ok(SquareResponse::Error(error)),
            SquareResponse::Success(res) => {
                if res.customers.is_empty() {
                    // create new customer -> POST
                    let create_customer_endpoint = self.base_url.clone() + "v2/customers";
                    let res = self
                        .client
                        .post(create_customer_endpoint)
                        .header("Square-Version", self.version.clone())
                        .bearer_auth(self.token.clone())
                        .header("Content-Type", "application/json")
                        .json(&request)
                        .send()
                        .await
                        .map_err(|_| {
                            actix_web::error::ErrorBadRequest(
                                "Failed to send POST create customer to Square",
                            )
                        })?;

                    self
                        .handle_response::<CustomerResponse>(
                            res,
                            "Failed to parse POST create customer response from Square",
                        )
                        .await
                } else {
                    // update existing customer to subscribe -> PUT
                    let customer_id = res.customers[0].id.clone();
                    let update_customer_endpoint =
                        format!("{}v2/customers/{}", self.base_url.clone(), customer_id);

                    let res = self
                        .client
                        .put(update_customer_endpoint)
                        .header("Square-Version", self.version.clone())
                        .bearer_auth(self.token.clone())
                        .header("Content-Type", "application/json")
                        .json(&request)
                        .send()
                        .await
                        .map_err(|_| {
                            actix_web::error::ErrorBadRequest(
                                "Failed to send PUT customer update to Square",
                            )
                        })?;
                    info!("PUT Square update customer: {:?}", &res);
                    match self
                        .handle_response::<UpdateCustomerResponse>(
                            res,
                            "Failed to parse PUT customer update response from Square",
                        )
                        .await?
                    {
                        SquareResponse::Error(error) => Ok(SquareResponse::Error(error)),
                        SquareResponse::Success(res) => Ok(SquareResponse::Success(res.customer)),
                    }
                }
            }
        }
    }

    /// After creating a catalog, use `result.catalog_object.id`
    ///
    /// or use `result.catalog_object.subscription_plan_variation_data.subscription_plan_id`
    ///
    /// to set as the SQUARE_SUBSCRIPTION_CATALOG_ID in the env
    pub async fn upsert_subscription_catalog(
        &self,
    ) -> Result<SquareResponse<SubscriptionPlanResponse>, Error> {
        let catalog_endpoint = self.base_url.clone() + "v2/catalog/object";

        let request = SubscriptionCatalogBuilder {
            id: "#plan".to_string(),
            name: self.subscription_name.clone(),
            price: self.subscription_price,
        };

        // upsert catalog
        let catalog_res = self
            .client
            .post(catalog_endpoint.clone())
            .header("Square-Version", self.version.clone())
            .bearer_auth(self.token.clone())
            .json(&CatalogRequest::new_subscription_catalog(request.clone()).to_value()?)
            .send()
            .await
            .map_err(|_| {
                actix_web::error::ErrorBadRequest(
                    "Failed to send POST subscription catalog upsert to Square",
                )
            })?;
        debug!("POST Square upsert catalog: {:?}", &catalog_res.status());

        match self
            .handle_response::<CatalogResponse>(
                catalog_res,
                "Failed to parse catalog upsert response from Square",
            )
            .await?
        {
            SquareResponse::Error(error) => Ok(SquareResponse::Error(error)),
            SquareResponse::Success(catalog) => {
                debug!("Square upsert catalog: {:?}", &catalog);
                info!("Subscription catalog ID: {}", &catalog.catalog_object.id);

                // create monthly subscription plan within catalog
                let subscription_res = self
                    .client
                    .post(catalog_endpoint)
                    .header("Square-Version", self.version.clone())
                    .bearer_auth(self.token.clone())
                    .json(&catalog.subscription_plan(request).to_value()?)
                    .send()
                    .await
                    .map_err(|_| {
                        actix_web::error::ErrorBadRequest(
                            "Failed to send POST catalog subscription plan to Square",
                        )
                    })?;
                self.handle_response::<SubscriptionPlanResponse>(
                    subscription_res,
                    "Failed to parse catalog subscription plan response from Square",
                )
                .await
            }
        }
    }

    pub async fn list_catalogs(&self) -> Result<SquareResponse<CatalogListResponse>, Error> {
        let list_catalogs_endpoint =
            self.base_url.clone() + "v2/catalog/list?types=SUBSCRIPTION_PLAN";

        let catalog_list_res = self
            .client
            .get(list_catalogs_endpoint)
            .header("Square-Version", self.version.clone())
            .bearer_auth(self.token.clone())
            .send()
            .await
            .map_err(|_| {
                actix_web::error::ErrorBadRequest("Failed to GET catalog list from Square")
            })
            .unwrap();
        self.handle_response::<CatalogListResponse>(
            catalog_list_res,
            "Failed to parse catalog subscription plan response from Square",
        )
        .await
    }

    async fn get_subscription_catalog(
        &self,
    ) -> Result<SquareResponse<CatalogResponseObject>, Error> {
        let list_catalogs_endpoint =
            self.base_url.clone() + "v2/catalog/list?types=SUBSCRIPTION_PLAN";

        let catalog_list_res = self
            .client
            .get(list_catalogs_endpoint)
            .header("Square-Version", self.version.clone())
            .bearer_auth(self.token.clone())
            .send()
            .await
            .map_err(|_| {
                actix_web::error::ErrorBadRequest("Failed to GET catalog list from Square")
            })
            .unwrap();

        match self
            .handle_response::<CatalogListResponse>(
                catalog_list_res,
                "Failed to parse catalog subscription plan response from Square",
            )
            .await?
        {
            SquareResponse::Error(error) => Ok(SquareResponse::Error(error)),
            SquareResponse::Success(catalog_list) => {
                // catalog.id == catalog.catalog_object.subscription_plan_variation_data.subscription_plan_id
                // which should match SQUARE_SUBSCRIPTION_CATALOG_ID in env
                let catalog = catalog_list
                    .objects
                    .into_iter()
                    .find(|plan| plan.id == self.subscription_catalog_id)
                    .unwrap();
                Ok(SquareResponse::Success(catalog))
            }
        }
    }

    async fn get_location(&self) -> Result<SquareResponse<LocationResponse>, Error> {
        let location_endpoint = self.base_url.clone() + "v2/locations";

        let res = self
            .client
            .get(location_endpoint)
            .header("Square-Version", self.version.clone())
            .bearer_auth(self.token.clone())
            .send()
            .await
            .map_err(|_| actix_web::error::ErrorBadRequest("Failed to GET location from Square"))
            .unwrap();

        match self
            .handle_response::<LocationListResponse>(
                res,
                "Failed to parse location response from Square",
            )
            .await?
        {
            SquareResponse::Error(error) => Ok(SquareResponse::Error(error)),
            SquareResponse::Success(location_list) => {
                let location = location_list
                    .locations
                    .into_iter()
                    .find(|location| location.id == self.location_id)
                    .unwrap();
                Ok(SquareResponse::Success(location))
            }
        }
    }

    async fn get_subscription_info(&self) -> Result<Option<SubscriptionInfo>, Error> {
        match self.get_subscription_catalog().await? {
            SquareResponse::Error(error) => Ok(None),
            SquareResponse::Success(catalog) => match catalog.subscription_plan_data {
                None => Ok(None),
                Some(data) => {
                    let cost = match data.subscription_plan_variations {
                        None => None,
                        Some(variations) => match variations.get(0) {
                            None => None,
                            Some(variation) => {
                                match variation.subscription_plan_variation_data.phases.get(0) {
                                    None => None,
                                    Some(phase) => match &phase.pricing.price_money {
                                        None => match &phase.pricing.price {
                                            None => None,
                                            Some(price) => {
                                                let price = price.amount as f64 / 100.0;
                                                Option::from(price)
                                            }
                                        },
                                        Some(price) => {
                                            let price = price.amount as f64 / 100.0;
                                            Option::from(price)
                                        }
                                    },
                                }
                            }
                        },
                    };
                    match cost {
                        Some(cost) => {
                            let title = data.name;
                            Ok(Some(SubscriptionInfo { title, cost }))
                        }
                        None => Ok(None),
                    }
                }
            },
        }
    }

    async fn get_user_subscription_info(
        &self,
        request: UserEmailRequest,
    ) -> Result<Option<UserSubscriptionInfo>, Error> {
        let subscription = self.get_subscription(request).await?;
        match subscription {
            Some(sub) => Ok(Some(UserSubscriptionInfo {
                start_date: sub.start_date,
                charged_through_date: sub.charged_through_date,
                canceled_date: sub.canceled_date,
            })),
            None => Ok(None),
        }
    }

    pub async fn list_subscriptions(&self) -> Result<Vec<SubscriptionResponse>, Error> {
        let list_subs_endpoint = self.base_url.clone() + "v2/subscriptions/search";
        let list_res = self
            .client
            .post(list_subs_endpoint)
            .header("Square-Version", self.version.clone())
            .bearer_auth(self.token.clone())
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|_| {
                actix_web::error::ErrorBadRequest(
                    "Failed to send POST subscription search to Square",
                )
            })
            .unwrap();

        let list = self
            .handle_response::<SubscriptionSearchResponse>(
                list_res,
                "Failed to parse POST subscription search response from Square",
            )
            .await?;

        let mut subs = Vec::<SubscriptionResponse>::new();
        if let SquareResponse::Success(list) = list {
            for sub in list.subscriptions.into_iter() {
                let retrieve_endpoint =
                    self.base_url.clone() + "v2/subscriptions/" + &*sub.id + "?include=actions";
                debug!("Subscription ID: {}", &sub.id);

                let res = self
                    .client
                    .get(retrieve_endpoint)
                    .header("Square-Version", self.version.clone())
                    .bearer_auth(self.token.clone())
                    .header("Content-Type", "application/json")
                    .send()
                    .await
                    .map_err(|_| {
                        actix_web::error::ErrorBadRequest("Failed to GET subscription from Square")
                    })
                    .unwrap();
                let sub = self
                    .handle_response::<SubscriptionResponse>(
                        res,
                        "Failed to parse GET retrieve subscription response from Square",
                    )
                    .await?;
                if let SquareResponse::Success(sub) = sub {
                    subs.push(sub);
                }
            }
        }
        Ok(subs)
    }

    pub async fn get_subscription(
        &self,
        request: UserEmailRequest,
    ) -> Result<Option<SubscriptionResponseObject>, Error> {
        // get customer from email
        let customer: Option<CustomerResponse> = self.get_customer(request).await?;
        match customer {
            None => Ok(None),
            Some(customer) => {
                let customer_id = customer.id;
                // search subscription by customer_id
                let list_subs_endpoint = self.base_url.clone() + "v2/subscriptions/search";
                let list_res = self
                    .client
                    .post(list_subs_endpoint)
                    .header("Square-Version", self.version.clone())
                    .bearer_auth(self.token.clone())
                    .header("Content-Type", "application/json")
                    .json(&SearchSubscriptionsRequest::new(customer_id).to_value()?)
                    .send()
                    .await
                    .map_err(|_| {
                        actix_web::error::ErrorBadRequest(
                            "Failed to send POST subscription search to Square",
                        )
                    })
                    .unwrap();

                match self
                    .handle_response::<SubscriptionSearchResponse>(
                        list_res,
                        "Failed to parse POST subscription search response from Square",
                    )
                    .await?
                {
                    SquareResponse::Error(_) => Ok(None),
                    SquareResponse::Success(list) => {
                        if list.subscriptions.is_empty() {
                            Ok(None)
                        } else {
                            let sub: SubscriptionResponseObject = list.subscriptions[0].clone();
                            Ok(Some(sub))
                        }
                    }
                }
            }
        }
    }

    pub async fn get_user_profile(&self, request: UserEmailRequest) -> Result<UserProfile, Error> {
        let customer = match self.get_customer_info(request.clone()).await {
            Ok(Some(customer)) => Some(customer),
            Ok(None) => None,
            Err(e) => {
                error!("Failed to get customer info: {:?}", &e);
                None
            }
        };
        let sub_info = self.get_subscription_info().await?;
        let user_sub_info = self.get_user_subscription_info(request).await?;
        debug!("get_user_profile, customer?: {:?}", &customer.is_some());
        debug!("get_user_profile, sub_info?: {:?}", &sub_info.is_some());
        debug!(
            "get_user_profile, user_sub_info?: {:?}",
            &user_sub_info.is_some()
        );
        Ok(UserProfile {
            customer,
            subscription_info: sub_info,
            user_subscription: user_sub_info,
        })
    }

    pub async fn subscribe_checkout(
        &self,
        user_email: UserEmailRequest,
    ) -> Result<SquareResponse<CheckoutInfo>, Error> {
        let checkout_endpoint = self.base_url.clone() + "v2/online-checkout/payment-links";
        match self.get_subscription_catalog().await? {
            SquareResponse::Error(error) => Ok(SquareResponse::Error(error)),
            SquareResponse::Success(subscription_plan) => {
                let subscription_plan_id = subscription_plan
                    .subscription_plan_data
                    .unwrap()
                    .subscription_plan_variations
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .id
                    .clone();

                match self.get_location().await? {
                    SquareResponse::Error(error) => Ok(SquareResponse::Error(error)),
                    SquareResponse::Success(location) => {
                        let location_id = location.id;
                        let res = self
                            .client
                            .post(checkout_endpoint)
                            .header("Square-Version", self.version.clone())
                            .bearer_auth(self.token.clone())
                            .header("Content-Type", "application/json")
                            .json(&CheckoutRequest::new_subscription(
                                SubscriptionCheckoutBuilder {
                                    name: self.subscription_name.to_string(),
                                    price: self.subscription_price,
                                    location_id,
                                    subscription_plan_id,
                                    redirect_url: self.redirect_url.clone(),
                                    buyer_email: user_email.email,
                                },
                            ))
                            .send()
                            .await
                            .map_err(|_| {
                                actix_web::error::ErrorBadRequest(
                                    "Failed to send POST subscription checkout to Square",
                                )
                            })?;
                        match self
                            .handle_response::<CheckoutResponse>(
                                res,
                                "Failed to parse POST subscription checkout response from Square",
                            )
                            .await?
                        {
                            SquareResponse::Error(error) => Ok(SquareResponse::Error(error)),
                            SquareResponse::Success(checkout) => {
                                let checkout_info = CheckoutInfo {
                                    url: checkout.payment_link.url,
                                    amount: checkout
                                        .related_resources
                                        .orders
                                        .get(0)
                                        .unwrap()
                                        .net_amount_due_money
                                        .amount as f64
                                        / 100.0,
                                };
                                debug!("Square subscription checkout info: {:?}", &checkout_info);

                                Ok(SquareResponse::Success(checkout_info))
                            }
                        }
                    }
                }
            }
        }
    }

    /// Provides customer name and card, but not email
    pub async fn list_customers(&self) -> Result<SquareResponse<CustomerListResponse>, Error> {
        let endpoint =
            self.base_url.clone() + "v2/customers?limit=10&sort_field=CREATED_AT&sort_order=DESC";
        let res = self
            .client
            .get(endpoint)
            .header("Square-Version", self.version.clone())
            .bearer_auth(self.token.clone())
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|_| actix_web::error::ErrorBadRequest("Failed to GET customers from Square"))
            .unwrap();

        self.handle_response::<CustomerListResponse>(
            res,
            "Failed to parse GET customers response from Square",
        )
        .await
    }

    pub async fn list_orders(&self) -> Result<SquareResponse<SearchOrdersResponse>, Error> {
        let customers_list = self.list_customers().await?;

        match customers_list {
            SquareResponse::Error(error) => Ok(SquareResponse::Error(error)),
            SquareResponse::Success(customers_list) => {
                let customer_ids = customers_list
                    .customers
                    .into_iter()
                    .map(|customer| customer.id)
                    .collect::<Vec<String>>();

                let builder = SearchOrdersRequestBuilder {
                    location_ids: vec![self.location_id.clone()],
                    customer_ids: Some(customer_ids),
                };

                let endpoint = self.base_url.clone() + "v2/orders/search";
                let res = self
                    .client
                    .post(endpoint)
                    .header("Square-Version", self.version.clone())
                    .bearer_auth(self.token.clone())
                    .header("Content-Type", "application/json")
                    .json(&SearchOrdersRequest::new(builder))
                    .send()
                    .await
                    .map_err(|_| {
                        actix_web::error::ErrorBadRequest("Failed to GET invoices from Square")
                    })
                    .unwrap();

                self.handle_response::<SearchOrdersResponse>(
                    res,
                    "Failed to parse POST search orders response from Square",
                )
                .await
            }
        }
    }

    pub async fn list_invoices(&self) -> Result<SquareResponse<InvoiceListResponse>, Error> {
        let endpoint = self.base_url.clone() + "v2/invoices?location_id=" + &*self.location_id;
        let res = self
            .client
            .get(endpoint)
            .header("Square-Version", self.version.clone())
            .bearer_auth(self.token.clone())
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|_| actix_web::error::ErrorBadRequest("Failed to GET invoices from Square"))
            .unwrap();

        self.handle_response::<InvoiceListResponse>(
            res,
            "Failed to parse GET invoices response from Square",
        )
        .await
    }
    
    // get customer email via invoices endpoint
    pub async fn email_list(&self) -> Result<SquareResponse<Vec<CustomerEmailInfo>>, Error> {
        match self.list_invoices().await? {
            SquareResponse::Error(error) => Ok(SquareResponse::Error(error)),
            SquareResponse::Success(res) => {
                let emails: Vec<CustomerEmailInfo> = res
                    .invoices
                    .into_iter()
                    .map(|invoice| CustomerEmailInfo {
                        email_address: invoice.primary_recipient.email_address,
                        family_name: invoice.primary_recipient.family_name,
                        given_name: invoice.primary_recipient.given_name,
                    })
                    .collect();
                // filter out duplicate email_address
                let emails: Vec<CustomerEmailInfo> =
                    emails.into_iter().fold(Vec::new(), |mut acc, email| {
                        if !acc.iter().any(|e| e.email_address == email.email_address) {
                            acc.push(email);
                        }
                        acc
                    });
                Ok(SquareResponse::Success(emails))
            }
        }
    }

    pub async fn cancel_subscription(
        &self,
        request: UserEmailRequest,
    ) -> Result<SquareResponse<CanceledSubscriptionInfo>, Error> {
        let subscription_id = self.get_subscription(request.clone()).await?.unwrap().id;
        let endpoint = self.base_url.clone() + "v2/subscriptions/" + &subscription_id + "/cancel";

        let res = self
            .client
            .post(endpoint)
            .header("Square-Version", self.version.clone())
            .bearer_auth(self.token.clone())
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|_| {
                actix_web::error::ErrorBadRequest("Failed to GET customer attribute from Square")
            })
            .unwrap();

        match self
            .handle_response::<CancelSubscriptionResponse>(
                res,
                "Failed to parse cancel subscription response",
            )
            .await?
        {
            SquareResponse::Error(error) => Ok(SquareResponse::Error(error)),
            SquareResponse::Success(object) => {
                // yyyy-mm-dd
                // break apart into year, month, day
                let charged_through_year = object.subscription.charged_through_date;
                let date_parts = charged_through_year.split('-').collect::<Vec<&str>>();
                let charged_through_year = date_parts[0].parse::<u16>().unwrap();
                let charged_through_month = date_parts[1].parse::<u8>().unwrap();
                let charged_through_day = date_parts[2].parse::<u8>().unwrap();

                let info = CanceledSubscriptionInfo {
                    email: request.email.unwrap(),
                    charged_through_year,
                    charged_through_month,
                    charged_through_day,
                };

                Ok(SquareResponse::Success(info))
            }
        }
    }
}
