use chrono::prelude::*;
use chrono::{NaiveDate, NaiveTime};
use flagset::{flags, FlagSet};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take;
use nom::bytes::complete::take_while;
use nom::character::complete::{digit1, hex_digit1};
use nom::combinator::opt;
use nom::combinator::{map_res, recognize};
use nom::sequence::tuple;
use nom::{AsChar, InputTakeAtPosition};
use parse_display::{Display, FromStr};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

fn parse_time<'a, E>(input: &'a str) -> nom::IResult<&'a str, NaiveTime, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseIntError>,
{
    let colon = nom::character::complete::char(':');
    let dot = nom::character::complete::char('.');

    let (input, hour) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = colon(input)?;
    let (input, min) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = colon(input)?;
    let (input, sec) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = dot(input)?;
    let (input, milli) = map_res(recognize(digit1), str::parse)(input)?;

    Ok((input, NaiveTime::from_hms_milli(hour, min, sec, milli)))
}

pub fn parse_entry<'a, E>(
    log_file_date: NaiveDate,
) -> impl FnMut(&'a str) -> nom::IResult<&'a str, Entry, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseIntError>
        + nom::error::FromExternalError<&'a str, std::num::ParseFloatError>
        + nom::error::FromExternalError<&'a str, parse_display::ParseError>,
{
    move |input| {
        let (input, time_stamp) = parse_time(input)?;
        let (input, _) = tag("| ")(input)?;
        let (input, message) = parse_message(input)?;
        let time_stamp = log_file_date.and_time(time_stamp);
        Ok((
            input,
            Entry {
                time_stamp,
                message,
            },
        ))
    }
}

fn parse_message<'a, E>(input: &'a str) -> nom::IResult<&'a str, Payload, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseIntError>
        + nom::error::FromExternalError<&'a str, std::num::ParseFloatError>
        + nom::error::FromExternalError<&'a str, parse_display::ParseError>,
{
    alt((
        parse_level_start,
        parse_test_start,
        parse_test_finish,
        parse_spawn_player,
        parse_game_start,
        parse_round_finish,
        parse_game_finish,
        parse_battle_start,
        parse_player_info,
        parse_score,
        parse_damage,
        parse_stripe,
        parse_kill,
        parse_assist,
    ))(input)
}

fn parse_level_start<'a, E>(input: &'a str) -> nom::IResult<&'a str, Payload, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseIntError>
        + nom::error::FromExternalError<&'a str, parse_display::ParseError>,
{
    let (input, _) = tag("====== starting level ")(input)?;
    let (input, level_no) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = tag(": '")(input)?;
    let (input, level_name) = take_while(|c| c != '\'')(input)?;
    let (input, _) = tag("' ")(input)?;
    let (input, game_mode) = parse_game_mode(take_while(not_ws))(input)?;
    let (input, _) = tag(" ======")(input)?;
    Ok((
        input,
        Payload::LevelStart {
            level_no,
            level_name: level_name.to_string(),
            game_mode,
        },
    ))
}

fn parse_game_mode<'a, P, E>(
    parser: P,
) -> impl FnOnce(&'a str) -> nom::IResult<&'a str, GameMode, E>
where
    P: nom::Parser<&'a str, &'a str, E>,
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, parse_display::ParseError>,
{
    move |input| {
        let (input, game_mode) = recognize(parser)(input)?;
        match if game_mode.is_empty() {
            Ok(GameMode::None)
        } else {
            GameMode::from_str(game_mode)
        } {
            Ok(game_mode) => Ok((input, game_mode)),
            Err(e) => Err(nom::Err::Error(E::from_external_error(
                input,
                nom::error::ErrorKind::MapRes,
                e,
            ))),
        }
    }
}

fn parse_test_start<'a, E>(input: &'a str) -> nom::IResult<&'a str, Payload, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseIntError>,
{
    let (input, _) = tag("====== TestDrive started ======")(input)?;
    Ok((input, Payload::TestStart))
}

fn parse_test_finish<'a, E>(input: &'a str) -> nom::IResult<&'a str, Payload, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseIntError>,
{
    let (input, _) = tag("====== TestDrive finish ======")(input)?;
    Ok((input, Payload::TestFinish))
}

