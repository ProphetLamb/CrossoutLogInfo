CREATE TABLE weapons (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL
);
CREATE TABLE kills (
    id SERIAL PRIMARY KEY,
    round_id SERIAL NOT NULL REFERENCES rounds(id),
    killer_id SERIAL NOT NULL REFERENCES spawns(id),
    victim_id SERIAL NOT NULL REFERENCES spawns(id)
);
CREATE TABLE assists (
    id SERIAL PRIMARY KEY,
    kill_id SERIAL NOT NULL REFERENCES kills(id),
    assistant_id SERIAL NOT NULL REFERENCES spawns(id),
    weapon_id SERIAL NOT NULL REFERENCES weapons(id),
    elapsed_sec REAL NOT NULL,
    damage_dealt REAL NOT NULL,
    damage_flags INTEGER NOT NULL
);
