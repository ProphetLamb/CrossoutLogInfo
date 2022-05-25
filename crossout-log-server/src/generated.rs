use wundergraph::query_builder::types::{HasMany, HasOne};
use wundergraph::scalar::WundergraphScalarValue;
use wundergraph::WundergraphEntity;

use crate::schema::*;

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "assists"]
#[primary_key(id)]
pub struct Assist {
    id: i32,
    kill_id: HasOne<i32, Kill>,
    assistant_id: HasOne<i32, Spawn>,
    weapon_id: HasOne<i32, Weapon>,
    elapsed_sec: f32,
    damage_dealt: f32,
    damage_flags: i32,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "badges"]
#[primary_key(id)]
pub struct Badge {
    id: i32,
    name: String,
    stripes: HasMany<Stripe, stripes::badge_id>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "games"]
#[primary_key(id)]
pub struct Game {
    id: i32,
    map_id: HasOne<i32, Map>,
    start_ts: chrono::DateTime<chrono::offset::Utc>,
    rounds: HasMany<Round, rounds::game_id>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "kills"]
#[primary_key(id)]
pub struct Kill {
    id: i32,
    round_id: HasOne<i32, Round>,
    killer_id: i32,
    victim_id: i32,
    assists: HasMany<Assist, assists::kill_id>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "maps"]
#[primary_key(id)]
pub struct Map {
    id: i32,
    name: String,
    games: HasMany<Game, games::map_id>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "players"]
#[primary_key(id)]
pub struct Player {
    id: i32,
    user_id: i64,
    name: String,
    spawns: HasMany<Spawn, spawns::player_id>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "rounds"]
#[primary_key(id)]
pub struct Round {
    id: i32,
    game_id: HasOne<i32, Game>,
    start_ts: chrono::DateTime<chrono::offset::Utc>,
    round_no: i16,
    duration: f32,
    finish_reason: i16,
    win_reason: i16,
    winning_team: f32,
    kills: HasMany<Kill, kills::round_id>,
    spawns: HasMany<Spawn, spawns::round_id>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "scores"]
#[primary_key(id)]
pub struct Score {
    id: i32,
    spawn_id: HasOne<i32, Spawn>,
    value: f32,
    reason: i16,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "spawns"]
#[primary_key(id)]
pub struct Spawn {
    id: i32,
    player_id: HasOne<i32, Player>,
    round_id: HasOne<i32, Round>,
    spawn_counter: i16,
    player_no: i16,
    team: i16,
    bot: i16,
    party: i64,
    session: i64,
    design: i64,
    assists: HasMany<Assist, assists::assistant_id>,
    scores: HasMany<Score, scores::spawn_id>,
    stripes: HasMany<Stripe, stripes::spawn_id>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "stripes"]
#[primary_key(id)]
pub struct Stripe {
    id: i32,
    badge_id: HasOne<i32, Badge>,
    spawn_id: HasOne<i32, Spawn>,
    value: f32,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "weapons"]
#[primary_key(id)]
pub struct Weapon {
    id: i32,
    name: String,
    assists: HasMany<Assist, assists::weapon_id>,
}



wundergraph::query_object!{
    Query {
        Assist,
        Badge,
        Game,
        Kill,
        Map,
        Player,
        Round,
        Score,
        Spawn,
        Stripe,
        Weapon,
    }
}


#[derive(Insertable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "assists"]
pub struct NewAssist {
    elapsed_sec: f32,
    damage_dealt: f32,
    damage_flags: i32,
}

#[derive(AsChangeset, Identifiable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "assists"]
#[primary_key(id)]
pub struct AssistChangeset {
    id: i32,
    kill_id: i32,
    assistant_id: i32,
    weapon_id: i32,
    elapsed_sec: f32,
    damage_dealt: f32,
    damage_flags: i32,
}

#[derive(Insertable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "badges"]
pub struct NewBadge {
    name: String,
}

#[derive(AsChangeset, Identifiable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "badges"]
#[primary_key(id)]
pub struct BadgeChangeset {
    id: i32,
    name: String,
}

