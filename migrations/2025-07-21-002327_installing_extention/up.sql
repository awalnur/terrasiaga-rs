-- This migration script installs the necessary extensions for the database.
-- Create extensions if it does not exist
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "postgis";