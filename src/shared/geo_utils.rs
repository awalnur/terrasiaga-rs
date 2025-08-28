/// Geographic utilities for Terra Siaga disaster management
/// Provides location calculations, distance measurements, and geographic operations

use geo::{Point, Polygon, LineString, Contains, HaversineDistance, BoundingRect};
use geo_types::Coord;
use std::collections::HashMap;
use rstar::{RTree, RTreeObject, AABB};
use serde::{Deserialize, Serialize};

use crate::shared::error::{AppResult};
use crate::domain::value_objects::Coordinates;
use crate::shared::types::{constants::EARTH_RADIUS_KM};

/// Geographic region types for disaster management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegionType {
    City,
    District,
    Province,
    Country,
    EmergencyZone,
    EvacuationArea,
    SafeZone,
    RestrictedArea,
}

/// Administrative region with geographic bounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdministrativeRegion {
    pub id: String,
    pub name: String,
    pub region_type: RegionType,
    pub bounds: GeoBounds,
    pub polygon: Option<Vec<Coordinates>>,
    pub parent_region_id: Option<String>,
    pub population: Option<u64>,
    pub area_km2: Option<f64>,
}

/// Point of interest for emergency management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointOfInterest {
    pub id: String,
    pub name: String,
    pub poi_type: PoiType,
    pub coordinates: Coordinates,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoiType {
    Hospital,
    FireStation,
    PoliceStation,
    School,
    Shelter,
    Government,
    Bridge,
    Airport,
    Port,
    PowerPlant,
    WaterTreatment,
    CommunicationTower,
    FuelStation,
    MilitaryBase,
    EmergencyServices,
    Other(String),
}

/// Spatial index for fast geographic queries
pub struct SpatialIndex {
    rtree: RTree<IndexedPoint>,
    regions: HashMap<String, AdministrativeRegion>,
    pois: HashMap<String, PointOfInterest>,
}

/// Point wrapper for R-tree indexing
#[derive(Debug, Clone)]
struct IndexedPoint {
    id: String,
    point: Point<f64>,
    point_type: IndexedPointType,
}

#[derive(Debug, Clone)]
enum IndexedPointType {
    Poi,
    DisasterReport,
    EmergencyResponse,
    Volunteer,
}

impl RTreeObject for IndexedPoint {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let coord = self.point.0;
        AABB::from_point([coord.x, coord.y])
    }
}

// Enable distance-based queries on RTree by providing distance metric in degrees (approximate)
impl rstar::PointDistance for IndexedPoint {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let dx = self.point.x() - point[0];
        let dy = self.point.y() - point[1];
        dx * dx + dy * dy
    }
}

impl SpatialIndex {
    pub fn new() -> Self {
        Self {
            rtree: RTree::new(),
            regions: HashMap::new(),
            pois: HashMap::new(),
        }
    }

    /// Add a point of interest to the spatial index
    pub fn add_poi(&mut self, poi: PointOfInterest) {
        let indexed_point = IndexedPoint {
            id: poi.id.clone(),
            point: Point::new(poi.coordinates.longitude, poi.coordinates.latitude),
            point_type: IndexedPointType::Poi,
        };

        self.rtree.insert(indexed_point);
        self.pois.insert(poi.id.clone(), poi);
    }

    /// Add an administrative region
    pub fn add_region(&mut self, region: AdministrativeRegion) {
        self.regions.insert(region.id.clone(), region);
    }

    /// Find points of interest within a radius
    pub fn find_pois_within_radius(
        &self,
        center: &Coordinates,
        radius_km: f64,
    ) -> Vec<&PointOfInterest> {
        let center_point = Point::new(center.longitude, center.latitude);

        // Use approximate degree conversion for initial candidate set, then filter precisely by Haversine
        let deg_radius = radius_km / 111.32; // ~ km per degree at equator
        let center_arr = [center_point.x(), center_point.y()];
        self.rtree
            .locate_within_distance(center_arr, deg_radius)
            .filter_map(|indexed_point| {
                if matches!(indexed_point.point_type, IndexedPointType::Poi) {
                    self.pois.get(&indexed_point.id)
                } else {
                    None
                }
            })
            .filter(|poi| GeoCalculations::haversine_distance(center, &poi.coordinates) <= radius_km)
            .collect()
    }

