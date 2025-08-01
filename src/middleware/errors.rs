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
                        let error_response = match status.as_u16() {
                            400 => HttpResponse::BadRequest().json(json!({
                                "error": "Bad Request",
                                "message": "Invalid request parameters",
                                "status": 400
                            })),
                            401 => HttpResponse::Unauthorized().json(json!({
                                "error": "Unauthorized",
                                "message": "Authentication required",
                                "status": 401
                            })),
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

                        Ok(res.map_body(|_, _| EitherBody::right(error_response.into_body())))
                    } else {
                        Ok(res.map_body(|_, body| EitherBody::left(body)))
                    }
                },
                Err(err) => {
                    error!("Service error: {}", err);

                    // Return the original error instead of trying to wrap HttpResponse
                    Err(err)
                }
            }
        })
    }
}