fn parse_spawn_player<'a, E>(input: &'a str) -> nom::IResult<&'a str, Payload, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseIntError>,
{
    let (input, _) = tag("Spawn player ")(input)?;
    let (input, player_no) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = tag(" [")(input)?;
    let (input, nick_name) = take_while(|c| c != ']')(input)?;
    let (input, _) = tag("], team ")(input)?;
    let (input, team) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = tag(", spawnCounter ")(input)?;
    let (input, spawn_counter) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = tag(" , designHash: ")(input)?;
    let (input, design_hash) = map_res(recognize(hex_digit1), from_hex)(input)?;
    let (input, _) = tag(".")(input)?;
    Ok((
        input,
        Payload::SpawnPlayer {
            player_no,
            nick_name: nick_name.to_string(),
            team,
            spawn_counter,
            design_hash,
        },
    ))
}

fn from_hex(input: &str) -> Result<usize, std::num::ParseIntError> {
    usize::from_str_radix(input, 16)
}

fn parse_game_start<'a, E>(input: &'a str) -> nom::IResult<&'a str, Payload, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseIntError>
        + nom::error::FromExternalError<&'a str, parse_display::ParseError>,
{
    let (input, _) = tag("===== Gameplay '")(input)?;
    let (input, game_mode) = parse_game_mode(take_while(|c| c != '\''))(input)?;
    let (input, _) = tag("' started, map '")(input)?;
    let (input, map) = take_while(|c| c != '\'')(input)?;
    let (input, _) = tag("' ======")(input)?;
    Ok((
        input,
        Payload::GameStart {
            game_mode,
            map: map.to_string(),
        },
    ))
}

fn parse_round_finish<'a, E>(input: &'a str) -> nom::IResult<&'a str, Payload, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseIntError>
        + nom::error::FromExternalError<&'a str, std::num::ParseFloatError>
        + nom::error::FromExternalError<&'a str, parse_display::ParseError>,
{
    let (input, _) = tag("===== Best Of N round ")(input)?;
    let (input, round) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = tag(" finish, reason: ")(input)?;
    let (input, finish_reason) = map_res(recognize(take_while(|c| c != ',')), str::parse)(input)?;
    let (input, _) = tag(", winner team ")(input)?;
    let (input, winning_team) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = tag(", win reason: ")(input)?;
    let (input, win_reason) = map_res(recognize(take_while(|c| c != ',')), str::parse)(input)?;
    let (input, _) = tag(", battle time: ")(input)?;
    let (input, duration_sec) = map_res(recognize(float_digit1), str::parse)(input)?;
    let (input, _) = tag(" sec =====")(input)?;
    Ok((
        input,
        Payload::GameFinish {
            round,
            finish_reason,
            winning_team,
            win_reason,
            duration_sec,
        },
    ))
}

fn parse_game_finish<'a, E>(input: &'a str) -> nom::IResult<&'a str, Payload, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseIntError>
        + nom::error::FromExternalError<&'a str, std::num::ParseFloatError>
        + nom::error::FromExternalError<&'a str, parse_display::ParseError>,
{
    let (input, _) = tag("===== Gameplay finish, reason: ")(input)?;
    let (input, finish_reason) = map_res(recognize(take_while(not_ws_comma)), str::parse)(input)?;
    let (input, _) = tag(", winner team ")(input)?;
    let (input, winning_team) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = tag(", win reason: ")(input)?;
    let (input, win_reason) = map_res(recognize(take_while(not_ws_comma)), str::parse)(input)?;
    let (input, _) = tag(", battle time: ")(input)?;
    let (input, duration_sec) = map_res(recognize(float_digit1), str::parse)(input)?;
    let (input, _) = tag(" sec =====")(input)?;
    Ok((
        input,
        Payload::GameFinish {
            round: 0,
            finish_reason,
            winning_team,
            win_reason,
            duration_sec,
        },
    ))
}

#[inline]
fn float_digit1<'a, E>(input: &'a str) -> nom::IResult<&'a str, &'a str, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseFloatError>,
{
    input.split_at_position1_complete(
        |item| !(item.is_dec_digit() || item == '.'),
        nom::error::ErrorKind::Digit,
    )
}

fn parse_battle_start<'a, E>(input: &'a str) -> nom::IResult<&'a str, Payload, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseIntError>,
{
    let (input, _) = tag("Active battle started.")(input)?;
    Ok((input, Payload::BattleStart))
}

