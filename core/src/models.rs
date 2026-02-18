use serde::{Deserialize, Serialize};

/// Supported countries for deal filtering (ISO 3166-1 alpha-2 codes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Region {
    // Europe
    AT,
    BE,
    BG,
    CH,
    CZ,
    DE,
    DK,
    EE,
    ES,
    FI,
    #[default]
    FR,
    GB,
    GR,
    HR,
    HU,
    IE,
    IT,
    LT,
    LV,
    NL,
    NO,
    PL,
    PT,
    RO,
    SE,
    SK,
    // Americas
    AR,
    BR,
    CA,
    CL,
    CO,
    MX,
    US,
    // Asia-Pacific
    AU,
    CN,
    HK,
    ID,
    IN,
    JP,
    KR,
    NZ,
    PH,
    SG,
    TH,
    TW,
    // Middle East & Africa
    AE,
    IL,
    SA,
    TR,
    ZA,
}

impl Region {
    pub fn name(&self) -> &str {
        match self {
            // Europe
            Region::AT => "Austria",
            Region::BE => "Belgium",
            Region::BG => "Bulgaria",
            Region::CH => "Switzerland",
            Region::CZ => "Czechia",
            Region::DE => "Germany",
            Region::DK => "Denmark",
            Region::EE => "Estonia",
            Region::ES => "Spain",
            Region::FI => "Finland",
            Region::FR => "France",
            Region::GB => "United Kingdom",
            Region::GR => "Greece",
            Region::HR => "Croatia",
            Region::HU => "Hungary",
            Region::IE => "Ireland",
            Region::IT => "Italy",
            Region::LT => "Lithuania",
            Region::LV => "Latvia",
            Region::NL => "Netherlands",
            Region::NO => "Norway",
            Region::PL => "Poland",
            Region::PT => "Portugal",
            Region::RO => "Romania",
            Region::SE => "Sweden",
            Region::SK => "Slovakia",
            // Americas
            Region::AR => "Argentina",
            Region::BR => "Brazil",
            Region::CA => "Canada",
            Region::CL => "Chile",
            Region::CO => "Colombia",
            Region::MX => "Mexico",
            Region::US => "United States",
            // Asia-Pacific
            Region::AU => "Australia",
            Region::CN => "China",
            Region::HK => "Hong Kong",
            Region::ID => "Indonesia",
            Region::IN => "India",
            Region::JP => "Japan",
            Region::KR => "South Korea",
            Region::NZ => "New Zealand",
            Region::PH => "Philippines",
            Region::SG => "Singapore",
            Region::TH => "Thailand",
            Region::TW => "Taiwan",
            // Middle East & Africa
            Region::AE => "United Arab Emirates",
            Region::IL => "Israel",
            Region::SA => "Saudi Arabia",
            Region::TR => "Turkey",
            Region::ZA => "South Africa",
        }
    }

    pub fn code(&self) -> &str {
        match self {
            Region::AT => "AT",
            Region::BE => "BE",
            Region::BG => "BG",
            Region::CH => "CH",
            Region::CZ => "CZ",
            Region::DE => "DE",
            Region::DK => "DK",
            Region::EE => "EE",
            Region::ES => "ES",
            Region::FI => "FI",
            Region::FR => "FR",
            Region::GB => "GB",
            Region::GR => "GR",
            Region::HR => "HR",
            Region::HU => "HU",
            Region::IE => "IE",
            Region::IT => "IT",
            Region::LT => "LT",
            Region::LV => "LV",
            Region::NL => "NL",
            Region::NO => "NO",
            Region::PL => "PL",
            Region::PT => "PT",
            Region::RO => "RO",
            Region::SE => "SE",
            Region::SK => "SK",
            Region::AR => "AR",
            Region::BR => "BR",
            Region::CA => "CA",
            Region::CL => "CL",
            Region::CO => "CO",
            Region::MX => "MX",
            Region::US => "US",
            Region::AU => "AU",
            Region::CN => "CN",
            Region::HK => "HK",
            Region::ID => "ID",
            Region::IN => "IN",
            Region::JP => "JP",
            Region::KR => "KR",
            Region::NZ => "NZ",
            Region::PH => "PH",
            Region::SG => "SG",
            Region::TH => "TH",
            Region::TW => "TW",
            Region::AE => "AE",
            Region::IL => "IL",
            Region::SA => "SA",
            Region::TR => "TR",
            Region::ZA => "ZA",
        }
    }

