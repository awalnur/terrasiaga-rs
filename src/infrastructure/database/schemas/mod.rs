// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "geography"))]
    pub struct Geography;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "geometry"))]
    pub struct Geometry;
}

diesel::table! {
    auth_sessions (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        token -> Text,
        user_agent -> Nullable<Text>,
        ip_address -> Nullable<Text>,
        expires_at -> Timestamp,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    disaster_analytics (id) {
        id -> Uuid,
        disaster_id -> Nullable<Uuid>,
        affected_population -> Nullable<Int4>,
        economic_loss_estimation -> Nullable<Int8>,
        infrastructure_damage -> Nullable<Text>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Geometry;

    disaster_movements (id) {
        id -> Uuid,
        disaster_id -> Nullable<Uuid>,
        geometry -> Nullable<Geometry>,
        speed -> Nullable<Float8>,
        direction -> Nullable<Float8>,
        description -> Nullable<Text>,
        recorded_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    disaster_reports (disaster_id, report_id) {
        disaster_id -> Uuid,
        report_id -> Uuid,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    disaster_types (id) {
        id -> Int4,
        name -> Text,
        description -> Nullable<Text>,
        severity -> Int4,
        icon_url -> Nullable<Text>,
        #[max_length = 7]
        color_code -> Nullable<Varchar>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Geometry;

    disaster_zones (id) {
        id -> Uuid,
        disaster_id -> Nullable<Uuid>,
        area -> Nullable<Geometry>,
        #[max_length = 50]
        zone_type -> Nullable<Varchar>,
        description -> Nullable<Text>,
        recorded_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    disasters (id) {
        id -> Uuid,
        disaster_type_id -> Nullable<Int4>,
        name -> Text,
        description -> Nullable<Text>,
        severity -> Nullable<Int4>,
        status -> Nullable<Text>,
        start_time -> Nullable<Timestamp>,
        end_time -> Nullable<Timestamp>,
        primary_location_id -> Nullable<Uuid>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    emergency_resources (id) {
        id -> Uuid,
        name -> Text,
        category -> Text,
        quantity -> Int4,
        unit -> Text,
        location_id -> Nullable<Uuid>,
        organization_id -> Nullable<Uuid>,
        expiry_date -> Nullable<Date>,
        status -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    evacuation_center_facilities (id) {
        id -> Uuid,
        evacuation_center_id -> Nullable<Uuid>,
        facility_name -> Text,
        quantity -> Nullable<Int4>,
        status -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    evacuation_centers (id) {
        id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        capacity -> Nullable<Int4>,
        current_occupancy -> Nullable<Int4>,
        location_id -> Nullable<Uuid>,
        status -> Nullable<Text>,
        contact_person -> Nullable<Text>,
        contact_phone -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Geography;

    locations (id) {
        id -> Uuid,
        name -> Text,
        region -> Text,
        province -> Nullable<Text>,
        city -> Nullable<Text>,
        postal_code -> Nullable<Text>,
        geometry -> Nullable<Geography>,
        address -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    notifications (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        title -> Text,
        message -> Text,
        channel -> Text,
        #[max_length = 20]
        status -> Nullable<Varchar>,
        is_read -> Nullable<Bool>,
        send_at -> Nullable<Timestamp>,
        read_at -> Nullable<Timestamp>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    organization_members (organization_id, user_id) {
        organization_id -> Uuid,
        user_id -> Uuid,
        role -> Text,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    organizations (id) {
        id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        logo_url -> Nullable<Text>,
        website -> Nullable<Text>,
        phone -> Nullable<Text>,
        email -> Nullable<Text>,
        address -> Nullable<Text>,
        location_id -> Nullable<Uuid>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    refresh_tokens (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        token -> Text,
        is_valid -> Nullable<Bool>,
        revoked_at -> Nullable<Timestamp>,
        expires_at -> Timestamp,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    report_comments (id) {
        id -> Uuid,
        report_id -> Nullable<Uuid>,
        user_id -> Nullable<Uuid>,
        content -> Text,
        parent_id -> Nullable<Uuid>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    report_history (id) {
        id -> Uuid,
        report_id -> Nullable<Uuid>,
        changed_by -> Nullable<Uuid>,
        status_from -> Nullable<Text>,
        status_to -> Nullable<Text>,
        notes -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    report_media (id) {
        id -> Uuid,
        report_id -> Nullable<Uuid>,
        media_type -> Text,
        media_url -> Text,
        caption -> Nullable<Text>,
        is_primary -> Nullable<Bool>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    reports (id) {
        id -> Uuid,
        reporter_id -> Nullable<Uuid>,
        anonymous_name -> Nullable<Text>,
        anonymous_phone -> Nullable<Text>,
        is_anonymous -> Nullable<Bool>,
        disaster_type_id -> Nullable<Int4>,
        title -> Text,
        description -> Nullable<Text>,
        location_id -> Nullable<Uuid>,
        address -> Nullable<Text>,
        impact_radius -> Nullable<Float8>,
        estimated_severity -> Nullable<Int4>,
        casualties -> Nullable<Int4>,
        injuries -> Nullable<Int4>,
        missing -> Nullable<Int4>,
        affected_people -> Nullable<Int4>,
        status -> Nullable<Text>,
        credibility_score -> Nullable<Float8>,
        validated_by -> Nullable<Uuid>,
        validation_notes -> Nullable<Text>,
        validation_date -> Nullable<Timestamp>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    resource_allocations (id) {
        id -> Uuid,
        resource_id -> Nullable<Uuid>,
        disaster_id -> Nullable<Uuid>,
        quantity -> Int4,
        allocated_by -> Nullable<Uuid>,
        status -> Nullable<Text>,
        notes -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    roles (id) {
        id -> Int4,
        #[max_length = 50]
        name -> Varchar,
        description -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    spatial_ref_sys (srid) {
        srid -> Int4,
        #[max_length = 256]
        auth_name -> Nullable<Varchar>,
        auth_srid -> Nullable<Int4>,
        #[max_length = 2048]
        srtext -> Nullable<Varchar>,
        #[max_length = 2048]
        proj4text -> Nullable<Varchar>,
    }
}

diesel::table! {
    user_roles (user_id, role_id) {
        user_id -> Uuid,
        role_id -> Int4,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Text,
        password_hash -> Text,
        email -> Text,
        phone -> Nullable<Text>,
        role_id -> Nullable<Int4>,
        full_name -> Nullable<Text>,
        address -> Nullable<Text>,
        profile_photo_url -> Nullable<Text>,
        bio -> Nullable<Text>,
        date_of_birth -> Nullable<Date>,
        #[max_length = 20]
        gender -> Nullable<Varchar>,
        is_verified -> Nullable<Bool>,
        is_active -> Nullable<Bool>,
        last_login -> Nullable<Timestamp>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    verification_codes (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        code -> Text,
        #[sql_name = "type"]
        #[max_length = 20]
        type_ -> Varchar,
        expires_at -> Timestamp,
        is_used -> Nullable<Bool>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Geography;

    volunteer_locations (id) {
        id -> Uuid,
        volunteer_id -> Nullable<Uuid>,
        geometry -> Nullable<Geography>,
        recorded_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    volunteer_tracking (id) {
        id -> Uuid,
        volunteer_id -> Nullable<Uuid>,
        report_id -> Nullable<Uuid>,
        status -> Nullable<Text>,
        notes -> Nullable<Text>,
        assigned_at -> Nullable<Timestamp>,
        arrived_at -> Nullable<Timestamp>,
        completed_at -> Nullable<Timestamp>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    volunteers (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        skills -> Nullable<Array<Nullable<Text>>>,
        certifications -> Nullable<Array<Nullable<Text>>>,
        availability -> Nullable<Bool>,
        availability_notes -> Nullable<Text>,
        experience_years -> Nullable<Int4>,
        specialization -> Nullable<Text>,
        current_location_id -> Nullable<Uuid>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    weather_data (id) {
        id -> Uuid,
        location_id -> Nullable<Uuid>,
        temperature -> Nullable<Float8>,
        humidity -> Nullable<Float8>,
        wind_speed -> Nullable<Float8>,
        wind_direction -> Nullable<Float8>,
        precipitation -> Nullable<Float8>,
        pressure -> Nullable<Float8>,
        weather_condition -> Nullable<Text>,
        recorded_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(auth_sessions -> users (user_id));
diesel::joinable!(disaster_analytics -> disasters (disaster_id));
diesel::joinable!(disaster_movements -> disasters (disaster_id));
diesel::joinable!(disaster_reports -> disasters (disaster_id));
diesel::joinable!(disaster_reports -> reports (report_id));
diesel::joinable!(disaster_zones -> disasters (disaster_id));
diesel::joinable!(disasters -> disaster_types (disaster_type_id));
diesel::joinable!(disasters -> locations (primary_location_id));
diesel::joinable!(emergency_resources -> locations (location_id));
diesel::joinable!(emergency_resources -> organizations (organization_id));
diesel::joinable!(evacuation_center_facilities -> evacuation_centers (evacuation_center_id));
diesel::joinable!(evacuation_centers -> locations (location_id));
diesel::joinable!(notifications -> users (user_id));
diesel::joinable!(organization_members -> organizations (organization_id));
diesel::joinable!(organization_members -> users (user_id));
diesel::joinable!(organizations -> locations (location_id));
diesel::joinable!(refresh_tokens -> users (user_id));
diesel::joinable!(report_comments -> reports (report_id));
diesel::joinable!(report_comments -> users (user_id));
diesel::joinable!(report_history -> reports (report_id));
diesel::joinable!(report_history -> users (changed_by));
diesel::joinable!(report_media -> reports (report_id));
diesel::joinable!(reports -> disaster_types (disaster_type_id));
diesel::joinable!(reports -> locations (location_id));
diesel::joinable!(resource_allocations -> disasters (disaster_id));
diesel::joinable!(resource_allocations -> emergency_resources (resource_id));
diesel::joinable!(resource_allocations -> users (allocated_by));
diesel::joinable!(user_roles -> roles (role_id));
diesel::joinable!(user_roles -> users (user_id));
diesel::joinable!(users -> roles (role_id));
diesel::joinable!(verification_codes -> users (user_id));
diesel::joinable!(volunteer_locations -> volunteers (volunteer_id));
diesel::joinable!(volunteer_tracking -> reports (report_id));
diesel::joinable!(volunteer_tracking -> volunteers (volunteer_id));
diesel::joinable!(volunteers -> locations (current_location_id));
diesel::joinable!(volunteers -> users (user_id));
diesel::joinable!(weather_data -> locations (location_id));

diesel::allow_tables_to_appear_in_same_query!(
    auth_sessions,
    disaster_analytics,
    disaster_movements,
    disaster_reports,
    disaster_types,
    disaster_zones,
    disasters,
    emergency_resources,
    evacuation_center_facilities,
    evacuation_centers,
    locations,
    notifications,
    organization_members,
    organizations,
    refresh_tokens,
    report_comments,
    report_history,
    report_media,
    reports,
    resource_allocations,
    roles,
    spatial_ref_sys,
    user_roles,
    users,
    verification_codes,
    volunteer_locations,
    volunteer_tracking,
    volunteers,
    weather_data,
);
