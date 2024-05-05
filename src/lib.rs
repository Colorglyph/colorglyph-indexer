use glyphs::{get_glyph, insert_or_update_glyph};
use zephyr_sdk::{prelude::*, soroban_sdk::{xdr::{ContractEvent, ContractEventBody, Hash, ScVal}, Symbol}, DatabaseDerive, EnvClient};

mod glyphs;

pub(crate) const CONTRACT_ADDRESS: [u8;32] = [40, 76, 4, 220, 239, 185, 174, 223, 218, 252, 223, 244, 153, 121, 154, 92, 108, 72, 251, 184, 70, 166, 134, 111, 165, 220, 84, 86, 184, 196, 55, 73];

#[derive(DatabaseDerive, Clone, Debug)]
#[with_name("glyphs")]
struct ColorGlyph {
    minter: ScVal,
    owner: ScVal,
    colors: ScVal,
    hash: ScVal,
    minted: ScVal,
    scraped: ScVal,
    owned: ScVal
}


#[no_mangle]
pub extern "C" fn on_close() {
    let env = EnvClient::new();
    let contract_events: Vec<ContractEvent> = env.reader().soroban_events().into_iter().filter(|event| event.contract_id == Some(Hash(CONTRACT_ADDRESS))).collect();
    
    env.log().debug(format!("Processing ledger {} events", env.reader().ledger_sequence()), None);

    for event in contract_events {
        let ContractEventBody::V0(event) = &event.body;
        let action: Symbol = env.from_scval(&event.topics[0]);

        if action == Symbol::new(&env.soroban(), "minted") {
            env.log().debug("minted", None);
            let glyph = get_glyph(&env, event.clone(), true);
            env.log().debug(format!("{:?}", glyph), None);
            insert_or_update_glyph(&env, glyph, event.topics[1].clone())
        }  else if action == Symbol::new(&env.soroban(), "minting") {
            env.log().debug("minting", None);
            let glyph = get_glyph(&env, event.clone(), false);
            insert_or_update_glyph(&env, glyph, event.topics[1].clone())
        } else if action == Symbol::new(&env.soroban(), "scrape_glyph") {
            env.log().debug("scrape_glyph", None);
            let hash = event.data.clone();
            let glyphs: std::vec::Vec<ColorGlyph> = ColorGlyph::read_to_rows(&env).into_iter().filter(|glyph| glyph.hash == hash).collect();
            let mut glyph = glyphs[0].clone();
            glyph.scraped = env.to_scval(true);
            env.update().column_equal_to_xdr("hash", &hash).execute(&glyph);
        } else if action == Symbol::new(&env.soroban(), "transfer_glyph") {
            env.log().debug("transfer_glyph", None);
            let to_filter = ColorGlyph::read_to_rows(&env);
            let hash = event.data.clone();
            
            let mut preiously_owned = {
                let glyphs: std::vec::Vec<ColorGlyph> = to_filter.into_iter().filter(|glyph| glyph.hash == hash).collect();
                glyphs[0].clone()
            };
            let mut new_owned = preiously_owned.clone();

            preiously_owned.owned = env.to_scval(false);
            new_owned.owned = env.to_scval(true);
            new_owned.owner = event.topics[2].clone();

            env.update().column_equal_to_xdr("hash", &hash).column_equal_to_xdr("owner", &preiously_owned.owner).execute(&preiously_owned);
            env.update().column_equal_to_xdr("hash", &hash).column_equal_to_xdr("owner", &new_owned.owner).execute(&new_owned);
        }
    }
}
