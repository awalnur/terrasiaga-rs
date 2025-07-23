/// Domain services - Complex business logic that doesn't belong to a single entity
/// These services orchestrate multiple entities and implement complex business rules

use crate::shared::{AppResult, UserId};
use crate::domain::entities::{Disaster, Location, User};
use crate::domain::entities::disaster::{DisasterType, DisasterSeverity};
use crate::domain::entities::user::UserRole;

/// Disaster Assessment Service - Evaluates disaster severity and impact
pub struct DisasterAssessmentService;

impl DisasterAssessmentService {
    /// Calculate comprehensive disaster impact score
    pub fn calculate_impact_score(
        disaster: &Disaster,
        location: &Location,
        affected_population: Option<u32>,
    ) -> f64 {
        let mut score = 0.0;
        
        // Base score from severity (1-5 scale to 0-100)
        score += (disaster.severity.clone() as u8 as f64) * 20.0;
        
        // Location risk factors
        if let Some(max_risk) = location.max_risk_level() {
            score += (max_risk as f64) * 10.0;
        }
        
        // Population impact
        if let Some(population) = affected_population {
            score += (population as f64).log10() * 15.0;
        }
        
        // Disaster type multiplier
        let type_multiplier = match disaster.disaster_type {
            DisasterType::Tsunami => 1.5,
            DisasterType::VolcanicEruption => 1.4,
            DisasterType::Earthquake => 1.3,
            _ => 1.0,
        };
        
        score * type_multiplier
    }
    
    /// Determine recommended response level
    pub fn recommend_response_level(impact_score: f64) -> ResponseLevel {
        match impact_score {
            score if score >= 80.0 => ResponseLevel::National,
            score if score >= 60.0 => ResponseLevel::Regional,
            score if score >= 40.0 => ResponseLevel::Provincial,
            score if score >= 20.0 => ResponseLevel::Local,
            _ => ResponseLevel::Community,
        }
    }
}

/// Emergency Coordination Service - Manages responder assignment and coordination
pub struct EmergencyCoordinationService;

impl EmergencyCoordinationService {
    /// Find optimal responders for a disaster based on proximity, skills, and availability
    pub fn find_optimal_responders(
        disaster: &Disaster,
        available_responders: &[User],
        disaster_location: &Location,
        max_responders: usize,
    ) -> Vec<UserId> {
        let mut scored_responders: Vec<(UserId, f64)> = available_responders
            .iter()
            .map(|user| {
                let score = Self::calculate_responder_score(user, disaster, disaster_location);
                (user.id, score)
            })
            .collect();
        
        // Sort by score (highest first)
        scored_responders.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        scored_responders
            .into_iter()
            .take(max_responders)
            .map(|(user_id, _)| user_id)
            .collect()
    }
    
    /// Calculate responder suitability score
    fn calculate_responder_score(
        responder: &User,
        disaster: &Disaster,
        _disaster_location: &Location,
    ) -> f64 {
        let mut score = 0.0;
        
        // Role-based scoring
        score += match responder.role {
            UserRole::Volunteer => 10.0,
            UserRole::Official => 15.0,
            _ => 5.0,
        };
        
        // Experience/expertise scoring
        score += responder.profile.expertise.len() as f64 * 2.0;
        
        // Disaster type expertise
        if responder.profile.expertise.iter().any(|exp| 
            exp.to_lowercase().contains(&format!("{:?}", disaster.disaster_type).to_lowercase())
        ) {
            score += 20.0;
        }
        
        score
    }
}

/// Risk Calculation Service - Calculates various risk metrics
pub struct RiskCalculationService;

impl RiskCalculationService {
    /// Calculate evacuation radius based on disaster type and severity
    pub fn calculate_evacuation_radius(
        disaster_type: &DisasterType,
        severity: &DisasterSeverity,
    ) -> f64 {
        let base_radius = match disaster_type {
            DisasterType::Tsunami => 5.0,
            DisasterType::VolcanicEruption => 10.0,
            DisasterType::Earthquake => 2.0,
            DisasterType::Flood => 1.0,
            DisasterType::Fire => 0.5,
            _ => 1.0,
        };
        
        let severity_multiplier = match severity {
            DisasterSeverity::Minor => 0.5,
            DisasterSeverity::Moderate => 1.0,
            DisasterSeverity::Major => 1.5,
            DisasterSeverity::Severe => 2.0,
            DisasterSeverity::Catastrophic => 3.0,
        };
        
        base_radius * severity_multiplier
    }
    
    /// Assess location vulnerability to specific disaster types
    pub fn assess_location_vulnerability(
        location: &Location,
        disaster_type: &DisasterType,
    ) -> VulnerabilityLevel {
        // Check specific risk factors for the disaster type
        let relevant_risks = location.risk_factors.iter()
            .filter(|rf| rf.risk_type.to_lowercase().contains(&format!("{:?}", disaster_type).to_lowercase()))
            .map(|rf| rf.severity_level)
            .max()
            .unwrap_or(0);
        
        match relevant_risks {
            5 => VulnerabilityLevel::Extreme,
            4 => VulnerabilityLevel::High,
            3 => VulnerabilityLevel::Moderate,
            2 => VulnerabilityLevel::Low,
            _ => VulnerabilityLevel::Minimal,
        }
    }
}

/// Response level recommendations
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseLevel {
    Community,  // Local community response
    Local,      // Local government
    Provincial, // Provincial government
    Regional,   // Regional coordination
    National,   // National emergency response
}

/// Vulnerability assessment levels
#[derive(Debug, Clone, PartialEq)]
pub enum VulnerabilityLevel {
    Minimal,
    Low,
    Moderate,
    High,
    Extreme,
}
