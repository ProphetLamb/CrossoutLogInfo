CREATE TYPE score_reason AS ENUM (
    'first_damage',
    'part_detach',
    'kill',
    'intercept',
    'point_capture',
    'shield'
);
CREATE TABLE scores (
    id SERIAL PRIMARY KEY,
    spawn_id SERIAL NOT NULL REFERENCES spawns(id),
    value REAL NOT NULL,
    reason score_reason NOT NULL
);
CREATE TABLE badges (
    id SERIAL PRIMARY KEY,
    name VARCHAR(512) NOT NULL
);
CREATE TABLE stripes (
    id SERIAL PRIMARY KEY,
    badge_id SERIAL NOT NULL REFERENCES badges(id),
    spawn_id SERIAL NOT NULL REFERENCES spawns(id),
    value REAL NOT NULL
);
