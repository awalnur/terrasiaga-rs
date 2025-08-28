// filepath: /Users/development/RUST/terra-siaga/src/domain/value_objects/user_status.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    SysAdmin,
    Pending,
    Active,
    Inactive,
    Suspended,
    Banned,
}

impl UserStatus {
    pub fn can_login(&self) -> bool {
        matches!(self, UserStatus::Active)
    }

    pub fn is_active(&self) -> bool {
        matches!(self, UserStatus::Active)
    }
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            UserStatus::Pending => "pending",
            UserStatus::Active => "active",
            UserStatus::Inactive => "inactive",
            UserStatus::Suspended => "suspended",
            UserStatus::Banned => "banned",
            UserStatus::SysAdmin => "sysadmin",
        };
        write!(f, "{}", s)
    }
}
