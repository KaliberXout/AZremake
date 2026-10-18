#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Kind {
    /// A missile at the nearest enemies (or instant if the spell has no speed).
    Bolt,
    /// An instant hit on the nearest enemies in range.
    Strike,
    /// A close hit on the nearest enemies in melee reach.
    Melee,
    /// Everything in a radius around the hero.
    Nova,
    /// Everything in a cone toward the nearest enemy.
    Cone,
    /// A ground area at the densest nearby enemy, ticking for `duration`.
    Ground,
    /// A ground area under the hero, ticking for `duration`.
    GroundSelf,
    /// Jumps from enemy to enemy.
    Chain,
    /// A damage-over-time on several enemies.
    Dot,
    /// Always on: pulses every `cooldown` around the hero, with the spell's state visual on him.
    Aura,
    /// Always on: zaps one enemy close to the hero every `cooldown`.
    Orbit,
    /// Absorbs `damage` for `duration`.
    Shield,
    /// Immune for `duration`.
    Immune,
    /// Returns `damage` to every attacker for `duration`.
    Thorns,
    /// Heals `damage` over `duration`.
    Heal,
    /// Drops a totem that casts `sub` for `duration`.
    Totem,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Effect {
    None,
    /// Speed multiplier for seconds.
    Slow(f32, f32),
    Root(f32),
    Stun(f32),
    Fear(f32),
    /// Outgoing damage multiplier for seconds.
    Weaken(f32, f32),
    /// Heal the hero by this fraction of damage dealt.
    Drain(f32),
    /// Kill outright below this health fraction.
    Execute(f32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Targeting {
    Nearest,
    Toughest,
}

pub(crate) struct AbilityDef {
    pub key: &'static str,
    pub spell: u32,
    pub kind: Kind,
    /// Direct damage; per tick for areas and pulses; absorb, heal or reflect for the self kinds.
    pub damage: f32,
    /// Damage over time, per second, for `duration`.
    pub dot: f32,
    pub cooldown: f32,
    pub range: f32,
    /// Splash for single hits, area for the area kinds, the jump for a chain.
    pub radius: f32,
    pub count: u32,
    pub duration: f32,
    pub effect: Effect,
    /// The spell whose state kit marks an affected enemy, 0 for none.
    pub mark: u32,
    /// A totem's display, and the spell it casts.
    pub totem: (u32, u32),
    /// The area's own visual hangs overhead and would hide the field from the camera, so the
    /// area plays the spell's impact on whatever it hits instead.
    pub overhead: bool,
    /// How the ability selects its targets.
    pub targeting: Targeting,
}

const fn ab(key: &'static str, spell: u32, kind: Kind) -> AbilityDef {
    AbilityDef {
        key,
        spell,
        kind,
        damage: 0.0,
        dot: 0.0,
        cooldown: 2.0,
        range: 20.0,
        radius: 0.0,
        count: 1,
        duration: 0.0,
        effect: Effect::None,
        mark: 0,
        totem: (0, 0),
        overhead: false,
        targeting: Targeting::Nearest,
    }
}

macro_rules! ability {
    ($key:literal, $spell:literal, $kind:ident { $($f:ident: $v:expr),* $(,)? }) => {
        AbilityDef { $($f: $v,)* ..ab($key, $spell, Kind::$kind) }
    };
}

pub(crate) static ABILITIES: &[AbilityDef] = &[
    // --- Mage ---
    ability!(
        "frostbolt",
        116,
        Bolt {
            damage: 12.0,
            cooldown: 1.3,
            range: 24.0,
            effect: Effect::Slow(0.5, 2.5),
            mark: 116
        }
    ),
    ability!(
        "fireball",
        133,
        Bolt {
            damage: 20.0,
            cooldown: 2.4,
            range: 26.0,
            radius: 3.5
        }
    ),
    ability!(
        "arcane_missiles",
        7268,
        Bolt {
            damage: 6.0,
            cooldown: 1.9,
            range: 22.0,
            count: 3
        }
    ),
    ability!(
        "frost_nova",
        122,
        Nova {
            damage: 8.0,
            cooldown: 7.0,
            radius: 9.0,
            effect: Effect::Root(2.5),
            mark: 122
        }
    ),
    ability!(
        "arcane_explosion",
        1449,
        Nova {
            damage: 10.0,
            cooldown: 2.8,
            radius: 7.0
        }
    ),
    ability!(
        "blizzard",
        10,
        Ground {
            damage: 7.0,
            cooldown: 9.0,
            range: 20.0,
            radius: 7.0,
            duration: 6.0,
            effect: Effect::Slow(0.6, 1.5)
        }
    ),
    ability!(
        "flamestrike",
        2120,
        Ground {
            damage: 14.0,
            cooldown: 8.0,
            range: 20.0,
            radius: 5.0,
            duration: 4.0
        }
    ),
    ability!(
        "fire_blast",
        2136,
        Strike {
            damage: 26.0,
            cooldown: 3.5,
            range: 18.0
        }
    ),
    ability!(
        "cone_of_cold",
        120,
        Cone {
            damage: 14.0,
            cooldown: 4.5,
            radius: 11.0,
            effect: Effect::Slow(0.5, 3.0),
            mark: 120
        }
    ),
    ability!(
        "blast_wave",
        11113,
        Nova {
            damage: 18.0,
            cooldown: 6.5,
            radius: 8.0,
            effect: Effect::Slow(0.6, 2.0)
        }
    ),
    ability!(
        "pyroblast",
        11366,
        Bolt {
            damage: 70.0,
            dot: 6.0,
            duration: 6.0,
            cooldown: 6.0,
            range: 30.0,
            targeting: Targeting::Toughest
        }
    ),
    // --- Warrior ---
    ability!(
        "cleave",
        845,
        Melee {
            damage: 14.0,
            cooldown: 1.1,
            range: 5.5,
            count: 3
        }
    ),
    ability!(
        "heroic_strike",
        78,
        Melee {
            damage: 28.0,
            cooldown: 1.6,
            range: 5.5
        }
    ),
    ability!(
        "whirlwind",
        1680,
        Nova {
            damage: 16.0,
            cooldown: 4.0,
            radius: 7.0
        }
    ),
    ability!(
        "thunder_clap",
        6343,
        Nova {
            damage: 10.0,
            cooldown: 5.0,
            radius: 8.0,
            effect: Effect::Slow(0.5, 3.0),
            mark: 6343
        }
    ),
    ability!(
        "sweeping_strikes",
        12292,
        Aura {
            damage: 6.0,
            cooldown: 0.6,
            radius: 5.5
        }
    ),
    ability!(
        "demoralizing_shout",
        1160,
        Nova {
            damage: 4.0,
            cooldown: 8.0,
            radius: 11.0,
            effect: Effect::Weaken(0.5, 5.0)
        }
    ),
    ability!(
        "execute",
        5308,
        Strike {
            damage: 40.0,
            cooldown: 2.0,
            range: 8.0,
            count: 2,
            effect: Effect::Execute(0.3)
        }
    ),
    ability!(
        "mortal_strike",
        12294,
        Melee {
            damage: 48.0,
            cooldown: 3.0,
            range: 6.0
        }
    ),
    ability!(
        "retaliation",
        20230,
        Thorns {
            damage: 14.0,
            cooldown: 18.0,
            duration: 8.0
        }
    ),
    // --- Rogue ---
    ability!(
        "sinister_strike",
        1752,
        Melee {
            damage: 15.0,
            cooldown: 0.85,
            range: 5.5
        }
    ),
    ability!(
        "eviscerate",
        2098,
        Melee {
            damage: 42.0,
            cooldown: 2.8,
            range: 5.5
        }
    ),
    ability!(
        "blade_flurry",
        13877,
        Nova {
            damage: 11.0,
            cooldown: 3.0,
            radius: 6.0
        }
    ),
    ability!(
        "rupture",
        1943,
        Dot {
            dot: 5.0,
            cooldown: 3.5,
            range: 10.0,
            count: 3,
            duration: 6.0
        }
    ),
    ability!(
        "gouge",
        1776,
        Melee {
            damage: 8.0,
            cooldown: 4.0,
            range: 6.0,
            count: 2,
            effect: Effect::Stun(2.0),
            mark: 1776
        }
    ),
    ability!(
        "kidney_shot",
        408,
        Melee {
            damage: 12.0,
            cooldown: 6.0,
            range: 6.0,
            effect: Effect::Stun(3.5),
            mark: 408
        }
    ),
    // --- Priest ---
    ability!(
        "smite",
        585,
        Strike {
            damage: 14.0,
            cooldown: 1.2,
            range: 20.0
        }
    ),
    ability!(
        "shadow_word_pain",
        589,
        Dot {
            dot: 4.0,
            cooldown: 3.0,
            range: 18.0,
            count: 3,
            duration: 9.0
        }
    ),
    ability!(
        "holy_fire",
        14914,
        Strike {
            damage: 22.0,
            dot: 3.0,
            duration: 6.0,
            cooldown: 3.5,
            range: 20.0
        }
    ),
    ability!(
        "mind_blast",
        8092,
        Strike {
            damage: 32.0,
            cooldown: 3.5,
            range: 22.0
        }
    ),
    ability!(
        "holy_nova",
        15237,
        Nova {
            damage: 11.0,
            cooldown: 3.2,
            radius: 9.0,
            effect: Effect::Drain(0.1)
        }
    ),
    ability!(
        "psychic_scream",
        8122,
        Nova {
            damage: 2.0,
            cooldown: 12.0,
            radius: 8.0,
            effect: Effect::Fear(3.5),
            mark: 8122
        }
    ),
    ability!(
        "power_word_shield",
        17,
        Shield {
            damage: 30.0,
            cooldown: 14.0,
            duration: 10.0
        }
    ),
    // --- Warlock ---
    ability!(
        "shadow_bolt",
        686,
        Bolt {
            damage: 15.0,
            cooldown: 1.4,
            range: 24.0
        }
    ),
    ability!(
        "corruption",
        172,
        Dot {
            dot: 4.0,
            cooldown: 3.0,
            range: 20.0,
            count: 3,
            duration: 10.0,
            mark: 172
        }
    ),
    ability!(
        "curse_of_agony",
        980,
        Dot {
            dot: 3.5,
            cooldown: 2.5,
            range: 20.0,
            count: 4,
            duration: 12.0
        }
    ),
    ability!(
        "immolate",
        348,
        Strike {
            damage: 12.0,
            dot: 4.0,
            duration: 9.0,
            cooldown: 2.4,
            range: 20.0,
            count: 2,
            mark: 348
        }
    ),
    ability!(
        "rain_of_fire",
        5740,
        Ground {
            damage: 8.0,
            cooldown: 9.0,
            range: 20.0,
            radius: 7.0,
            duration: 6.0
        }
    ),
    ability!(
        "hellfire",
        5857,
        Aura {
            damage: 6.0,
            cooldown: 1.0,
            radius: 7.0
        }
    ),
    ability!(
        "death_coil",
        6789,
        Bolt {
            damage: 26.0,
            cooldown: 5.0,
            range: 22.0,
            effect: Effect::Drain(0.5)
        }
    ),
    ability!(
        "searing_pain",
        5676,
        Strike {
            damage: 18.0,
            cooldown: 1.8,
            range: 20.0
        }
    ),
    ability!(
        "shadowburn",
        17877,
        Strike {
            damage: 46.0,
            cooldown: 5.0,
            range: 20.0
        }
    ),
    // --- Hunter ---
    ability!(
        "arcane_shot",
        3044,
        Bolt {
            damage: 13.0,
            cooldown: 1.0,
            range: 28.0
        }
    ),
    ability!(
        "multi_shot",
        2643,
        Bolt {
            damage: 10.0,
            cooldown: 2.0,
            range: 26.0,
            count: 3
        }
    ),
    ability!(
        "serpent_sting",
        1978,
        Bolt {
            damage: 2.0,
            dot: 4.0,
            duration: 9.0,
            cooldown: 2.6,
            range: 26.0,
            count: 2
        }
    ),
    ability!(
        "volley",
        1510,
        Ground {
            damage: 8.0,
            cooldown: 9.0,
            range: 24.0,
            radius: 7.0,
            duration: 5.0
        }
    ),
    ability!(
        "explosive_trap",
        13812,
        Ground {
            damage: 22.0,
            dot: 4.0,
            cooldown: 8.0,
            range: 14.0,
            radius: 5.0,
            duration: 0.0
        }
    ),
    ability!(
        "concussive_shot",
        5116,
        Bolt {
            damage: 8.0,
            cooldown: 3.0,
            range: 28.0,
            effect: Effect::Slow(0.5, 4.0),
            mark: 5116
        }
    ),
    ability!(
        "aimed_shot",
        19434,
        Bolt {
            damage: 48.0,
            cooldown: 4.0,
            range: 32.0,
            targeting: Targeting::Toughest
        }
    ),
    // --- Shaman ---
    ability!(
        "lightning_bolt",
        403,
        Bolt {
            damage: 15.0,
            cooldown: 1.3,
            range: 24.0
        }
    ),
    ability!(
        "chain_lightning",
        421,
        Chain {
            damage: 18.0,
            cooldown: 3.0,
            range: 20.0,
            radius: 9.0,
            count: 3
        }
    ),
    ability!(
        "earth_shock",
        8042,
        Strike {
            damage: 22.0,
            cooldown: 2.5,
            range: 20.0
        }
    ),
    ability!(
        "flame_shock",
        8050,
        Strike {
            damage: 10.0,
            dot: 4.0,
            duration: 9.0,
            cooldown: 3.0,
            range: 20.0,
            count: 2
        }
    ),
    ability!(
        "frost_shock",
        8056,
        Strike {
            damage: 16.0,
            cooldown: 3.0,
            range: 20.0,
            effect: Effect::Slow(0.5, 4.0)
        }
    ),
    ability!(
        "searing_totem",
        3599,
        Totem {
            damage: 9.0,
            cooldown: 14.0,
            range: 20.0,
            duration: 14.0,
            totem: (4589, 3606)
        }
    ),
    ability!(
        "magma_totem",
        8190,
        Totem {
            damage: 8.0,
            cooldown: 16.0,
            radius: 7.0,
            duration: 14.0,
            totem: (4683, 8349)
        }
    ),
    ability!(
        "lightning_shield",
        324,
        Orbit {
            damage: 12.0,
            cooldown: 0.9,
            radius: 4.0
        }
    ),
    // --- Paladin ---
    ability!(
        "hammer_of_wrath",
        24275,
        Bolt {
            damage: 16.0,
            cooldown: 1.5,
            range: 24.0
        }
    ),
    ability!(
        "consecration",
        26573,
        GroundSelf {
            damage: 7.0,
            cooldown: 9.0,
            radius: 7.0,
            duration: 8.0
        }
    ),
    ability!(
        "holy_shock",
        25912,
        Strike {
            damage: 24.0,
            cooldown: 2.5,
            range: 20.0
        }
    ),
    ability!(
        "exorcism",
        879,
        Strike {
            damage: 36.0,
            cooldown: 5.0,
            range: 22.0,
            targeting: Targeting::Toughest
        }
    ),
    ability!(
        "holy_wrath",
        2812,
        Nova {
            damage: 14.0,
            cooldown: 5.0,
            radius: 11.0
        }
    ),
    ability!(
        "hammer_of_justice",
        853,
        Strike {
            damage: 8.0,
            cooldown: 6.0,
            range: 12.0,
            count: 2,
            effect: Effect::Stun(3.0),
            mark: 853
        }
    ),
    ability!(
        "divine_shield",
        642,
        Immune {
            cooldown: 28.0,
            duration: 4.0
        }
    ),
    ability!(
        "judgement",
        20271,
        Strike {
            damage: 26.0,
            cooldown: 4.0,
            range: 14.0,
            count: 2
        }
    ),
    // --- Druid ---
    ability!(
        "wrath",
        5176,
        Bolt {
            damage: 13.0,
            cooldown: 1.2,
            range: 24.0
        }
    ),
    ability!(
        "moonfire",
        8921,
        Strike {
            damage: 10.0,
            dot: 3.0,
            duration: 9.0,
            cooldown: 2.0,
            range: 22.0,
            count: 2
        }
    ),
    ability!(
        "starfire",
        2912,
        Strike {
            damage: 40.0,
            cooldown: 4.0,
            range: 24.0,
            targeting: Targeting::Toughest
        }
    ),
    ability!(
        "hurricane",
        16914,
        Ground {
            damage: 7.0,
            cooldown: 10.0,
            range: 20.0,
            radius: 8.0,
            duration: 7.0,
            effect: Effect::Slow(0.6, 1.5),
            overhead: true
        }
    ),
    ability!(
        "insect_swarm",
        5570,
        Dot {
            dot: 4.0,
            cooldown: 3.0,
            range: 20.0,
            count: 3,
            duration: 10.0,
            mark: 5570
        }
    ),
    ability!(
        "entangling_roots",
        339,
        Dot {
            dot: 2.0,
            cooldown: 6.0,
            range: 16.0,
            count: 4,
            duration: 5.0,
            effect: Effect::Root(4.0),
            mark: 339
        }
    ),
    ability!(
        "thorns",
        467,
        Thorns {
            damage: 8.0,
            cooldown: 15.0,
            duration: 15.0
        }
    ),
    ability!(
        "rejuvenation",
        774,
        Heal {
            damage: 30.0,
            cooldown: 15.0,
            duration: 10.0
        }
    ),
];

pub(crate) fn ability(key: &str) -> &'static AbilityDef {
    ABILITIES
        .iter()
        .find(|a| a.key == key)
        .unwrap_or_else(|| panic!("survivors: no ability {key:?}"))
}

pub(crate) const MAX_ABILITY_LEVEL: u8 = 8;
pub(crate) const MAX_ABILITIES: usize = 6;
