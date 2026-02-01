use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    All,
    AllYouPlay,
    Blizzard,
    DLGamer,
    Dreamgame,
    EAStore,
    EpicGames,
    Fanatical,
    FireFlower,
    GameBillet,
    GamersGate,
    Gamesload,
    GamesPlanetDE,
    GamesPlanetFR,
    GamesPlanetUK,
    GamesPlanetUS,
    Gog,
    GreenManGaming,
    HumbleStore,
    IndieGala,
    JoyBuggy,
    MacGameStore,
    MicrosoftStore,
    Newegg,
    Nuuvem,
    PlanetPlay,
    PlayerLand,
    Playsum,
    Steam,
    UbisoftStore,
    WinGameStore,
    ZoomPlatform,
}

impl Platform {
    pub fn name(&self) -> &str {
        match self {
            Platform::All => "All Platforms",
            Platform::AllYouPlay => "AllYouPlay",
            Platform::Blizzard => "Blizzard",
            Platform::DLGamer => "DLGamer",
            Platform::Dreamgame => "Dreamgame",
            Platform::EAStore => "EA Store",
            Platform::EpicGames => "Epic Game Store",
            Platform::Fanatical => "Fanatical",
            Platform::FireFlower => "FireFlower",
            Platform::GameBillet => "GameBillet",
            Platform::GamersGate => "GamersGate",
            Platform::Gamesload => "Gamesload",
            Platform::GamesPlanetDE => "GamesPlanet DE",
            Platform::GamesPlanetFR => "GamesPlanet FR",
            Platform::GamesPlanetUK => "GamesPlanet UK",
            Platform::GamesPlanetUS => "GamesPlanet US",
            Platform::Gog => "GOG",
            Platform::GreenManGaming => "GreenManGaming",
            Platform::HumbleStore => "Humble Store",
            Platform::IndieGala => "IndieGala Store",
            Platform::JoyBuggy => "JoyBuggy",
            Platform::MacGameStore => "MacGameStore",
            Platform::MicrosoftStore => "Microsoft Store",
            Platform::Newegg => "Newegg",
            Platform::Nuuvem => "Nuuvem",
            Platform::PlanetPlay => "PlanetPlay",
            Platform::PlayerLand => "PlayerLand",
            Platform::Playsum => "Playsum",
            Platform::Steam => "Steam",
            Platform::UbisoftStore => "Ubisoft Store",
            Platform::WinGameStore => "WinGameStore",
            Platform::ZoomPlatform => "ZOOM Platform",
        }
    }

    pub fn shop_id(&self) -> Option<u32> {
        match self {
            Platform::All => None,
            Platform::AllYouPlay => Some(2),
            Platform::Blizzard => Some(4),
            Platform::DLGamer => Some(13),
            Platform::Dreamgame => Some(15),
            Platform::EAStore => Some(52),
            Platform::EpicGames => Some(16),
            Platform::Fanatical => Some(6),
            Platform::FireFlower => Some(17),
            Platform::GameBillet => Some(20),
            Platform::GamersGate => Some(24),
            Platform::Gamesload => Some(25),
            Platform::GamesPlanetDE => Some(27),
            Platform::GamesPlanetFR => Some(28),
            Platform::GamesPlanetUK => Some(26),
            Platform::GamesPlanetUS => Some(29),
            Platform::Gog => Some(35),
            Platform::GreenManGaming => Some(36),
            Platform::HumbleStore => Some(37),
            Platform::IndieGala => Some(42),
            Platform::JoyBuggy => Some(65),
            Platform::MacGameStore => Some(47),
            Platform::MicrosoftStore => Some(48),
            Platform::Newegg => Some(49),
            Platform::Nuuvem => Some(50),
            Platform::PlanetPlay => Some(73),
            Platform::PlayerLand => Some(74),
            Platform::Playsum => Some(70),
            Platform::Steam => Some(61),
            Platform::UbisoftStore => Some(62),
            Platform::WinGameStore => Some(64),
            Platform::ZoomPlatform => Some(72),
        }
    }

    pub const ALL: &'static [Platform] = &[
        Platform::All,
        Platform::AllYouPlay,
        Platform::Blizzard,
        Platform::DLGamer,
        Platform::Dreamgame,
        Platform::EAStore,
        Platform::EpicGames,
        Platform::Fanatical,
        Platform::FireFlower,
        Platform::GameBillet,
        Platform::GamersGate,
        Platform::Gamesload,
        Platform::GamesPlanetDE,
        Platform::GamesPlanetFR,
        Platform::GamesPlanetUK,
        Platform::GamesPlanetUS,
        Platform::Gog,
        Platform::GreenManGaming,
        Platform::HumbleStore,
        Platform::IndieGala,
        Platform::JoyBuggy,
        Platform::MacGameStore,
        Platform::MicrosoftStore,
        Platform::Newegg,
        Platform::Nuuvem,
        Platform::PlanetPlay,
        Platform::PlayerLand,
        Platform::Playsum,
        Platform::Steam,
        Platform::UbisoftStore,
        Platform::WinGameStore,
        Platform::ZoomPlatform,
    ];
}

/// Represents a game deal from IsThereAnyDeal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    pub title: String,
    pub shop: Shop,
    pub price: Price,
    pub url: String,
    pub history_low: Option<f64>,
}

/// Store/shop information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shop {
    pub id: String,
    pub name: String,
}

/// Price information with discount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    pub amount: f64,
    pub currency: String,
    pub discount: u8,
}

/// Filter options for deal queries
#[derive(Debug, Clone, Default)]
pub struct DealFilter {
    pub shop_ids: Option<Vec<String>>,
    pub country: String,
    pub limit: usize,
}

impl Default for Shop {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
        }
    }
}

impl Default for Price {
    fn default() -> Self {
        Self {
            amount: 0.0,
            currency: "USD".to_string(),
            discount: 0,
        }
    }
}