    /// Find nearest points of interest
    pub fn find_nearest_pois(
        &self,
        center: &Coordinates,
        limit: usize,
        poi_type: Option<PoiType>,
    ) -> Vec<&PointOfInterest> {
        if limit == 0 { return Vec::new(); }
        let center_point = Point::new(center.longitude, center.latitude);
        let center_arr = [center_point.x(), center_point.y()];

        let mut candidates: Vec<&PointOfInterest> = self
            .rtree
            .nearest_neighbor_iter(&center_arr)
            .filter_map(|indexed_point| {
                if matches!(indexed_point.point_type, IndexedPointType::Poi) {
                    self.pois.get(&indexed_point.id)
                } else {
                    None
                }
            })
            .filter(|poi| {
                if let Some(ref filter_type) = poi_type {
                    std::mem::discriminant(&poi.poi_type) == std::mem::discriminant(filter_type)
                } else {
                    true
                }
            })
            .collect();

        // Sort by geodesic distance to ensure correctness
        candidates.sort_by(|a, b| {
            let da = GeoCalculations::haversine_distance(center, &a.coordinates);
            let db = GeoCalculations::haversine_distance(center, &b.coordinates);
            da.partial_cmp(&db).unwrap()
        });

        candidates.into_iter().take(limit).collect()
    }

    /// Check if a point is within any administrative region
    pub fn find_containing_regions(&self, point: &Coordinates) -> Vec<&AdministrativeRegion> {
        self.regions
            .values()
            .filter(|region| {
                if let Some(ref polygon_coords) = region.polygon {
                    self.point_in_polygon(point, polygon_coords)
                } else {
                    region.bounds.contains(point)
                }
            })
            .collect()
    }

    /// Check if point is within polygon
    fn point_in_polygon(&self, point: &Coordinates, polygon: &[Coordinates]) -> bool {
        if polygon.len() < 3 {
            return false;
        }

        let coords: Vec<Coord<f64>> = polygon
            .iter()
            .map(|c| Coord { x: c.longitude, y: c.latitude })
            .collect();

        let line_string = LineString::new(coords);
        let polygon = Polygon::new(line_string, vec![]);
        let point_geo = Point::new(point.longitude, point.latitude);

        polygon.contains(&point_geo)
    }
}

/// Geographic calculation utilities
pub struct GeoCalculations;

impl GeoCalculations {
    /// Calculate distance between two points using Haversine formula
    pub fn haversine_distance(point1: &Coordinates, point2: &Coordinates) -> f64 {
        let p1 = Point::new(point1.longitude, point1.latitude);
        let p2 = Point::new(point2.longitude, point2.latitude);

        p1.haversine_distance(&p2) / 1000.0 // Convert to kilometers
    }

    /// Calculate bearing from point1 to point2
    pub fn calculate_bearing(point1: &Coordinates, point2: &Coordinates) -> f64 {
        let lat1 = point1.latitude.to_radians();
        let lat2 = point2.latitude.to_radians();
        let delta_lon = (point2.longitude - point1.longitude).to_radians();

        let y = delta_lon.sin() * lat2.cos();
        let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * delta_lon.cos();

        let bearing = y.atan2(x).to_degrees();
        (bearing + 360.0) % 360.0
    }

    /// Calculate destination point given distance and bearing
    pub fn calculate_destination(
        start: &Coordinates,
        distance_km: f64,
        bearing_degrees: f64,
    ) -> Coordinates {
        let lat1 = start.latitude.to_radians();
        let lon1 = start.longitude.to_radians();
        let bearing = bearing_degrees.to_radians();
        let angular_distance = distance_km / EARTH_RADIUS_KM;

        let lat2 = (lat1.sin() * angular_distance.cos()
            + lat1.cos() * angular_distance.sin() * bearing.cos())
        .asin();

        let lon2 = lon1
            + (bearing.sin() * angular_distance.sin() * lat1.cos())
                .atan2(angular_distance.cos() - lat1.sin() * lat2.sin());

        Coordinates {
            latitude: lat2.to_degrees(),
            longitude: lon2.to_degrees(),
            altitude: start.altitude,
        }
    }

