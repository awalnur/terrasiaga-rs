-- This file reverts the sample data added in up.sql
-- Created on: 2025-07-21

-- Note: DELETE statements should be in reverse order of the INSERT statements
-- to properly handle foreign key constraints

-- Delete resource allocations
DELETE FROM resource_allocations WHERE id LIKE 'alloc%';

-- Delete emergency resources
DELETE FROM emergency_resources WHERE id LIKE 'resource%';

-- Delete evacuation center facilities
DELETE FROM evacuation_center_facilities WHERE id LIKE 'facility%';

-- Delete evacuation centers
DELETE FROM evacuation_centers WHERE id LIKE 'evac%';

-- Delete report comments
DELETE FROM report_comments WHERE id LIKE 'comment%';

-- Delete report media
DELETE FROM report_media WHERE id LIKE 'media%';

-- Delete disaster reports (linking table)
DELETE FROM disaster_reports WHERE disaster_id LIKE 'aabbccdd-1111%';

-- Delete disasters
DELETE FROM disasters WHERE id LIKE 'aabbccdd-1111%';

-- Delete reports
DELETE FROM reports WHERE id LIKE '11223344-5566%';

-- Delete volunteers
DELETE FROM volunteers WHERE id LIKE 'aabbccdd-eeff%';

-- Delete user roles
DELETE FROM user_roles WHERE user_id IN (
    '11111111-1111-1111-1111-111111111111',
    '22222222-2222-2222-2222-222222222222',
    '33333333-3333-3333-3333-333333333333',
    '44444444-4444-4444-4444-444444444444',
    '55555555-5555-5555-5555-555555555555',
    '66666666-6666-6666-6666-666666666666',
    '77777777-7777-7777-7777-777777777777',
    '88888888-8888-8888-8888-888888888888',
    '99999999-9999-9999-9999-999999999999',
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
);

-- Delete organizations
DELETE FROM organizations WHERE id LIKE '%qqqqq%' OR id LIKE '%rrrr%' OR id LIKE '%ssss%' OR id LIKE '%tttt%' OR id LIKE '%uuuu%' OR id LIKE '%vvvv%' OR id LIKE '%wwww%' OR id LIKE '%xxxx%' OR id LIKE '%yyyy%' OR id LIKE '%zzzz%';

-- Delete locations
DELETE FROM locations WHERE id LIKE '%bbbb%' OR id LIKE '%cccc%' OR id LIKE '%dddd%' OR id LIKE '%eeee%' OR id LIKE '%ffff%' OR id LIKE '%gggg%' OR id LIKE '%hhhh%' OR id LIKE '%iiii%' OR id LIKE '%jjjj%' OR id LIKE '%kkkk%' OR id LIKE '%llll%' OR id LIKE '%mmmm%' OR id LIKE '%nnnn%' OR id LIKE '%oooo%' OR id LIKE '%pppp%';

-- Delete disaster types
DELETE FROM disaster_types WHERE id BETWEEN 1 AND 10;

-- Delete users
DELETE FROM users WHERE id IN (
    '11111111-1111-1111-1111-111111111111',
    '22222222-2222-2222-2222-222222222222',
    '33333333-3333-3333-3333-333333333333',
    '44444444-4444-4444-4444-444444444444',
    '55555555-5555-5555-5555-555555555555',
    '66666666-6666-6666-6666-666666666666',
    '77777777-7777-7777-7777-777777777777',
    '88888888-8888-8888-8888-888888888888',
    '99999999-9999-9999-9999-999999999999',
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
);