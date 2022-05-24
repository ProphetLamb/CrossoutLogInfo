use chrono::prelude::*;
use flagset::{flags, FlagSet};
use parse_display::{Display, FromStr};

#[derive(Debug, PartialEq)]
pub struct Entry {
    pub time_stamp: NaiveDateTime,
    pub message: Payload,
}

#[derive(Debug, PartialEq)]
pub enum Payload {
    LevelStart {
        level_no: usize,
        level_name: String,
        layout_mode: String,
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
        game_mode: String,
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
    #[derive(Display, FromStr)]
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

#[derive(Display, PartialEq, Eq, FromStr, Debug)]
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

#[derive(Display, PartialEq, Eq, FromStr, Debug)]
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

#[derive(Display, PartialEq, Eq, FromStr, Debug)]
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
