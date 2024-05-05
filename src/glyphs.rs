use zephyr_sdk::{
    soroban_sdk::{
        self, contracttype,
        xdr::{ContractEventV0, ScVal},
        Address, BytesN, Map, Vec,
    },
    DatabaseInteract, EnvClient,
};

use crate::{ColorGlyph, CONTRACT_ADDRESS};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum StorageKey {
    OwnerAddress,
    TokenAddress,
    FeeAddress,
    MaxEntryLifetime,
    MaxPaymentCount,
    MineMultiplier,
    MinterRoyaltyRate,
    MinerRoyaltyRate,
    Color(Address, Address, u32),
    Colors(Address),
    Glyph(BytesN<32>),
    GlyphOwner(BytesN<32>),
    GlyphMinter(BytesN<32>),
    GlyphOffer(BytesN<32>),
    AssetOffer(BytesN<32>, Address, i128),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    pub width: u32,
    pub length: u32,
    pub colors: Map<Address, Map<u32, Vec<u32>>>,
}

pub(crate) fn get_glyph_colors_from_ledger(env: &EnvClient, hash: BytesN<32>) -> ScVal {
    let key = StorageKey::Glyph(hash);
    let glyph: Glyph = env
        .read_contract_entry_by_key(CONTRACT_ADDRESS, key)
        .unwrap()
        .unwrap();
    env.to_scval(glyph.colors)
}

pub(crate) fn get_minter_colors_from_ledger(env: &EnvClient, minter: Address) -> ScVal {
    let key = StorageKey::Colors(minter);
    let colors: Map<Address, Map<u32, Vec<u32>>> = env
        .read_contract_entry_by_key(CONTRACT_ADDRESS, key)
        .unwrap()
        .unwrap_or(Map::new(&env.soroban()));
    env.to_scval(colors)
}

pub(crate) fn get_glyph(env: &EnvClient, event: ContractEventV0, minted: bool) -> ColorGlyph {
    ColorGlyph {
        minter: event.topics[1].clone(),
        owner: if let ScVal::Void = event.topics[2] {
            event.topics[1].clone()
        } else {
            event.topics[2].clone()
        },
        colors: if minted {
            get_glyph_colors_from_ledger(env, env.from_scval(&event.data))
        } else {
            get_minter_colors_from_ledger(env, env.from_scval(&event.topics[1].clone()))
        },
        hash: if minted {
            event.data.clone()
        } else {
            event.topics[1].clone()
        },
        minted: env.to_scval(minted),
        scraped: env.to_scval(false),
        owned: env.to_scval(true),
    }
}

pub(crate) fn insert_or_update_glyph(env: &EnvClient, glyph: ColorGlyph, minter: ScVal) {
    let glyphs: std::vec::Vec<ColorGlyph> = ColorGlyph::read_to_rows(env)
        .into_iter()
        .filter(|glyph| glyph.hash == minter)
        .collect();

    if glyphs.len() > 0 {
        env.update()
            .column_equal_to_xdr("hash", &minter)
            .execute(&glyph);
    } else {
        glyph.put(env)
    }
}
