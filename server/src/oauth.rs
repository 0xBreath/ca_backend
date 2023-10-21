use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web::{dev::ServiceRequest, Error as ActixError};
use crate::errors::ServiceError;
use alcoholic_jwt::{token_kid, validate, Validation, JWKS};
use serde::{Deserialize, Serialize};
use std::error::Error;
use log::*;

// Auth0 Rust example
// https://auth0.com/blog/build-an-api-in-rust-with-jwt-authentication-using-actix-web/#Getting-Started

pub async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (ActixError, ServiceRequest)> {
  debug!("req: {:?}", req);
  debug!("credentials: {:?}", credentials);
  let config = req
    .app_data::<Config>().cloned().unwrap_or_default();
  match validate_token(credentials.token()).await {
    Ok(res) => {
      if res {
        debug!("Token validated");
        Ok(req)
      } else {
        info!("Token validation failed");
        Err((AuthenticationError::from(config).into(), req))
      }
    }
    Err(_) => {
      error!("Token validation errored");
      Err((AuthenticationError::from(config).into(), req))
    },
  }
}

async fn fetch_jwks(uri: &str) -> Result<JWKS, Box<dyn Error>> {
  let res = reqwest::get(uri).await?;
  let val = res.json::<JWKS>().await?;
  Ok(val)
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
  sub: String,
  company: String,
  exp: usize,
}

pub async fn validate_token(token: &str) -> Result<bool, ServiceError> {
  let authority = std::env::var("AUTHORITY").expect("AUTHORITY must be set");
  let jwks = fetch_jwks(&format!("{}{}", authority.as_str(), ".well-known/jwks.json")).await
    .expect("failed to fetch jwks");
  let validations = vec![Validation::Issuer(authority), Validation::SubjectPresent];
  let kid = match token_kid(token) {
    Ok(res) => res.expect("failed to decode kid"),
    Err(_) => return Err(ServiceError::JWKSFetchError),
  };
  let jwk = jwks.find(&kid).expect("Specified key not found in set");
  let res = validate(token, jwk, validations);
  Ok(res.is_ok())
}