    pub fn continent(&self) -> &str {
        match self {
            Region::AT
            | Region::BE
            | Region::BG
            | Region::CH
            | Region::CZ
            | Region::DE
            | Region::DK
            | Region::EE
            | Region::ES
            | Region::FI
            | Region::FR
            | Region::GB
            | Region::GR
            | Region::HR
            | Region::HU
            | Region::IE
            | Region::IT
            | Region::LT
            | Region::LV
            | Region::NL
            | Region::NO
            | Region::PL
            | Region::PT
            | Region::RO
            | Region::SE
            | Region::SK => "Europe",

            Region::AR
            | Region::BR
            | Region::CA
            | Region::CL
            | Region::CO
            | Region::MX
            | Region::US => "Americas",

            Region::AU
            | Region::CN
            | Region::HK
            | Region::ID
            | Region::IN
            | Region::JP
            | Region::KR
            | Region::NZ
            | Region::PH
            | Region::SG
            | Region::TH
            | Region::TW => "Asia-Pacific",

            Region::AE | Region::IL | Region::SA | Region::TR | Region::ZA => {
                "Middle East & Africa"
            }
        }
    }

    /// All regions, ordered by continent then alphabetically by name
    pub const ALL: &'static [Region] = &[
        // Europe
        Region::AT,
        Region::BE,
        Region::BG,
        Region::HR,
        Region::CZ,
        Region::DK,
        Region::EE,
        Region::FI,
        Region::FR,
        Region::DE,
        Region::GR,
        Region::HU,
        Region::IE,
        Region::IT,
        Region::LV,
        Region::LT,
        Region::NL,
        Region::NO,
        Region::PL,
        Region::PT,
        Region::RO,
        Region::SK,
        Region::ES,
        Region::SE,
        Region::CH,
        Region::GB,
        // Americas
        Region::AR,
        Region::BR,
        Region::CA,
        Region::CL,
        Region::CO,
        Region::MX,
        Region::US,
        // Asia-Pacific
        Region::AU,
        Region::CN,
        Region::HK,
        Region::IN,
        Region::ID,
        Region::JP,
        Region::NZ,
        Region::PH,
        Region::SG,
        Region::KR,
        Region::TW,
        Region::TH,
        // Middle East & Africa
        Region::AE,
        Region::IL,
        Region::SA,
        Region::ZA,
        Region::TR,
    ];

    pub fn from_code(code: &str) -> Option<Region> {
        match code {
            "AT" => Some(Region::AT),
            "BE" => Some(Region::BE),
            "BG" => Some(Region::BG),
            "CH" => Some(Region::CH),
            "CZ" => Some(Region::CZ),
            "DE" => Some(Region::DE),
            "DK" => Some(Region::DK),
            "EE" => Some(Region::EE),
            "ES" => Some(Region::ES),
            "FI" => Some(Region::FI),
            "FR" => Some(Region::FR),
            "GB" => Some(Region::GB),
            "GR" => Some(Region::GR),
            "HR" => Some(Region::HR),
            "HU" => Some(Region::HU),
            "IE" => Some(Region::IE),
            "IT" => Some(Region::IT),
            "LT" => Some(Region::LT),
            "LV" => Some(Region::LV),
            "NL" => Some(Region::NL),
            "NO" => Some(Region::NO),
            "PL" => Some(Region::PL),
            "PT" => Some(Region::PT),
            "RO" => Some(Region::RO),
            "SE" => Some(Region::SE),
            "SK" => Some(Region::SK),
            "AR" => Some(Region::AR),
            "BR" => Some(Region::BR),
            "CA" => Some(Region::CA),
            "CL" => Some(Region::CL),
            "CO" => Some(Region::CO),
            "MX" => Some(Region::MX),
            "US" => Some(Region::US),
            "AU" => Some(Region::AU),
            "CN" => Some(Region::CN),
            "HK" => Some(Region::HK),
            "ID" => Some(Region::ID),
            "IN" => Some(Region::IN),
            "JP" => Some(Region::JP),
            "KR" => Some(Region::KR),
            "NZ" => Some(Region::NZ),
            "PH" => Some(Region::PH),
            "SG" => Some(Region::SG),
            "TH" => Some(Region::TH),
            "TW" => Some(Region::TW),
            "AE" => Some(Region::AE),
            "IL" => Some(Region::IL),
            "SA" => Some(Region::SA),
            "TR" => Some(Region::TR),
            "ZA" => Some(Region::ZA),
            // Handle old config values
            "EU1" => Some(Region::FR),
            "EU2" => Some(Region::PL),
            "UK" => Some(Region::GB),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub id: String,
    pub title: String,
    pub shop: Shop,
    pub price: Price,
    pub regular_price: f64,
    pub url: String,
    pub history_low: Option<f64>,
}

/// Detailed game information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub id: String,
    pub title: String,
    pub release_date: Option<String>,
    pub developers: Vec<String>,
    pub publishers: Vec<String>,
    pub tags: Vec<String>,
}

/// Price history data point for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistoryPoint {
    pub timestamp: i64,
    pub price: f64,
    pub shop_name: String,
}

/// Store/shop information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

impl Default for Price {
    fn default() -> Self {
        Self {
            amount: 0.0,
            currency: "USD".to_string(),
            discount: 0,
        }
    }
}
