// filepath: /Users/development/RUST/terra-siaga/src/domain/value_objects/user_role.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Citizen,
    Volunteer,
    Responder,
    Admin,
    SuperAdmin,
}

impl UserRole {
    pub fn can_perform(&self, permission: &str) -> bool {
        match self {
            UserRole::SuperAdmin => true,
            UserRole::Admin => matches!(
                permission,
                "manage_users"
                    | "analytics:read"
                    | "analytics:write"
                    | "reports:read"
                    | "reports:write"
                    | "alerts:read"
                    | "alerts:write"
                    | "emergency:manage"
            ),
            UserRole::Responder => matches!(
                permission,
                "emergency:update_status"
                    | "emergency:respond"
                    | "reports:read"
                    | "alerts:read"
            ),
            UserRole::Volunteer => matches!(
                permission,
                "volunteers:respond"
                    | "volunteers:read"
                    | "reports:read"
                    | "reports:write"
                    | "alerts:read"
            ),
            UserRole::Citizen => matches!(
                permission,
                "reports:read" | "reports:write" | "alerts:read" | "data:read_public"
            ),
        }
    }
}
