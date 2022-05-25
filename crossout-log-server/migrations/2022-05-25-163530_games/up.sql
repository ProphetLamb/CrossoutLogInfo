CREATE TABLE maps (
    id SERIAL PRIMARY KEY,
    name VARCHAR(64) NOT NULL
);
CREATE TABLE games (
    id SERIAL PRIMARY KEY,
    map_id SERIAL NOT NULL REFERENCES maps(id),
    start_ts TIMESTAMPTZ NOT NULL
);
CREATE TYPE win_reason AS ENUM (
    'best_of_three',
    'best_of_three_timer',
    'death_match',
    'death_match_timer',
    'domination',
    'domination_timer',
    'more_base_captured',
    'more_base_captured_timer',
    'more_cars_left',
    'more_cars_left_timer',
    'none'
);
CREATE TYPE finish_reason AS ENUM (
    'no_cars',
    'base_captured',
    'timer'
);
CREATE TABLE rounds (
    id SERIAL PRIMARY KEY,
    game_id SERIAL NOT NULL REFERENCES games(id),
    start_ts TIMESTAMPTZ NOT NULL,
    round_no SMALLINT NOT NULL,
    duration REAL NOT NULL,
    finish_reason finish_reason NOT NULL,
    win_reason win_reason NOT NULL,
    winning_team REAL NOT NULL
);
CREATE TABLE players(
    id SERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    name VARCHAR(64) NOT NULL
);
CREATE TABLE spawns(
    id SERIAL PRIMARY KEY,
    player_id SERIAL NOT NULL REFERENCES players(id),
    round_id SERIAL NOT NULL REFERENCES rounds(id),
    spawn_counter SMALLINT NOT NULL,
    player_no SMALLINT NOT NULL,
    team SMALLINT NOT NULL,
    bot SMALLINT NOT NULL,
    party BIGINT NOT NULL,
    session BIGINT NOT NULL,
    design BIGINT NOT NULL
);
