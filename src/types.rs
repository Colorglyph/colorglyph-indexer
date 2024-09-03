use colorglyph::types::Offer;
use serde::Serialize;
use zephyr_sdk::{
    prelude::*,
    soroban_sdk::{xdr::ScVal, Address, Vec as SorobanVec},
    DatabaseDerive, EnvClient,
};

// --- COLORS ---

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("colors")]
pub struct ZephyrColor {
    pub miner: ScVal,
    pub owner: ScVal,
    pub color: u32,
    pub amount: u32,
}

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("colors")]
pub struct ZephyrColorAmount {
    pub amount: u32,
}

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("colors")]
pub struct ZephyrColorEmpty {}

// --- GLYPHS ---

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("glyphs")]
pub struct ZephyrGlyph {
    pub hash: ScVal,
    pub owner: ScVal,
    pub minter: ScVal,
    pub width: u32,
    pub length: u32,
    pub colors: ScVal,
}

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("glyphs")]
pub struct ZephyrGlyphNoColors {
    hash: ScVal,
    owner: ScVal,
    minter: ScVal,
    width: u32,
    length: u32,
}

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("glyphs")]
pub struct ZephyrGlyphWidthLengthColors {
    pub width: u32,
    pub length: u32,
    pub colors: ScVal,
}

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("glyphs")]
pub struct ZephyrGlyphOwner {
    pub owner: ScVal,
}

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("glyphs")]
pub struct ZephyrGlyphMinter {
    pub minter: ScVal,
}

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("glyphs")]
pub struct ZephyrGlyphEmpty {}

// --- OFFERS ---

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("offers")]
pub struct ZephyrOffer {
    pub seller: ScVal,
    pub selling: ScVal,
    pub buying: ScVal,
    pub amount: ScVal, // because currently i128 is broken
    pub active: ScVal,
}

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("offers")]
pub struct ZephyrOfferNoActive {
    seller: ScVal,
    selling: ScVal,
    buying: ScVal,
    amount: ScVal, // because currently i128 is broken
}

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("offers")]
pub struct ZephyrOfferActive {
    pub active: ScVal,
}

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("offers")]
pub struct ZephyrOfferEmpty {}

// --- OTHER ---

#[derive(Clone, Debug)]
pub enum Offers {
    Offers(SorobanVec<Offer>),
    Addresses(SorobanVec<Address>),
}
