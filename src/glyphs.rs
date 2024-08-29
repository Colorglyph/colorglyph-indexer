use crate::{ColorGlyph, CONTRACT_ADDRESS};
use colorglyph::types::{Glyph, StorageKey};
use zephyr_sdk::{
    soroban_sdk::{
        xdr::{ContractEventV0, ScVal},
        Address, BytesN, Map, Vec,
    },
    DatabaseInteract, EnvClient,
};

pub(crate) fn get_glyph_from_ledger(env: &EnvClient, hash: BytesN<32>) -> Glyph {
    let key = StorageKey::Glyph(hash);

    env.read_contract_entry_by_key(CONTRACT_ADDRESS, key)
        .unwrap()
        .unwrap()
}

pub(crate) fn get_glyph(env: &EnvClient, event: ContractEventV0, minted: bool) -> ColorGlyph {
    let glyph = get_glyph_from_ledger(env, env.from_scval(&event.data));

    ColorGlyph {
        minter: event.topics[1].clone(),
        owner: if let ScVal::Void = event.topics[2] {
            event.topics[1].clone()
        } else {
            event.topics[2].clone()
        },
        colors: env.to_scval(glyph.colors),
        hash: if minted {
            event.data.clone()
        } else {
            event.topics[1].clone()
        },
        width: env.to_scval(glyph.width),
        length: env.to_scval(glyph.length),
    }
}

pub(crate) fn insert_or_update_glyph(env: &EnvClient, glyph: ColorGlyph, minter: ScVal) {
    let glyphs: std::vec::Vec<ColorGlyph> = ColorGlyph::read_to_rows(env, None)
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
