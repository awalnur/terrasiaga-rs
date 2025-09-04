use actix_web::{dev::{ServiceRequest, ServiceResponse}, HttpResponse, Error, Result, body::EitherBody, ResponseError};
use futures::future::{ok, Ready};
use serde_json::json;
use tracing::error;
use std::pin::Pin;
use std::future::Future;
use std::task::{Context, Poll};
use crate::AppError;

pub struct ErrorHandler;

impl ErrorHandler {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> actix_web::dev::Transform<S, ServiceRequest> for ErrorHandler
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = ErrorHandlerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ErrorHandlerMiddleware { service })
    }
}

pub struct ErrorHandlerMiddleware<S> {
    service: S,
}

impl<S, B> actix_web::dev::Service<ServiceRequest> for ErrorHandlerMiddleware<S>
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            match fut.await {
                Ok(res) => {
                    let status = res.status();

                    if status.is_client_error() || status.is_server_error() {
                        // Let 400 Bad Request (e.g., JSON deserialization) pass through to preserve detailed message
                        if status.as_u16() == 400 {
                            return Ok(res.map_into_left_body());
                        }

                        let error_response = match status.as_u16() {
                            401 => AppError::Unauthorized("Unauthorized".to_string()).error_response(),
                            403 => HttpResponse::Forbidden().json(json!({
                                "error": "Forbidden",
                                "message": "Access denied",
                                "status": 403
                            })),

                            404 => AppError::NotFound("Resource Not Found".to_string()).error_response(),
                            500 => {
                                error!("Internal server error occurred");
                                HttpResponse::InternalServerError().json(json!({
                                    "error": "Internal Server Error",
                                    "message": "An unexpected error occurred",
                                    "status": 500
                                }))
                            },
                            _ => HttpResponse::build(status).json(json!({
                                "error": status.canonical_reason().unwrap_or("Unknown Error"),
                                "message": "An error occurred",
                                "status": status.as_u16()
                            }))
                        };

                        // Replace entire response to keep headers/body consistent
                        let new_res = res.into_response(error_response.map_into_right_body());
                        Ok(new_res)
                    } else {
                        Ok(res.map_into_left_body())
                    }
                },
                Err(err) => {
                    error!("Service error: {}", err);
                    Err(err)
                }
            }
        })
    }
}