    /// Calculate bounding box for a circle
    pub fn calculate_bounding_box(center: &Coordinates, radius_km: f64) -> GeoBounds {
        let north = Self::calculate_destination(center, radius_km, 0.0);
        let south = Self::calculate_destination(center, radius_km, 180.0);
        let east = Self::calculate_destination(center, radius_km, 90.0);
        let west = Self::calculate_destination(center, radius_km, 270.0);

        GeoBounds {
            north_east: Coordinates {
                latitude: north.latitude,
                longitude: east.longitude,
                altitude: None,
            },
            south_west: Coordinates {
                latitude: south.latitude,
                longitude: west.longitude,
                altitude: None,
            },
        }
    }

    /// Calculate area of a polygon in square kilometers
    pub fn calculate_polygon_area(coordinates: &[Coordinates]) -> f64 {
        if coordinates.len() < 3 {
            return 0.0;
        }

        let coords: Vec<Coord<f64>> = coordinates
            .iter()
            .map(|c| Coord { x: c.longitude, y: c.latitude })
            .collect();

        let line_string = LineString::new(coords);
        let polygon = Polygon::new(line_string, vec![]);

        // This is an approximation - for accurate area calculation,
        // you'd want to project to an appropriate coordinate system
        let bbox = polygon.bounding_rect().unwrap();
        // Correct lat/lon (y/x) ordering when constructing Coordinates
        let width = Self::haversine_distance(
            &Coordinates::new(bbox.min().y, bbox.min().x).unwrap(),
            &Coordinates::new(bbox.max().y, bbox.min().x).unwrap(),
        );
        let height = Self::haversine_distance(
            &Coordinates::new(bbox.min().y, bbox.min().x).unwrap(),
            &Coordinates::new(bbox.min().y, bbox.max().x).unwrap(),
        );

        width * height // Rough approximation
    }

    /// Find centroid of a set of points
    pub fn calculate_centroid(points: &[Coordinates]) -> Option<Coordinates> {
        if points.is_empty() {
            return None;
        }

        let lat_sum: f64 = points.iter().map(|p| p.latitude).sum();
        let lon_sum: f64 = points.iter().map(|p| p.longitude).sum();
        let count = points.len() as f64;

        Some(Coordinates {
            latitude: lat_sum / count,
            longitude: lon_sum / count,
            altitude: None,
        })
    }

    /// Check if two bounding boxes intersect
    pub fn bounds_intersect(bounds1: &GeoBounds, bounds2: &GeoBounds) -> bool {
        bounds1.north_east.latitude >= bounds2.south_west.latitude
            && bounds1.south_west.latitude <= bounds2.north_east.latitude
            && bounds1.north_east.longitude >= bounds2.south_west.longitude
            && bounds1.south_west.longitude <= bounds2.north_east.longitude
    }

    /// Generate points along a line between two coordinates
    pub fn interpolate_line(
        start: &Coordinates,
        end: &Coordinates,
        num_points: usize,
    ) -> Vec<Coordinates> {
        if num_points == 0 {
            return vec![];
        }

        let mut points = Vec::with_capacity(num_points + 2);
        points.push(start.clone());

        if num_points > 0 {
            for i in 1..=num_points {
                let t = i as f64 / (num_points + 1) as f64;
                let lat = start.latitude + (end.latitude - start.latitude) * t;
                let lon = start.longitude + (end.longitude - start.longitude) * t;

                points.push(Coordinates {
                    latitude: lat,
                    longitude: lon,
                    altitude: None,
                });
            }
        }

        points.push(end.clone());
        points
    }
}

