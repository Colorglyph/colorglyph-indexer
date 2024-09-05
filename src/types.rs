use colorglyph::types::Offer;
use serde::{Deserialize, Serialize};
use zephyr_sdk::soroban_sdk::{xdr::ScVal, Address, Vec as SorobanVec};

// --- COLORS ---

#[derive(Serialize, Deserialize)]
pub struct DataColor {
    pub change: Change,
    pub miner: String,
    pub owner: String,
    pub color: u32,
    pub amount: u32,
}

// --- GLYPHS ---

#[derive(Serialize, Deserialize)]
pub struct DataGlyph {
    pub change: Change,
    pub hash: String,
    pub width: u32,
    pub length: u32,
    pub colors: String, // TODO maybe cheaper as a Vec<8>
}

#[derive(Serialize, Deserialize)]
pub struct DataGlyphOwner {
    pub change: Change,
    pub hash: String,
    pub owner: String,
}

#[derive(Serialize, Deserialize)]
pub struct DataGlyphMinter {
    pub change: Change,
    pub hash: String,
    pub minter: String,
}

// --- OFFERS ---

#[derive(Serialize, Deserialize)]
pub struct DataOffer {
    pub change: Change,
    pub seller: String,
    pub selling: String,
    pub buying: String,
    pub amount: Option<ScVal>, // because currently i128 is broken
}

#[derive(Serialize, Deserialize)]
pub struct DataOfferSellerSelling {
    pub change: Change,
    pub seller: String,
    pub selling: String,
}

#[derive(Serialize, Deserialize)]
pub struct DataOfferSellingBuyingAmount {
    pub change: Change,
    pub selling: String,
    pub buying: String,
    pub amount: Option<ScVal>, // because currently i128 is broken
}

// --- OTHER ---

#[derive(Serialize, Deserialize)]
pub enum Data {
    Color(DataColor),
    Glyph(DataGlyph),
    GlyphOwner(DataGlyphOwner),
    GlyphMinter(DataGlyphMinter),
    Offer(DataOffer),
    OfferSellerSelling(DataOfferSellerSelling),
    OfferSellingBuyingAmount(DataOfferSellingBuyingAmount),
}

#[derive(Serialize, Deserialize)]
pub struct Body {
    pub seq_num: i64,
    pub data: Vec<Data>,
}

#[derive(Serialize, Deserialize, Clone)]
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