#[derive(Insertable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "games"]
pub struct NewGame {
    start_ts: chrono::DateTime<chrono::offset::Utc>,
}

#[derive(AsChangeset, Identifiable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "games"]
#[primary_key(id)]
pub struct GameChangeset {
    id: i32,
    map_id: i32,
    start_ts: chrono::DateTime<chrono::offset::Utc>,
}

#[derive(Insertable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "kills"]
pub struct NewKill {
    id: i32,
    round_id: i32,
    killer_id: i32,
    victim_id: i32,
}

#[derive(AsChangeset, Identifiable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "kills"]
#[primary_key(id)]
pub struct KillChangeset {
    id: i32,
    round_id: i32,
    killer_id: i32,
    victim_id: i32,
}

#[derive(Insertable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "maps"]
pub struct NewMap {
    name: String,
}

#[derive(AsChangeset, Identifiable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "maps"]
#[primary_key(id)]
pub struct MapChangeset {
    id: i32,
    name: String,
}

#[derive(Insertable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "players"]
pub struct NewPlayer {
    user_id: i64,
    name: String,
}

#[derive(AsChangeset, Identifiable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "players"]
#[primary_key(id)]
pub struct PlayerChangeset {
    id: i32,
    user_id: i64,
    name: String,
}

#[derive(Insertable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "rounds"]
pub struct NewRound {
    start_ts: chrono::DateTime<chrono::offset::Utc>,
    round_no: i16,
    duration: f32,
    finish_reason: i16,
    win_reason: i16,
    winning_team: f32,
}

#[derive(AsChangeset, Identifiable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "rounds"]
#[primary_key(id)]
pub struct RoundChangeset {
    id: i32,
    game_id: i32,
    start_ts: chrono::DateTime<chrono::offset::Utc>,
    round_no: i16,
    duration: f32,
    finish_reason: i16,
    win_reason: i16,
    winning_team: f32,
}

#[derive(Insertable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "scores"]
pub struct NewScore {
    value: f32,
    reason: i16,
}

#[derive(AsChangeset, Identifiable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "scores"]
#[primary_key(id)]
pub struct ScoreChangeset {
    id: i32,
    spawn_id: i32,
    value: f32,
    reason: i16,
}

#[derive(Insertable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "spawns"]
pub struct NewSpawn {
    spawn_counter: i16,
    player_no: i16,
    team: i16,
    bot: i16,
    party: i64,
    session: i64,
    design: i64,
}

#[derive(AsChangeset, Identifiable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "spawns"]
#[primary_key(id)]
pub struct SpawnChangeset {
    id: i32,
    player_id: i32,
    round_id: i32,
    spawn_counter: i16,
    player_no: i16,
    team: i16,
    bot: i16,
    party: i64,
    session: i64,
    design: i64,
}

#[derive(Insertable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "stripes"]
pub struct NewStripe {
    value: f32,
}

#[derive(AsChangeset, Identifiable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "stripes"]
#[primary_key(id)]
pub struct StripeChangeset {
    id: i32,
    badge_id: i32,
    spawn_id: i32,
    value: f32,
}

#[derive(Insertable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "weapons"]
pub struct NewWeapon {
    name: String,
}

#[derive(AsChangeset, Identifiable, juniper::GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "weapons"]
#[primary_key(id)]
pub struct WeaponChangeset {
    id: i32,
    name: String,
}

wundergraph::mutation_object!{
    Mutation{
        Assist(insert = NewAssist, update = AssistChangeset, ),
        Badge(insert = NewBadge, update = BadgeChangeset, ),
        Game(insert = NewGame, update = GameChangeset, ),
        Kill(insert = NewKill, update = KillChangeset, ),
        Map(insert = NewMap, update = MapChangeset, ),
        Player(insert = NewPlayer, update = PlayerChangeset, ),
        Round(insert = NewRound, update = RoundChangeset, ),
        Score(insert = NewScore, update = ScoreChangeset, ),
        Spawn(insert = NewSpawn, update = SpawnChangeset, ),
        Stripe(insert = NewStripe, update = StripeChangeset, ),
        Weapon(insert = NewWeapon, update = WeaponChangeset, ),
    }
}

