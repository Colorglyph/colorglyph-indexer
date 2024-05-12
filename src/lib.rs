use std::collections::BTreeMap;

use glyphs::{get_glyph, insert_or_update_glyph};
use serde::{Deserialize, Serialize};
use zephyr_sdk::{
    prelude::*, soroban_sdk::{
        xdr::{AccountId, ContractEvent, ContractEventBody, Hash, InvokeContractArgs, LedgerEntryData, ScVal, Uint256}, Address, Map, String as SorobanString, Symbol
    }, utils::parts_to_i128, DatabaseDerive, EnvClient,
};
mod glyphs;

pub(crate) const CONTRACT_ADDRESS: [u8; 32] = [
    40, 76, 4, 220, 239, 185, 174, 223, 218, 252, 223, 244, 153, 121, 154, 92, 108, 72, 251, 184,
    70, 166, 134, 111, 165, 220, 84, 86, 184, 196, 55, 73,
];

#[derive(DatabaseDerive, Clone, Debug)]
#[with_name("glyphs")]
struct ColorGlyph {
    minter: ScVal,
    owner: ScVal,
    colors: ScVal,
    hash: ScVal,
    minted: ScVal,
    scraped: ScVal,
    owned: ScVal,
}

#[no_mangle]
pub extern "C" fn on_close() {
    let env = EnvClient::new();
    let contract_events: Vec<ContractEvent> = env
        .reader()
        .soroban_events()
        .into_iter()
        .filter(|event| event.contract_id == Some(Hash(CONTRACT_ADDRESS)))
        .collect();

    env.log().debug(
        format!(
            "Processing ledger {} events",
            env.reader().ledger_sequence()
        ),
        None,
    );

    for event in contract_events {
        let ContractEventBody::V0(event) = &event.body;
        let action: Symbol = env.from_scval(&event.topics[0]);

        if action == Symbol::new(&env.soroban(), "minted") {
            env.log().debug("minted", None);
            let glyph = get_glyph(&env, event.clone(), true);
            env.log().debug(format!("{:?}", glyph), None);
            insert_or_update_glyph(&env, glyph, event.topics[1].clone())
        } else if action == Symbol::new(&env.soroban(), "minting") {
            env.log().debug("minting", None);
            let glyph = get_glyph(&env, event.clone(), false);
            insert_or_update_glyph(&env, glyph, event.topics[1].clone())
        } else if action == Symbol::new(&env.soroban(), "scrape_glyph") {
            env.log().debug("scrape_glyph", None);
            let hash = event.data.clone();
            let glyphs: std::vec::Vec<ColorGlyph> = ColorGlyph::read_to_rows(&env)
                .into_iter()
                .filter(|glyph| glyph.hash == hash)
                .collect();
            let mut glyph = glyphs[0].clone();
            glyph.scraped = env.to_scval(true);
            env.update()
                .column_equal_to_xdr("hash", &hash)
                .execute(&glyph);
        } else if action == Symbol::new(&env.soroban(), "transfer_glyph") {
            env.log().debug("transfer_glyph", None);
            let to_filter = ColorGlyph::read_to_rows(&env);
            let hash = event.data.clone();

            let mut preiously_owned = {
                let glyphs: std::vec::Vec<ColorGlyph> = to_filter
                    .into_iter()
                    .filter(|glyph| glyph.hash == hash)
                    .collect();
                glyphs[0].clone()
            };
            let mut new_owned = preiously_owned.clone();

            preiously_owned.owned = env.to_scval(false);
            new_owned.owned = env.to_scval(true);
            new_owned.owner = event.topics[2].clone();

            env.update()
                .column_equal_to_xdr("hash", &hash)
                .column_equal_to_xdr("owner", &preiously_owned.owner)
                .execute(&preiously_owned);
            env.update()
                .column_equal_to_xdr("hash", &hash)
                .column_equal_to_xdr("owner", &new_owned.owner)
                .execute(&new_owned);
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct ColorClient {
    color: u32,
    amount: u32
}

#[derive(Serialize, Deserialize)]
pub struct ColorMintRequest {
    source: String,
    colors: Vec<ColorClient>,
}

#[no_mangle]
pub extern "C" fn simulate_color_mint() {
    let env = EnvClient::empty();
    let request: ColorMintRequest = env.read_request_body();
    
    let source = stellar_strkey::ed25519::PublicKey::from_string(&request.source).unwrap().0;
    let source = AccountId(zephyr_sdk::soroban_sdk::xdr::PublicKey::PublicKeyTypeEd25519(Uint256(source)));

    let function_name = Symbol::new(&env.soroban(), "colors_mine");
    let source_addr = Address::from_string(&SorobanString::from_str(&env.soroban(), &request.source));
    let mut colors = Map::new(&env.soroban());
    for color in request.colors {
        colors.set(color.color, color.amount);
    }

    let ScVal::Symbol(function_name) = env.to_scval(function_name) else {panic!()};
    let args = vec![env.to_scval(source_addr), env.to_scval(colors), ScVal::Void, ScVal::Void];

    let resp = env.simulate(source, zephyr_sdk::soroban_sdk::xdr::HostFunction::InvokeContract(InvokeContractArgs {
        contract_address: zephyr_sdk::soroban_sdk::xdr::ScAddress::Contract(Hash(CONTRACT_ADDRESS)),
        function_name,
        args: args.try_into().unwrap()
    })).unwrap();

    env.conclude(resp)
}

// NB: the code below is experimental.

#[derive(Serialize, Deserialize)]
pub struct OffersByHashRequest {
    hash: String
}

// NB: the below code describes a serverless function that executes quite an
// intensive call, thus doesn't rely on calling the host environment to improve
// execution speed. 
//
// This kind of behaviour is generally discouraged as requests could take up
// to 1s or more if the contract has many entries.
//
// We use this approach here only for demonstration purposes and will likely
// soon move this logic to the ingestion process and only keep the function for running
// a very quick catchup. 

// NB: the SDK doesn't have yet the helpers to deal with such functions more easily, 
// hence the code you're seeing here is very verbose. The hope is that it will make you 
// appreciate all the code above and Zephyr's built-in compatibility with Soroban
// types.

#[derive(Serialize, Deserialize)]
pub struct UnifiedResponse {
    owned_colors: Vec<Color>,
    mined_colors: Vec<Color>,
    glyph_offers: BTreeMap<String, Vec<OfferType>>
}

#[derive(Serialize, Deserialize)]
pub enum OfferType {
    Glyph(String),
    Asset(String, i128),
    AssetSell(String, String, i128)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Color {
    miner: String,
    owner: String,
    color: u32,
    amount: u32
}

#[derive(Serialize, Deserialize)]
pub struct UnifiedRequest {
    user: String
}

#[no_mangle]
pub extern "C" fn unified_cg_query() {
    let env = EnvClient::empty();
    let from_ledger = env.read_contract_entries(CONTRACT_ADDRESS).unwrap();
    let request: UnifiedRequest = env.read_request_body();

    let mut owned_colors = Vec::new();
    let mut mined_colors = Vec::new();
    let mut glyph_offers = BTreeMap::new();

    for entry in from_ledger {
        if let ScVal::Vec(Some(vec))  = entry.key {
            if let Some(ScVal::Symbol(symbol)) = vec.get(0) {
                if symbol.to_string() == "Color" {
                    let ScVal::Address(miner) = vec.get(1).unwrap() else {panic!()};
                    let ScVal::Address(owner) = vec.get(2).unwrap() else {panic!()};
                    let ScVal::U32(color) = vec.get(3).unwrap() else {panic!()};
                    
                    let LedgerEntryData::ContractData(data) = entry.entry.data else {panic!()};
                    let ScVal::U32(val) = data.val else {panic!()};

                    let color = Color {
                        miner: miner.to_string(),
                        owner: owner.to_string(),
                        color: *color,
                        amount: val
                    };
                    
                    if owner.to_string() == request.user {
                        owned_colors.push(color.clone())
                    } 
                    
                    if miner.to_string() == request.user {
                        mined_colors.push(color)
                    }
                } else if symbol.to_string() == "GlyphOffer" {
                    let hash = bytes_to_str(vec.get(1).unwrap());

                    let LedgerEntryData::ContractData(data) = entry.entry.data else {panic!()};
                    let ScVal::Vec(Some(offers)) = data.val else {panic!()};

                    let mut mapped_offers = Vec::new();

                    for offer in offers.to_vec() {
                        let ScVal::Vec(Some(offer)) = offer else {panic!()};
                        if let Some(ScVal::Symbol(symbol)) = offer.get(0) {
                            if symbol.to_string() == "Glyph" {
                                let hash = bytes_to_str(offer.get(1).unwrap());
                                mapped_offers.push(OfferType::Glyph(hash.into()));
                            } else if symbol.to_string() == "Asset" {
                                let ScVal::Address(addr) = vec.get(1).unwrap() else {panic!()};
                                let ScVal::I128(parts) = vec.get(2).unwrap() else {panic!()};
                                mapped_offers.push(OfferType::Asset(addr.to_string(), parts_to_i128(parts)));
                            } else if symbol.to_string() == "AssetSell" {
                                let ScVal::Address(addr) = vec.get(1).unwrap() else {panic!()};
                                let ScVal::Address(addr1) = vec.get(2).unwrap() else {panic!()};
                                let ScVal::I128(parts) = vec.get(3).unwrap() else {panic!()};
                                mapped_offers.push(OfferType::AssetSell(addr.to_string(), addr1.to_string(), parts_to_i128(parts)));
                            }
                        }
                    }

                    glyph_offers.insert(hash.to_string(), mapped_offers);
                }
            }
        }
    }

    env.conclude(UnifiedResponse {
        mined_colors,
        owned_colors,
        glyph_offers
    });
}


fn bytes_to_str(bytes: &ScVal) -> String {
    let ScVal::Bytes(hash) = bytes else {panic!()};
    let hash = hex::encode(hash.0.to_vec());

    hash
}