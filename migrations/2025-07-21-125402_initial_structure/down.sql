-- This file should undo anything in `up.sql`

-- Drop triggers first
DROP TRIGGER IF EXISTS update_roles_timestamp ON roles;
DROP TRIGGER IF EXISTS update_users_timestamp ON users;
DROP TRIGGER IF EXISTS update_auth_sessions_timestamp ON auth_sessions;
DROP TRIGGER IF EXISTS update_refresh_tokens_timestamp ON refresh_tokens;
DROP TRIGGER IF EXISTS update_disaster_types_timestamp ON disaster_types;
DROP TRIGGER IF EXISTS update_locations_timestamp ON locations;
DROP TRIGGER IF EXISTS update_reports_timestamp ON reports;
DROP TRIGGER IF EXISTS update_report_media_timestamp ON report_media;
DROP TRIGGER IF EXISTS update_notifications_timestamp ON notifications;
DROP TRIGGER IF EXISTS update_volunteers_timestamp ON volunteers;
DROP TRIGGER IF EXISTS update_volunteer_tracking_timestamp ON volunteer_tracking;
DROP TRIGGER IF EXISTS update_organizations_timestamp ON organizations;
DROP TRIGGER IF EXISTS update_organization_members_timestamp ON organization_members;
DROP TRIGGER IF EXISTS update_disasters_timestamp ON disasters;
DROP TRIGGER IF EXISTS update_evacuation_centers_timestamp ON evacuation_centers;
DROP TRIGGER IF EXISTS update_evacuation_center_facilities_timestamp ON evacuation_center_facilities;
DROP TRIGGER IF EXISTS update_emergency_resources_timestamp ON emergency_resources;
DROP TRIGGER IF EXISTS update_resource_allocations_timestamp ON resource_allocations;
DROP TRIGGER IF EXISTS update_report_comments_timestamp ON report_comments;

-- Drop the timestamp update function
DROP FUNCTION IF EXISTS update_timestamp();

-- Drop tables in reverse order of creation (respecting foreign key constraints)
DROP TABLE IF EXISTS weather_data;
DROP TABLE IF EXISTS resource_allocations;
DROP TABLE IF EXISTS emergency_resources;
DROP TABLE IF EXISTS evacuation_center_facilities;
DROP TABLE IF EXISTS evacuation_centers;
DROP TABLE IF EXISTS disaster_zones;
DROP TABLE IF EXISTS disaster_movements;
DROP TABLE IF EXISTS disaster_analytics;
DROP TABLE IF EXISTS disaster_reports;
DROP TABLE IF EXISTS volunteer_locations;
DROP TABLE IF EXISTS volunteer_tracking;
DROP TABLE IF EXISTS volunteers;
DROP TABLE IF EXISTS report_comments;
DROP TABLE IF EXISTS report_media;
DROP TABLE IF EXISTS report_history;
DROP TABLE IF EXISTS notifications;
DROP TABLE IF EXISTS organization_members;
DROP TABLE IF EXISTS organizations;
DROP TABLE IF EXISTS disasters;
DROP TABLE IF EXISTS reports;
DROP TABLE IF EXISTS locations;
DROP TABLE IF EXISTS disaster_types;
DROP TABLE IF EXISTS refresh_tokens;
DROP TABLE IF EXISTS auth_sessions;
DROP TABLE IF EXISTS verification_codes;
DROP TABLE IF EXISTS user_roles;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS roles;