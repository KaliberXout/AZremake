#[derive(Clone, Copy)]
pub(crate) struct EnemyDef {
    pub name: &'static str,
    pub display: u32,
    pub hp: f32,
    pub speed: f32,
    pub damage: f32,
    pub scale: f32,
    pub xp: u32,
    pub level: u32,
    /// A great beast, which takes more room in the crowd.
    pub big: bool,
}

const fn en(
    name: &'static str,
    display: u32,
    hp: f32,
    speed: f32,
    damage: f32,
    xp: u32,
    level: u32,
) -> EnemyDef {
    EnemyDef {
        name,
        display,
        hp,
        speed,
        damage,
        scale: 1.0,
        xp,
        level,
        big: false,
    }
}

/// The Barrens, roughly in the order a character meets them.
pub(crate) static ENEMIES: &[EnemyDef] = &[
    en("Fleeting Plainstrider", 1284, 8.0, 3.6, 2.0, 1, 10),
    en("Zhevra Runner", 6087, 10.0, 4.4, 2.0, 1, 11),
    en("Razormane Quilboar", 1218, 16.0, 3.4, 3.0, 2, 12),
    en("Razormane Hunter", 6094, 18.0, 3.6, 4.0, 2, 12),
    en("Savannah Huntress", 1056, 22.0, 4.8, 8.0, 2, 13),
    en("Sunscale Lashtail", 1744, 26.0, 4.6, 8.0, 3, 13),
    en("Kolkar Wrangler", 9442, 36.0, 4.0, 10.0, 3, 14),
    en("Kolkar Stormer", 9443, 40.0, 4.0, 11.0, 3, 14),
    en("Witchwing Harpy", 3218, 34.0, 5.0, 10.0, 3, 15),
    en("Hecklefang Hyena", 2710, 44.0, 5.2, 12.0, 4, 15),
    en("Razormane Defender", 1253, 70.0, 3.3, 14.0, 5, 16),
    en("Razormane Mystic", 4643, 55.0, 3.8, 12.0, 4, 16),
    en("Sunscale Screecher", 1747, 60.0, 5.0, 13.0, 4, 17),
    en("Kolkar Bloodcharger", 9447, 85.0, 4.6, 16.0, 5, 18),
    en("Savannah Prowler", 1973, 75.0, 5.4, 15.0, 5, 18),
    en("Thunderhawk Hatchling", 1742, 90.0, 4.2, 16.0, 6, 19),
    en("Witchwing Slayer", 2163, 95.0, 5.0, 18.0, 6, 20),
    en("Bristleback Geomancer", 6091, 120.0, 3.6, 20.0, 7, 21),
    huge(en("Stormsnout", 1537, 150.0, 4.0, 22.0, 8, 22)),
    huge(en("Barrens Kodo", 1453, 220.0, 3.2, 28.0, 12, 23)),
    en("Sunscale Scytheclaw", 4442, 160.0, 5.6, 24.0, 8, 24),
    huge(en("Thunderhead", 1538, 180.0, 4.2, 26.0, 10, 25)),
    en("Silithid Swarmer", 2731, 90.0, 6.0, 18.0, 5, 25),
    huge(en("Greater Thunderhawk", 1974, 240.0, 4.4, 30.0, 12, 26)),
    en("Hecklefang Stalker", 2712, 200.0, 6.0, 30.0, 10, 27),
    en("Kolkar Destroyer", 9446, 280.0, 4.6, 34.0, 14, 28),
    en("Kolkar Mauler", 4874, 300.0, 4.6, 36.0, 14, 29),
    huge(en("Greater Barrens Kodo", 1232, 420.0, 3.4, 42.0, 24, 30)),
    huge(en("Elder Thunder Lizard", 2764, 380.0, 4.4, 44.0, 22, 31)),
    huge(en("Silithid Protector", 1307, 360.0, 5.0, 40.0, 20, 32)),
];

/// The great beasts at three quarters size, so a crowd of them still leaves the field readable.
const fn huge(def: EnemyDef) -> EnemyDef {
    EnemyDef {
        scale: 0.75,
        big: true,
        ..def
    }
}

pub(crate) static BOSSES: &[EnemyDef] = &[
    EnemyDef {
        scale: 1.6,
        ..en("Echeyakee", 1934, 700.0, 5.2, 20.0, 60, 16)
    },
    EnemyDef {
        scale: 1.6,
        ..en("Takk the Leaper", 1337, 1800.0, 5.4, 32.0, 120, 19)
    },
    EnemyDef {
        scale: 1.7,
        ..en("Lakota'mani", 1241, 4200.0, 4.0, 45.0, 200, 22)
    },
    EnemyDef {
        scale: 1.7,
        ..en("Humar the Pridelord", 4424, 7000.0, 5.6, 55.0, 300, 26)
    },
    EnemyDef {
        scale: 1.8,
        ..en("Agathelos the Raging", 2450, 12000.0, 5.0, 70.0, 400, 33)
    },
    EnemyDef {
        scale: 1.8,
        ..en("Hezrul Bloodmark", 9448, 20000.0, 4.8, 90.0, 500, 33)
    },
];
