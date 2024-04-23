use actix_utils::future::{err, ok, Ready};
use actix_web::FromRequest;
use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use base64::prelude::*;
use c8y_sdk::inventory::Inventory;

use crate::microservice_subscription::SERVICE;

#[derive(Clone)]
pub struct Platform {
    pub tenant: String,
    pub username: String,
    pub password: String,
    pub base_url: String,
    pub inventory_api: Inventory,
}

#[derive(Debug, thiserror::Error)]
#[error("Error while processing header")]
pub struct HeaderProcessingError {
    pub message: String,
    pub detailed_message: Option<String>,
}

impl FromRequest for Platform {
    type Error = HeaderProcessingError;

    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let platform: Platform = match get_current_platform(req) {
            Ok(value) => value,
            Err(value) => return err(value),
        };
        ok(platform)
    }
}

impl error::ResponseError for HeaderProcessingError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(format!(
                "{{\"message\": \"{}\", \"detailedMessage\": \"{}\"}}",
                self.message.clone(),
                self.detailed_message.clone().unwrap_or(String::from(""))
            ))
    }
}

pub fn get_current_platform(
    req: &actix_web::HttpRequest,
) -> Result<Platform, HeaderProcessingError> {
    let platforms = &SERVICE.platforms;
    let auth_header = req.headers().get("authorization");
    let auth = match auth_header {
        Some(ah) => ah.to_str().unwrap(),
        None => {
            return Err(HeaderProcessingError {
                message: String::from("Authorization header is missing"),
                detailed_message: None,
            });
        }
    };
    let mut splitted = auth.split(' ');
    match splitted.next() {
        Some(b) => {
            if !b.eq("Basic") {
                return Err(HeaderProcessingError {
                    message: String::from("Authorization header must be basic authentication"),
                    detailed_message: None,
                });
            }
        }
        None => {
            return Err(HeaderProcessingError {
                message: String::from("Authorization header must be basic authentication"),
                detailed_message: None,
            });
        }
    };
    let base64_encoded = match splitted.next() {
        Some(be) => be,
        None => {
            return Err(HeaderProcessingError {
                message: String::from("Missing base64 encoded part of authorization header"),
                detailed_message: None,
            });
        }
    };
    let decoded = match BASE64_STANDARD.decode(base64_encoded) {
        Ok(d) => d,
        Err(e) => {
            return Err(HeaderProcessingError {
                message: String::from("Couldn't decode authorization header"),
                detailed_message: Some(e.to_string()),
            });
        }
    };
    let decoded = match String::from_utf8(decoded) {
        Ok(d) => d,
        Err(e) => {
            return Err(HeaderProcessingError {
                message: String::from("No UTF8 compatible string"),
                detailed_message: Some(e.to_string()),
            });
        }
    };
    let mut decoded_splitted = decoded.split('/');
    let tenant = match decoded_splitted.next() {
        Some(t) => t,
        None => {
            return Err(HeaderProcessingError {
                message: String::from("Missing tenant prefix in credentials"),
                detailed_message: None,
            });
        }
    };
    match decoded_splitted.next() {
        Some(t) => t,
        None => {
            return Err(HeaderProcessingError {
                message: String::from("Missing tenant prefix in credentials"),
                detailed_message: None,
            });
        }
    };
    println!("Tenant: {}", tenant);
    let platform = match platforms.get(tenant) {
        Some(p) => p,
        None => {
            return Err(HeaderProcessingError {
                message: format!("No subscription for tenant {}", tenant),
                detailed_message: None,
            });
        }
    };
    Ok(platform.clone())
}
