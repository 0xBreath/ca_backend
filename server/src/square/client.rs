use crate::*;
use actix_web::Error;
use log::*;
use reqwest::Client;

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
    pub coaching_catalog_id: String,
    pub coaching_catalog_item_name: String,
    pub coaching_1_session_price: String,
    pub coaching_3_session_price: String,
    pub coaching_6_session_price: String,
    pub coaching_10_session_price: String,
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
            coaching_catalog_id: std::env::var("SQUARE_COACHING_CATALOG_ID")
                .unwrap_or_else(|_| "".to_string()),
            coaching_catalog_item_name: std::env::var("SQUARE_COACHING_CATALOG_ITEM_NAME")
                .unwrap_or_else(|_| "".to_string()),
            coaching_1_session_price: std::env::var("SQUARE_COACHING_1_SESSION_PRICE")
                .unwrap_or_else(|_| "".to_string()),
            coaching_3_session_price: std::env::var("SQUARE_COACHING_3_SESSION_PRICE")
                .unwrap_or_else(|_| "".to_string()),
            coaching_6_session_price: std::env::var("SQUARE_COACHING_6_SESSION_PRICE")
                .unwrap_or_else(|_| "".to_string()),
            coaching_10_session_price: std::env::var("SQUARE_COACHING_10_SESSION_PRICE")
                .unwrap_or_else(|_| "".to_string()),
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
        let query = SearchCustomerRequest::new(request.email.unwrap()).to_value()?;
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
        debug!("POST Square search customer: {:?}", &search_res);
        let customer_search = search_res
            .json::<SearchCustomerResponse>()
            .await
            .map_err(|_| {
                actix_web::error::ErrorBadRequest(
                    "Failed to parse SearchCustomerResponse from Square",
                )
            })?;

        if customer_search.customers.is_empty() {
            Ok(None)
        } else {
            let customer = customer_search.customers[0].clone();
            Ok(Some(customer))
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
    ) -> Result<CustomerResponse, Error> {
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
        debug!("POST Square search customer: {:?}", &search_res);
        let customer_search = search_res
            .json::<SearchCustomerResponse>()
            .await
            .map_err(|_| {
                actix_web::error::ErrorBadRequest(
                    "Failed to parse SearchCustomerResponse from Square",
                )
            })?;

        if customer_search.customers.is_empty() {
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
                        "Failed to send POST customer create to Square",
                    )
                })?;
            let res = res.json::<CustomerResponse>().await.map_err(|_| {
                actix_web::error::ErrorBadRequest("Failed to parse POST response from Square")
            })?;
            debug!("POST Square create customer: {:?}", &res);

            Ok(res)
        } else {
            // update existing customer to subscribe -> PUT
            let customer_id = customer_search.customers[0].id.clone();
            let update_customer_endpoint =
                format!("{}v2/customers/{}", self.base_url.clone(), customer_id);
            debug!("Update customer endpoint: {}", update_customer_endpoint);

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
            let res = res.json::<UpdateCustomerResponse>().await.map_err(|_| {
                actix_web::error::ErrorBadRequest("Failed to parse PUT response from Square")
            })?;
            info!("PUT Square update customer: {:?}", &res);

            Ok(res.customer)
        }
    }

    /// After creating a catalog, use `result.catalog_object.id`
    ///
    /// or use `result.catalog_object.subscription_plan_variation_data.subscription_plan_id`
    ///
    /// to set as the SQUARE_SUBSCRIPTION_CATALOG_ID in the env
    pub async fn upsert_subscription_catalog(&self) -> Result<SubscriptionPlanResponse, Error> {
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
        debug!("POST Square upsert catalog: {:?}", &catalog_res);
        let catalog = catalog_res.json::<CatalogResponse>().await.map_err(|_| {
            actix_web::error::ErrorBadRequest("Failed to parse catalog upsert response from Square")
        })?;
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
        debug!(
            "POST Square catalog subscription plan: {:?}",
            &subscription_res
        );
        let subscription_plan = subscription_res
            .json::<SubscriptionPlanResponse>()
            .await
            .map_err(|_| {
                actix_web::error::ErrorBadRequest(
                    "Failed to parse catalog subscription plan response from Square",
                )
            })?;
        debug!("Square catalog subscription plan: {:?}", &subscription_plan);

        Ok(subscription_plan)
    }

    pub async fn list_catalogs(&self) -> Result<CatalogListResponse, Error> {
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
        let catalog_list = catalog_list_res
            .json::<CatalogListResponse>()
            .await
            .map_err(|_| {
                actix_web::error::ErrorBadRequest(
                    "Failed to parse catalog subscription plan response from Square",
                )
            })?;
        debug!("Square catalog list: {:?}", &catalog_list);

        Ok(catalog_list)
    }

    /// After creating a catalog, use `result.catalog_object.id`
    ///
    /// or use `result.catalog_object.subscription_plan_variation_data.subscription_plan_id`
    ///
    /// to set as the SQUARE_SUBSCRIPTION_CATALOG_ID in the env
    pub async fn upsert_coaching_catalog(&self) -> Result<CatalogResponse, Error> {
        let catalog_endpoint = self.base_url.clone() + "v2/catalog/object";

        let request = CoachingCatalogBuilder {
            id: "#coaching".to_string(),
            name: self.coaching_catalog_item_name.clone(),
            single_session_price: self.coaching_1_session_price.parse::<u64>().unwrap(),
            three_session_price: self.coaching_3_session_price.parse::<u64>().unwrap(),
            six_session_price: self.coaching_6_session_price.parse::<u64>().unwrap(),
            ten_session_price: self.coaching_10_session_price.parse::<u64>().unwrap(),
        };

        // upsert catalog
        let catalog_res = self
            .client
            .post(catalog_endpoint.clone())
            .header("Square-Version", self.version.clone())
            .bearer_auth(self.token.clone())
            .json(&CatalogRequest::new_coaching_catalog(request.clone()).to_value()?)
            .send()
            .await
            .map_err(|_| {
                actix_web::error::ErrorBadRequest(
                    "Failed to send POST coaching catalog upsert to Square",
                )
            })?;
        debug!("POST Square upsert catalog: {:?}", &catalog_res);
        let catalog = catalog_res.json::<CatalogResponse>().await.map_err(|_| {
            actix_web::error::ErrorBadRequest(
                "Failed to parse coaching catalog upsert response from Square",
            )
        })?;
        debug!("Square upsert catalog: {:?}", &catalog);
        info!("Coaching catalog ID: {}", &catalog.catalog_object.id);

        Ok(catalog)
    }

    async fn get_subscription_catalog(&self) -> Result<CatalogResponseObject, Error> {
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
        let catalog_list = catalog_list_res
            .json::<CatalogListResponse>()
            .await
            .map_err(|_| {
                actix_web::error::ErrorBadRequest(
                    "Failed to parse catalog subscription plan response from Square",
                )
            })?;
        debug!("Square catalog list: {:?}", &catalog_list);

        // catalog.id == catalog.catalog_object.subscription_plan_variation_data.subscription_plan_id
        // which should match SQUARE_SUBSCRIPTION_CATALOG_ID in env
        let catalog = catalog_list
            .objects
            .into_iter()
            .find(|plan| plan.id == self.subscription_catalog_id)
            .unwrap();
        debug!("Square catalog: {:?}", &catalog);
        Ok(catalog)
    }

    pub async fn get_coaching_catalog(&self) -> Result<CatalogResponseObject, Error> {
        let list_catalogs_endpoint = self.base_url.clone() + "v2/catalog/list?types=ITEM";

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

        // todo: fix parsing
        let catalog_list = catalog_list_res
            .json::<CatalogListResponse>()
            .await
            .map_err(|_| {
                actix_web::error::ErrorBadRequest(
                    "Failed to parse coaching catalog response from Square",
                )
            })?;
        debug!("Coaching catalog list: {:?}", &catalog_list);

        let catalog = catalog_list
            .objects
            .into_iter()
            .find(|plan| plan.id == self.coaching_catalog_id)
            .unwrap();
        debug!("Coaching catalog: {:?}", &catalog);
        Ok(catalog)
    }

    async fn get_location(&self) -> Result<LocationResponse, Error> {
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
        let location_list = res.json::<LocationListResponse>().await.map_err(|_| {
            actix_web::error::ErrorBadRequest("Failed to parse location response from Square")
        })?;
        debug!("Square location: {:?}", &location_list);
        // todo: error handle
        let location = location_list
            .locations
            .into_iter()
            .find(|location| location.id == self.location_id)
            .unwrap();
        Ok(location)
    }

    async fn get_subscription_info(&self) -> Result<Option<SubscriptionInfo>, Error> {
        let catalog = self.get_subscription_catalog().await?;
        match catalog.subscription_plan_data {
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
        let list = list_res
            .json::<SubscriptionSearchResponse>()
            .await
            .map_err(|_| {
                actix_web::error::ErrorBadRequest(
                    "Failed to parse POST subscription search response from Square",
                )
            })
            .unwrap();
        debug!("Square subscription list: {:?}", &list);

        let mut subs = Vec::<SubscriptionResponse>::new();
        for sub in list.subscriptions.into_iter() {
            let retrieve_endpoint =
                self.base_url.clone() + "v2/subscriptions/" + &*sub.id + "?include=actions";
            debug!("Subcription ID: {}", &sub.id);

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
            let sub = res
                .json::<SubscriptionResponse>()
                .await
                .map_err(|_| {
                    actix_web::error::ErrorBadRequest(
                        "Failed to parse GET retrieve subscription response from Square",
                    )
                })
                .unwrap();
            debug!("Square subscription: {:?}", &sub);
            subs.push(sub);
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
                let list = list_res
                    .json::<SubscriptionSearchResponse>()
                    .await
                    .map_err(|_| {
                        actix_web::error::ErrorBadRequest(
                            "Failed to parse POST subscription search response from Square",
                        )
                    })
                    .unwrap();

                if list.subscriptions.is_empty() {
                    Ok(None)
                } else {
                    let sub: SubscriptionResponseObject = list.subscriptions[0].clone();
                    debug!("Square subscription: {:?}", &sub);
                    Ok(Some(sub))
                }
            }
        }
    }

    pub async fn get_user_profile(
        &self,
        request: UserEmailRequest,
    ) -> Result<Option<UserProfile>, Error> {
        let customer = self.get_customer_info(request.clone()).await?;
        let sub_info = self.get_subscription_info().await?;
        let user_sub_info = self.get_user_subscription_info(request).await?;
        match (customer, sub_info) {
            (Some(customer), Some(sub_info)) => {
                debug!("Customer card: {:?}", customer.cards);
                Ok(Some(UserProfile {
                    customer,
                    subscription_info: sub_info,
                    user_subscription: user_sub_info,
                }))
            }
            _ => Ok(None),
        }
    }

    pub async fn subscribe_checkout(
        &self,
        user_email: UserEmailRequest,
    ) -> Result<CheckoutInfo, Error> {
        let checkout_endpoint = self.base_url.clone() + "v2/online-checkout/payment-links";
        let subscription_plan_id = self
            .get_subscription_catalog()
            .await?
            .subscription_plan_data
            .unwrap()
            .subscription_plan_variations
            .unwrap()
            .get(0)
            .unwrap()
            .id
            .clone();
        let location_id = self.get_location().await?.id;

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
        let checkout = res.json::<CheckoutResponse>().await.map_err(|_| {
            actix_web::error::ErrorBadRequest(
                "Failed to parse POST subscription checkout response from Square",
            )
        })?;
        debug!("Square subscription checkout: {:?}", &checkout);

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

        Ok(checkout_info)
    }

    pub async fn coaching_checkout(&self, request: CoachingRequest) -> Result<CheckoutInfo, Error> {
        let checkout_endpoint = self.base_url.clone() + "v2/online-checkout/payment-links";
        let coaching_package_id = self
            .get_coaching_catalog()
            .await?
            .item_data
            .unwrap()
            .variations
            .unwrap()
            .into_iter()
            .find(|v| v.item_variation_data.name == request.coaching_package.name())
            .unwrap_or_else(|| {
                panic!(
                    "Coaching checkout, no coaching package found by name: {}",
                    request.coaching_package.name()
                )
            })
            .id;
        let location_id = self.get_location().await?.id;
        let customer_id = self
            .get_customer(request.user_email.clone())
            .await?
            .unwrap_or_else(|| {
                panic!(
                    "No customer found for: {}",
                    request.user_email.clone().email.unwrap()
                )
            })
            .id;
        info!("Coaching package id: {}", &coaching_package_id);

        let res = self
            .client
            .post(checkout_endpoint)
            .header("Square-Version", self.version.clone())
            .bearer_auth(self.token.clone())
            .header("Content-Type", "application/json")
            .json(&CheckoutRequest::new_coaching_package(
                CoachingCheckoutBuilder {
                    coaching_package_id,
                    location_id,
                    customer_id,
                    redirect_url: self.redirect_url.clone(),
                    buyer_email: request.user_email.email,
                },
            ))
            .send()
            .await
            .map_err(|_| {
                actix_web::error::ErrorBadRequest("Failed to send POST coaching checkout to Square")
            })?;

        let checkout = res.json::<CheckoutResponse>().await.map_err(|_| {
            actix_web::error::ErrorBadRequest(
                "Failed to parse POST coaching checkout response from Square",
            )
        })?;
        debug!("Square coaching checkout: {:?}", &checkout);

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
        info!("Square coaching checkout info: {:?}", &checkout_info);

        Ok(checkout_info)
    }

    /// Provides customer name and card, but not email
    pub async fn list_customers(&self) -> Result<CustomerListResponse, Error> {
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
        let list = res.json::<CustomerListResponse>().await.map_err(|_| {
            actix_web::error::ErrorBadRequest("Failed to parse GET customers response from Square")
        })?;
        debug!("Square customer list: {:?}", &list);
        Ok(list)
    }

    pub async fn list_invoices(&self) -> Result<InvoiceListResponse, Error> {
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
        let list = res
            .json::<InvoiceListResponse>()
            .await
            .map_err(|_| {
                actix_web::error::ErrorBadRequest(
                    "Failed to parse GET invoices response from Square",
                )
            })
            .unwrap();
        Ok(list)
    }

    // get customer email via invoices endpoint
    pub async fn email_list(&self) -> Result<Vec<CustomerEmailInfo>, Error> {
        let res = self.list_invoices().await?;
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

        Ok(emails)
    }
}
