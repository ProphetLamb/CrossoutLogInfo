table! {
    assists (id) {
        id -> Int4,
        kill_id -> Int4,
        assistant_id -> Int4,
        weapon_id -> Int4,
        elapsed_sec -> Float4,
        damage_dealt -> Float4,
        damage_flags -> Int4,
    }
}

table! {
    badges (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    games (id) {
        id -> Int4,
        map_id -> Int4,
        start_ts -> Timestamptz,
    }
}

table! {
    kills (id) {
        id -> Int4,
        round_id -> Int4,
        killer_id -> Int4,
        victim_id -> Int4,
    }
}

table! {
    maps (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    players (id) {
        id -> Int4,
        user_id -> Int8,
        name -> Varchar,
    }
}

table! {
    rounds (id) {
        id -> Int4,
        game_id -> Int4,
        start_ts -> Timestamptz,
        round_no -> Int2,
        duration -> Float4,
        finish_reason -> Int2,
        win_reason -> Int2,
        winning_team -> Float4,
    }
}

table! {
    scores (id) {
        id -> Int4,
        spawn_id -> Int4,
        value -> Float4,
        reason -> Int2,
    }
}

table! {
    spawns (id) {
        id -> Int4,
        player_id -> Int4,
        round_id -> Int4,
        spawn_counter -> Int2,
        player_no -> Int2,
        team -> Int2,
        bot -> Int2,
        party -> Int8,
        session -> Int8,
        design -> Int8,
    }
}

table! {
    stripes (id) {
        id -> Int4,
        badge_id -> Int4,
        spawn_id -> Int4,
        value -> Float4,
    }
}

table! {
    weapons (id) {
        id -> Int4,
        name -> Varchar,
    }
}

joinable!(assists -> kills (kill_id));
joinable!(assists -> spawns (assistant_id));
joinable!(assists -> weapons (weapon_id));
joinable!(games -> maps (map_id));
joinable!(kills -> rounds (round_id));
joinable!(rounds -> games (game_id));
joinable!(scores -> spawns (spawn_id));
joinable!(spawns -> players (player_id));
joinable!(spawns -> rounds (round_id));
joinable!(stripes -> badges (badge_id));
joinable!(stripes -> spawns (spawn_id));

allow_tables_to_appear_in_same_query!(
    assists,
    badges,
    games,
    kills,
    maps,
    players,
    rounds,
    scores,
    spawns,
    stripes,
    weapons,
);
