use colorglyph::types::Offer;
use serde::{Deserialize, Serialize};
use zephyr_sdk::soroban_sdk::{xdr::ScVal, Address, Vec as SorobanVec};

// --- COLORS ---

#[derive(Serialize, Deserialize)]
pub struct CloudflareColor {
    pub kind: Kind,
    pub change: Change,
    pub miner: String,
    pub owner: String,
    pub color: u32,
    pub amount: u32,
}

// --- GLYPHS ---

#[derive(Serialize, Deserialize)]
pub struct CloudflareGlyph {
    pub kind: Kind,
    pub change: Change,
    pub hash: String,
    pub owner: String,
    pub minter: String,
    pub width: u32,
    pub length: u32,
    pub colors: ScVal,
}

#[derive(Serialize, Deserialize)]
pub struct CloudflareGlyphOwner {
    pub kind: Kind,
    pub change: Change,
    pub hash: String,
    pub owner: String,
}

#[derive(Serialize, Deserialize)]
pub struct CloudflareGlyphMinter {
    pub kind: Kind,
    pub change: Change,
    pub hash: String,
    pub minter: String,
}

// --- OFFERS ---

#[derive(Serialize, Deserialize)]
pub struct CloudflareOffer {
    pub kind: Kind,
    pub change: Change,
    pub seller: String,
    pub selling: String,
    pub buying: String,
    pub amount: Option<ScVal>, // because currently i128 is broken
}

#[derive(Serialize, Deserialize)]
pub struct CloudflareOfferSellerSelling {
    pub kind: Kind,
    pub change: Change,
    pub seller: String,
    pub selling: String,
}

#[derive(Serialize, Deserialize)]
pub struct CloudflareOfferSellingBuyingAmount {
    pub kind: Kind,
    pub change: Change,
    pub selling: String,
    pub buying: String,
    pub amount: Option<ScVal>, // because currently i128 is broken
}

// --- OTHER ---

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Kind {
    Color,
    Glyph,
    Offer,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Change {
    Create,
    Update,
    Remove,
}

#[derive(Clone, Debug)]
pub enum Offers {
    Offers(SorobanVec<Offer>),
    Addresses(SorobanVec<Address>),
}