fn parse_player_info<'a, E>(input: &'a str) -> nom::IResult<&'a str, Payload, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseIntError>,
{
    let (input, _) = tuple((
        take_while(char::is_whitespace),
        tag("player"),
        take_while(char::is_whitespace),
    ))(input)?;
    let (input, player_no) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = tag(", uid ")(input)?;
    let (input, user_id) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = tag(", party ")(input)?;
    let (input, party_id) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = tag(", nickname: ")(input)?;
    let (input, nick_name) = take_while(not_ws_comma)(input)?;
    let (input, _) = take_while(char::is_whitespace)(input)?;
    let (input, _) = tag(", team: ")(input)?;
    let (input, team) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = tag(", bot: ")(input)?;
    let (input, bot) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = tag(", ur: ")(input)?;
    let (input, session) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = tag(", mmHash: ")(input)?;
    let (input, design_hash) = map_res(recognize(hex_digit1), from_hex)(input)?;
    Ok((
        input,
        Payload::PlayerInfo {
            player_no,
            user_id,
            party_id,
            nick_name: nick_name.to_string(),
            team,
            bot,
            session,
            design_hash,
        },
    ))
}

fn parse_score<'a, E>(input: &'a str) -> nom::IResult<&'a str, Payload, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseIntError>
        + nom::error::FromExternalError<&'a str, parse_display::ParseError>,
{
    let (input, _) = tuple((
        tag("Score:"),
        take_while(char::is_whitespace),
        tag("player:"),
        take_while(char::is_whitespace),
    ))(input)?;
    let (input, player_no) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = tuple((
        tag(","),
        take_while(char::is_whitespace),
        tag("nick:"),
        take_while(char::is_whitespace),
    ))(input)?;
    let (input, nick_name) = take_while(|c: char| c != ',')(input)?;
    let (input, _) = tuple((
        tag(","),
        take_while(char::is_whitespace),
        tag("Got:"),
        take_while(char::is_whitespace),
    ))(input)?;
    let (input, points) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = tuple((
        tag(","),
        take_while(char::is_whitespace),
        tag("reason:"),
        take_while(char::is_whitespace),
    ))(input)?;
    let (input, reason) = map_res(recognize(take_while(|_| true)), str::parse)(input)?;
    Ok((
        input,
        Payload::Score {
            player_no,
            nick_name: nick_name.to_string(),
            points,
            reason,
        },
    ))
}

fn parse_damage<'a, E>(input: &'a str) -> nom::IResult<&'a str, Payload, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseIntError>
        + nom::error::FromExternalError<&'a str, std::num::ParseFloatError>
        + nom::error::FromExternalError<&'a str, parse_display::ParseError>,
{
    let (input, _) = tag("Damage. Victim: ")(input)?;
    let (input, victim) = take_while(not_ws_comma)(input)?;
    let (input, _) = tuple((take_while(char::is_whitespace), tag(", attacker: ")))(input)?;
    let (input, attacker) = take_while(not_ws_comma)(input)?;
    let (input, _) = tuple((take_while(char::is_whitespace), tag(", weapon '")))(input)?;
    let (input, weapon) = take_while(|c| c != '\'')(input)?;
    let (input, _) = tag("', damage: ")(input)?;
    let (input, damage) = map_res(recognize(float_digit1), str::parse)(input)?;
    let (input, _) = take_while(char::is_whitespace)(input)?;
    let (input, flags) = parse_damage_flags(input)?;
    Ok((
        input,
        Payload::Damage {
            victim: victim.to_string(),
            attacker: attacker.to_string(),
            weapon: weapon.to_string(),
            damage,
            flags,
        },
    ))
}

fn not_ws_comma(c: char) -> bool {
    !(c.is_whitespace() || c == ',')
}

fn not_ws(c: char) -> bool {
    !c.is_whitespace()
}

fn parse_damage_flags<'a, E>(mut input: &'a str) -> nom::IResult<&'a str, FlagSet<DamageFlag>, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, parse_display::ParseError>,
{
    let mut flags = FlagSet::<DamageFlag>::default();
    while !input.is_empty() {
        let (remainder, flag) =
            map_res(take_while(|c: char| c != '|'), str::parse::<DamageFlag>)(input)?;
        let (remainder, _) = opt(take(1usize))(remainder)?;
        input = remainder;
        flags |= flag;
    }
    Ok((input, flags))
}

fn parse_stripe<'a, E>(input: &'a str) -> nom::IResult<&'a str, Payload, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseIntError>,
{
    let (input, _) = tag("Stripe '")(input)?;
    let (input, name) = take_while(|c| c != '\'')(input)?;
    let (input, _) = tag("' value increased by ")(input)?;
    let (input, value) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = tag(" for player ")(input)?;
    let (input, player_no) = map_res(recognize(digit1), str::parse)(input)?;
    let (input, _) = tag(" [")(input)?;
    let (input, nick_name) = take_while(|c| c != ']')(input)?;
    let (input, _) = tag("].")(input)?;
    Ok((
        input,
        Payload::Stripe {
            name: name.to_string(),
            value,
            player_no,
            nick_name: nick_name.to_string(),
        },
    ))
}

