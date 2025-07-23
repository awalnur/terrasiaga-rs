/// Authentication middleware
/// Handles JWT token validation and user authentication

use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use uuid::Uuid;
use crate::shared::jwt::validate_token;

pub async fn jwt_middleware(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    let token = credentials.token();
    
    match validate_token(token) {
        Ok(claims) => {
            // Parse user ID from claims
            if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                req.extensions_mut().insert(user_id);
                Ok(req)
            } else {
                Err(actix_web::error::ErrorUnauthorized("Invalid user ID in token"))
            }
        }
        Err(_) => Err(actix_web::error::ErrorUnauthorized("Invalid token")),
    }
}
