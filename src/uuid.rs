use uuid::Uuid;

/// Generates a new random UUID (v4) as a String.
pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}
