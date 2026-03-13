#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityType {
    SoloArtist,
    Collective,
    MythicEntity,
    AIConstruct,
    AnonymousOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    TriangleHalo,
    ReactorRing,
    CrystalShard,
    NeonGlyph,
    OrbitalSigil,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToneType {
    Cyberpunk,
    Cosmic,
    Mystical,
    Industrial,
    Ethereal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleBias {
    Electronic,
    Jazz,
    World,
    Classical,
    Fusion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityProfile {
    pub name: String,
    pub identity_type: IdentityType,
    pub color_palette: [u32; 3],
    pub symbol: SymbolType,
    pub tone: ToneType,
    pub genre_bias: StyleBias,
}

pub fn generate_identity_name(seed: u64, identity_type: IdentityType) -> String {
    const HEAD: [&str; 12] = [
        "Pulse", "Resonant", "Nyx", "AX-13", "Oracle", "Signal", "Velvet", "Ghost", "Voltage",
        "Prism", "Echo", "Solar",
    ];
    const TAIL: [&str; 12] = [
        "Architect",
        "Choir",
        "Solaris",
        "Resonance Node",
        "Oracle",
        "Pilgrim",
        "Conclave",
        "Frequency",
        "Cathedral",
        "Ascension",
        "Protocol",
        "Assembly",
    ];

    let h = HEAD[(splitmix_u64(seed ^ 0x1111) as usize) % HEAD.len()];
    let t = TAIL[(splitmix_u64(seed ^ 0x2222) as usize) % TAIL.len()];

    match identity_type {
        IdentityType::SoloArtist => format!("{} {}", h, t),
        IdentityType::Collective => format!("The {} {}", h, t),
        IdentityType::MythicEntity => format!("{} {}", h, t),
        IdentityType::AIConstruct => format!("{} {}", h, t),
        IdentityType::AnonymousOrder => format!("The {} {}", h, t),
    }
}

pub fn generate_identity(seed: u64) -> IdentityProfile {
    let identity_type = match splitmix_u64(seed ^ 0xA001) % 5 {
        0 => IdentityType::SoloArtist,
        1 => IdentityType::Collective,
        2 => IdentityType::MythicEntity,
        3 => IdentityType::AIConstruct,
        _ => IdentityType::AnonymousOrder,
    };

    let symbol = match splitmix_u64(seed ^ 0xA002) % 5 {
        0 => SymbolType::TriangleHalo,
        1 => SymbolType::ReactorRing,
        2 => SymbolType::CrystalShard,
        3 => SymbolType::NeonGlyph,
        _ => SymbolType::OrbitalSigil,
    };

    let tone = match splitmix_u64(seed ^ 0xA003) % 5 {
        0 => ToneType::Cyberpunk,
        1 => ToneType::Cosmic,
        2 => ToneType::Mystical,
        3 => ToneType::Industrial,
        _ => ToneType::Ethereal,
    };

    let genre_bias = match splitmix_u64(seed ^ 0xA004) % 5 {
        0 => StyleBias::Electronic,
        1 => StyleBias::Jazz,
        2 => StyleBias::World,
        3 => StyleBias::Classical,
        _ => StyleBias::Fusion,
    };

    let color_palette = [
        rgb_from_seed(seed ^ 0xB001),
        rgb_from_seed(seed ^ 0xB002),
        rgb_from_seed(seed ^ 0xB003),
    ];

    IdentityProfile {
        name: generate_identity_name(seed, identity_type),
        identity_type,
        color_palette,
        symbol,
        tone,
        genre_bias,
    }
}

fn rgb_from_seed(seed: u64) -> u32 {
    let v = splitmix_u64(seed);
    let r = ((v >> 16) & 0xFF) as u32;
    let g = ((v >> 32) & 0xFF) as u32;
    let b = ((v >> 48) & 0xFF) as u32;
    (r << 16) | (g << 8) | b
}

#[cfg(test)]
mod tests {
    use super::{IdentityType, generate_identity, generate_identity_name};

    #[test]
    fn identity_generation_is_deterministic() {
        let a = generate_identity(88);
        let b = generate_identity(88);
        assert_eq!(a, b);
    }

    #[test]
    fn identity_name_generation_is_deterministic() {
        let a = generate_identity_name(3, IdentityType::AIConstruct);
        let b = generate_identity_name(3, IdentityType::AIConstruct);
        assert_eq!(a, b);
    }
}
use crate::determinism::splitmix_u64;
