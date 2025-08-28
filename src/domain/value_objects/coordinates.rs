// filepath: /Users/development/RUST/terra-siaga/src/domain/value_objects/coordinates.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
}

impl Coordinates {
    pub fn new(latitude: f64, longitude: f64) -> Result<Self, &'static str> {
        if latitude < -90.0 || latitude > 90.0 {
            return Err("Latitude must be between -90 and 90 degrees");
        }
        if longitude < -180.0 || longitude > 180.0 {
            return Err("Longitude must be between -180 and 180 degrees");
        }
        Ok(Self { latitude, longitude, altitude: None })
    }

    pub fn with_altitude(mut self, altitude: f64) -> Self {
        self.altitude = Some(altitude);
        self
    }

    pub fn is_within_indonesia(&self) -> bool {
        // Rough bounding box for Indonesia
        let lat_ok = self.latitude >= -11.0 && self.latitude <= 6.5;
        let lon_ok = self.longitude >= 95.0 && self.longitude <= 141.0;
        lat_ok && lon_ok
    }

    pub fn distance_to(&self, other: &Coordinates) -> f64 {
        use std::f64::consts::PI;
        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let dlat = (other.latitude - self.latitude).to_radians();
        let dlon = (other.longitude - self.longitude).to_radians();
        let a = (dlat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        6371.0 * c
    }
}
