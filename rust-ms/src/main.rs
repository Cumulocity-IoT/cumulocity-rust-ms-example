use actix_web::{
    delete, error, get,
    http::{header::ContentType, StatusCode},
    post, put, web, App, HttpResponse, HttpServer, Responder,
};
use c8y_ms_sdk::{microservice_subscription::SERVICE, platform::Platform};
use c8y_sdk::{
    cumulocity_error::CumulocityError,
    inventory::{CreateManagedObject, ManagedObject},
};

#[derive(thiserror::Error, Debug)]
pub enum BackendError {
    #[error(transparent)]
    WrappedCumulocityError(#[from] CumulocityError),
}

impl error::ResponseError for BackendError {
    fn error_response(&self) -> HttpResponse {
        match self {
            BackendError::WrappedCumulocityError(e) => HttpResponse::build(self.status_code())
                .insert_header(ContentType::json())
                .body(format!(
                    "{{\"message\":\"{}\", \"detailedMessage\":\"{}\"}}",
                    e.to_string(),
                    e.source.to_string()
                )),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            BackendError::WrappedCumulocityError(e) => {
                StatusCode::from_u16(e.source.status().unwrap().as_u16()).unwrap()
            }
        }
    }
}

#[get("/{id}")]
async fn get_managed_object(
    id: web::Path<String>,
    platform: Platform,
) -> Result<impl Responder, BackendError> {
    Ok(web::Json(
        platform
            .inventory_api
            .get_managed_object(id.to_string())
            .await?,
    ))
}

#[delete("/{id}")]
async fn delete_managed_object(
    id: web::Path<String>,
    platform: Platform,
) -> Result<impl Responder, BackendError> {
    Ok((
        platform
            .inventory_api
            .delete_managed_object(id.to_string())
            .await?,
        StatusCode::NO_CONTENT,
    ))
}

#[post("/")]
async fn create_managed_object(
    mo: web::Json<CreateManagedObject>,
    platform: Platform,
) -> Result<impl Responder, BackendError> {
    Ok((
        web::Json(platform.inventory_api.create_managed_object(mo.0).await?),
        StatusCode::CREATED,
    ))
}

#[put("/")]
async fn update_managed_object(
    mo: web::Json<ManagedObject>,
    platform: Platform,
) -> Result<impl Responder, BackendError> {
    Ok(web::Json(
        platform.inventory_api.update_managed_object(mo.0).await?,
    ))
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    SERVICE.add_subscription_listener(on_new_subscription).await;

    SERVICE.start_subscription_listener().await;

    HttpServer::new(move || {
        App::new()
            .service(get_managed_object)
            .service(create_managed_object)
            .service(delete_managed_object)
            .service(update_managed_object)
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}

fn on_new_subscription(platform: Platform) {
    println!("New subscription detected: {}", platform.tenant);
}