/// Emergency zone management
pub struct EmergencyZoneManager {
    spatial_index: SpatialIndex,
    evacuation_routes: HashMap<String, EvacuationRoute>,
    safe_zones: HashMap<String, SafeZone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvacuationRoute {
    pub id: String,
    pub name: String,
    pub start_location: Coordinates,
    pub end_location: Coordinates,
    pub waypoints: Vec<Coordinates>,
    pub capacity_per_hour: u32,
    pub is_active: bool,
    pub alternative_routes: Vec<String>,
    pub terrain_difficulty: TerrainDifficulty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerrainDifficulty {
    Easy,    // Paved roads, accessible to vehicles
    Medium,  // Some rough terrain, walkable
    Hard,    // Difficult terrain, requires preparation
    Extreme, // Very dangerous, last resort only
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeZone {
    pub id: String,
    pub name: String,
    pub location: Coordinates,
    pub radius_km: f64,
    pub capacity: u32,
    pub current_occupancy: u32,
    pub facilities: Vec<Facility>,
    pub is_operational: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Facility {
    MedicalStation,
    FoodDistribution,
    WaterSupply,
    Shelter,
    Communication,
    PowerGeneration,
    Sanitation,
    Security,
}

impl EmergencyZoneManager {
    pub fn new() -> Self {
        Self {
            spatial_index: SpatialIndex::new(),
            evacuation_routes: HashMap::new(),
            safe_zones: HashMap::new(),
        }
    }

    /// Find nearest safe zones to a location
    pub fn find_nearest_safe_zones(
        &self,
        location: &Coordinates,
        limit: usize,
    ) -> Vec<&SafeZone> {
        let mut zones: Vec<_> = self
            .safe_zones
            .values()
            .filter(|zone| zone.is_operational && zone.current_occupancy < zone.capacity)
            .collect();

        zones.sort_by(|a, b| {
            let dist_a = GeoCalculations::haversine_distance(location, &a.location);
            let dist_b = GeoCalculations::haversine_distance(location, &b.location);
            dist_a.partial_cmp(&dist_b).unwrap()
        });

        zones.into_iter().take(limit).collect()
    }

    /// Find evacuation routes from a location
    pub fn find_evacuation_routes(
        &self,
        from_location: &Coordinates,
        max_distance_km: f64,
    ) -> Vec<&EvacuationRoute> {
        self.evacuation_routes
            .values()
            .filter(|route| {
                route.is_active
                    && GeoCalculations::haversine_distance(from_location, &route.start_location)
                        <= max_distance_km
            })
            .collect()
    }

    /// Calculate evacuation time estimate
    pub fn estimate_evacuation_time(
        &self,
        from_location: &Coordinates,
        route_id: &str,
        population: u32,
    ) -> Option<std::time::Duration> {
        let route = self.evacuation_routes.get(route_id)?;

        let distance = GeoCalculations::haversine_distance(from_location, &route.start_location);
        let route_distance = GeoCalculations::haversine_distance(
            &route.start_location,
            &route.end_location,
        );

        // Simple calculation - in practice this would be much more complex
        let travel_time_hours = (distance + route_distance) / 5.0; // Assume 5 km/h walking speed
        let processing_time_hours = population as f64 / route.capacity_per_hour as f64;

        let total_hours = travel_time_hours + processing_time_hours;
        Some(std::time::Duration::from_secs((total_hours * 3600.0) as u64))
    }
}

/// Geographic bounds for a rectangular area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoBounds {
    pub north_east: Coordinates,
    pub south_west: Coordinates,
}

impl GeoBounds {
    pub fn new(north_east: Coordinates, south_west: Coordinates) -> Self {
        Self { north_east, south_west }
    }

    pub fn contains(&self, point: &Coordinates) -> bool {
        point.latitude <= self.north_east.latitude
            && point.latitude >= self.south_west.latitude
            && point.longitude <= self.north_east.longitude
            && point.longitude >= self.south_west.longitude
    }
}

/// Location information with address details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    pub coordinates: Coordinates,
    pub address: Option<String>,
    pub administrative: Option<AdministrativeRegion>,
    pub landmark: Option<String>,
    pub accuracy_radius: Option<f64>,
}

/// Geocoding service interface
#[async_trait::async_trait]
pub trait GeocodingService: Send + Sync {
    async fn geocode_address(&self, address: &str) -> AppResult<Vec<Coordinates>>;
    async fn reverse_geocode(&self, coordinates: &Coordinates) -> AppResult<LocationInfo>;
    async fn get_administrative_info(&self, coordinates: &Coordinates) -> AppResult<Vec<AdministrativeRegion>>;
}

/// Mock geocoding service for testing
pub struct MockGeocodingService;

#[async_trait::async_trait]
impl GeocodingService for MockGeocodingService {
    async fn geocode_address(&self, _address: &str) -> AppResult<Vec<Coordinates>> {
        // Return Jakarta coordinates as default
        Ok(vec![Coordinates::new(-6.2088, 106.8456).unwrap()])
    }

    async fn reverse_geocode(&self, coordinates: &Coordinates) -> AppResult<LocationInfo> {
        Ok(LocationInfo {
            coordinates: coordinates.clone(),
            address: Some("Mock Address".to_string()),
            administrative: None,
            landmark: None,
            accuracy_radius: Some(100.0),
        })
    }

    async fn get_administrative_info(&self, _coordinates: &Coordinates) -> AppResult<Vec<AdministrativeRegion>> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_distance() {
        let jakarta = Coordinates::new(-6.2088, 106.8456).unwrap();
        let bandung = Coordinates::new(-6.9175, 107.6191).unwrap();

        let distance = GeoCalculations::haversine_distance(&jakarta, &bandung);

        // Distance between Jakarta and Bandung is approximately 150km
        assert!((distance - 150.0).abs() < 20.0);
    }

    #[test]
    fn test_bearing_calculation() {
        let start = Coordinates::new(0.0, 0.0).unwrap();
        let north = Coordinates::new(1.0, 0.0).unwrap();
        let east = Coordinates::new(0.0, 1.0).unwrap();

        let bearing_north = GeoCalculations::calculate_bearing(&start, &north);
        let bearing_east = GeoCalculations::calculate_bearing(&start, &east);

        assert!((bearing_north - 0.0).abs() < 1.0);
        assert!((bearing_east - 90.0).abs() < 1.0);
    }

    #[test]
    fn test_bounding_box_calculation() {
        let center = Coordinates::new(-6.2088, 106.8456).unwrap();
        let bounds = GeoCalculations::calculate_bounding_box(&center, 10.0);

        assert!(bounds.north_east.latitude > center.latitude);
        assert!(bounds.south_west.latitude < center.latitude);
        assert!(bounds.north_east.longitude > center.longitude);
        assert!(bounds.south_west.longitude < center.longitude);
    }

    #[test]
    fn test_spatial_index() {
        let mut index = SpatialIndex::new();

        let poi = PointOfInterest {
            id: "hospital1".to_string(),
            name: "General Hospital".to_string(),
            poi_type: PoiType::Hospital,
            coordinates: Coordinates::new(-6.2088, 106.8456).unwrap(),
            metadata: HashMap::new(),
        };

        index.add_poi(poi);

        let center = Coordinates::new(-6.2, 106.85).unwrap();
        let nearby_pois = index.find_pois_within_radius(&center, 5.0);

        assert_eq!(nearby_pois.len(), 1);
        assert_eq!(nearby_pois[0].id, "hospital1");
    }

    #[test]
    fn test_evacuation_time_estimation() {
        let mut manager = EmergencyZoneManager::new();

        let route = EvacuationRoute {
            id: "route1".to_string(),
            name: "Main Evacuation Route".to_string(),
            start_location: Coordinates::new(-6.2088, 106.8456).unwrap(),
            end_location: Coordinates::new(-6.1744, 106.8227).unwrap(),
            waypoints: vec![],
            capacity_per_hour: 1000,
            is_active: true,
            alternative_routes: vec![],
            terrain_difficulty: TerrainDifficulty::Easy,
        };

        manager.evacuation_routes.insert(route.id.clone(), route);

        let from_location = Coordinates::new(-6.21, 106.84).unwrap();
        let time = manager.estimate_evacuation_time(&from_location, "route1", 500);

        assert!(time.is_some());
        assert!(time.unwrap().as_secs() > 0);
    }
}