fn parse_kill<'a, E>(input: &'a str) -> nom::IResult<&'a str, Payload, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseIntError>,
{
    let (input, _) = tag("Kill. Victim: ")(input)?;
    let (input, victim) = take_while(not_ws)(input)?;
    let (input, _) = tuple((take_while(char::is_whitespace), tag("killer: ")))(input)?;
    let (input, killer) = take_while(not_ws)(input)?;
    Ok((
        input,
        Payload::Kill {
            victim: victim.to_string(),
            killer: killer.to_string(),
        },
    ))
}

fn parse_assist<'a, E>(input: &'a str) -> nom::IResult<&'a str, Payload, E>
where
    E: nom::error::ParseError<&'a str>
        + nom::error::FromExternalError<&'a str, std::num::ParseIntError>
        + nom::error::FromExternalError<&'a str, std::num::ParseFloatError>
        + nom::error::FromExternalError<&'a str, parse_display::ParseError>,
{
    let (input, _) = tuple((take_while(char::is_whitespace), tag("assist by ")))(input)?;
    let (input, assistant) = take_while(not_ws)(input)?;
    let (input, _) = tuple((take_while(char::is_whitespace), tag("weapon: '")))(input)?;
    let (input, weapon) = take_while(|c| c != '\'')(input)?;
    let (input, _) = tag("', ")(input)?;
    let (input, elapsed_sec) = map_res(recognize(float_digit1), str::parse)(input)?;
    let (input, _) = tag(" sec ago, damage: ")(input)?;
    let (input, damage_dealt) = map_res(recognize(float_digit1), str::parse)(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, flags) = parse_damage_flags(input)?;
    Ok((
        input,
        Payload::Assist {
            assistant: assistant.to_string(),
            weapon: weapon.to_string(),
            elapsed_sec,
            damage_dealt,
            flags,
        },
    ))
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Entry {
    pub time_stamp: NaiveDateTime,
    pub message: Payload,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Payload {
    LevelStart {
        level_no: usize,
        level_name: String,
        game_mode: GameMode,
    },
    TestStart,
    TestFinish,
    SpawnPlayer {
        player_no: u8,
        nick_name: String,
        team: u8,
        spawn_counter: usize,
        design_hash: usize,
    },
    GameStart {
        game_mode: GameMode,
        map: String,
    },
    GameFinish {
        round: u8,
        finish_reason: FinishReason,
        winning_team: u8,
        win_reason: WinReason,
        duration_sec: f32,
    },
    BattleStart,
    PlayerInfo {
        player_no: u8,
        user_id: usize,
        party_id: usize,
        nick_name: String,
        team: u8,
        bot: u8,
        session: usize,
        design_hash: usize,
    },
    Score {
        player_no: u8,
        nick_name: String,
        points: usize,
        reason: ScoreReason,
    },
    Damage {
        victim: String,
        attacker: String,
        weapon: String,
        damage: f32,
        flags: FlagSet<DamageFlag>,
    },
    Stripe {
        name: String,
        value: usize,
        player_no: u8,
        nick_name: String,
    },
    Kill {
        victim: String,
        killer: String,
    },
    Assist {
        assistant: String,
        weapon: String,
        elapsed_sec: f32,
        damage_dealt: f32,
        flags: FlagSet<DamageFlag>,
    },
}

flags! {
    #[derive(Serialize, Deserialize, Display, FromStr)]
    pub enum DamageFlag: usize {
        /// DMG_GENERIC: Indirect damage or other.
        #[display("DMG_GENERIC")]
        Generic,
        /// DMG_DIRECT: Damage resulting from a direct projectile hit.
        #[display("DMG_DIRECT")]
        Direct,
        /// DMG_BLAST: Damage resulting only from explosion, no projectile hit.
        #[display("DMG_BLAST")]
        Blast,
        /// DMG_ENERGY: Damage resulting from energy weapons.
        #[display("DMG_ENERGY")]
        Energy,
        /// DMG_COLLISION: Damage resulting from a collision.
        #[display("DMG_COLLISION")]
        Collision,
        /// DMG_FLAME: Damage resulting from flames. Such as FireBug flames or Mandrake puddles. Always Continuous.
        #[display("DMG_FLAME")]
        Flame,
        /// CONTINUOUS: Stream of damage, from weapons such as the Spark or FireBug.
        #[display("CONTINUOUS")]
        Continuous,
        /// CONTACT: Damage involving touch. Not necessarily collision damage, but usually accompanied by collision damage.
        #[display("CONTACT")]
        Contact,
        /// PIERCING: Damage that pierces parts from the Scorpion.
        #[display("PIERCING")]
        Piercing,
        /// PIERCING_TRANSITION: Damage that pierces, while loosing damage per pierced layer.
        #[display("PIERCING_TRANSITION")]
        PiercingTransition,
        /// DIRECT_PIERCING: Damage that pierces, and ignores damage from pierced layers.
        #[display("DIRECT_PIERCING")]
        DirectPiercing,
        /// IGNORE_DAMAGE_SCALE: Mostly explosive damage
        #[display("IGNORE_DAMAGE_SCALE")]
        IgnoreDamageScale,
        /// SUICIDE: Damage by self-detonation, also applies to mine and fuse drone damage.
        #[display("SUICIDE")]
        Suicide,
        /// SUICIDE_DESPAWN: Self-destruction resulted in death. Nearly always accompanies SUICIDE, but for self-destructing with the werewolf cabin.
        #[display("SUICIDE_DESPAWN")]
        SuicideDespawn,
        /// HUD_IMPORTANT: Yellow damage
        #[display("HUD_IMPORTANT")]
        Important,
        /// HUD_HIDDEN: Hidden damage
        #[display("HUD_HIDDEN")]
        Hidden,
        /// HIGH_CAR_RESIST: Lower damage dealt then expected, due to high cabin resistance.
        #[display("HIGH_CAR_RESIST")]
        HighResist,
    }
}

#[derive(Serialize, Deserialize, Display, PartialEq, Eq, FromStr, Debug)]
pub enum ScoreReason {
    /// FIRST_DAMAGE: First Blood
    #[display("FIRST_DAMAGE")]
    FirstDamage,
    /// PART_DETACH: Parts destroyed
    #[display("PART_DETACH")]
    PartDetach,
    /// KILL: Entity killed. Not just players and buts, but also drones and mines.
    #[display("KILL")]
    Kill,
    /// INTERCEPT: Incoming missiles destroyed, often with the Spark.
    #[display("INTERCEPT")]
    Intercept,
    /// POINT_CAPTURE: Captured a point.
    #[display("POINT_CAPTURE")]
    PointCapture,
    /// SHIELD: Absorbed damage with a shield.
    #[display("SHIELD")]
    Shield,
}

#[derive(Serialize, Deserialize, Display, PartialEq, Eq, FromStr, Debug)]
pub enum FinishReason {
    /// no_cars: All vehicles are eliminated
    #[display("no_cars")]
    NoCars,
    /// base_captured: Base/s are captured before the timer runs out.
    #[display("base_captured")]
    BaseCaptured,
    /// timer: The timer runs out. Also the case, if only one base is captured, in certain game modes.
    #[display("timer")]
    Timer,
}

#[derive(Serialize, Deserialize, Display, PartialEq, Eq, FromStr, Debug)]
pub enum WinReason {
    /// NONE: Quit the game
    #[display("NONE")]
    None,
    /// MORE_CARS_LEFT: All enemies eliminated
    #[display("MORE_CARS_LEFT")]
    MoreCarsLeft,
    /// MORE_BASE_CAPTURED: Captured the majority of the bases. Ended before timer runs out.
    #[display("MORE_BASE_CAPTURED")]
    MoreBaseCaptured,
    /// MORE_BASE_CAPTURED_TIMER: Captured more bases then the enemy, and the timer runs out.
    #[display("MORE_BASE_CAPTURED_TIMER")]
    MoreBaseCapturedTimer,
    /// DOMINATION: Captured the central base.
    #[display("DOMINATION")]
    Domination,
    /// DEATMATCH_TIMER: The timer ran out in PvP, wile enemies were left. And the XO devs cant type for shit.
    #[display("DEATMATCH_TIMER")]
    DeathMatchTimer,
    /// BEST_OF_THREE: The clan-wars battle is decided
    #[display("BEST_OF_THREE")]
    BestOfThree,
}

#[derive(Serialize, Deserialize, Display, PartialEq, Eq, FromStr, Debug)]
pub enum GameMode {
    #[display("Unknown")]
    None,
    #[display("Conquer")]
    Conquer,
    #[display("ConquerBestOf3")]
    ConquerBestOf3,
    #[display("Assault")]
    Assault,
    #[display("AssaultBestOf3")]
    AssaultBestOf3,
    #[display("Domination")]
    Domination,
    #[display("Brawl_BDCrossout")]
    BrawlBDCrossout,
}
