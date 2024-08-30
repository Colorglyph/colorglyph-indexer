use colorglyph::types::{Glyph, StorageKey};
use serde::{Deserialize, Serialize};
use stellar_xdr::curr::ScMap;
use zephyr_sdk::{
    prelude::*,
    soroban_sdk::{xdr::{
        FeeBumpTransaction, FeeBumpTransactionEnvelope, FeeBumpTransactionInnerTx, Hash,
        HostFunction, InnerTransactionResult, InnerTransactionResultPair,
        InnerTransactionResultResult, InvokeContractArgs, InvokeHostFunctionOp,
        InvokeHostFunctionResult, LedgerEntry, LedgerEntryChange, LedgerEntryChanges,
        LedgerEntryData, LedgerKey, Operation, OperationBody, OperationMeta, OperationResult,
        OperationResultTr, ScAddress, ScVal, ToXdr, TransactionEnvelope, TransactionMeta,
        TransactionMetaV3, TransactionResult, TransactionResultMeta, TransactionResultPair,
        TransactionResultResult, TransactionV1Envelope, VecM,
    }, FromVal},
    utils::address_to_alloc_string,
    DatabaseDerive, EnvClient,
};

pub(crate) const CONTRACT_ADDRESS: [u8; 32] = [
    // 40, 76, 4, 220, 239, 185, 174, 223, 218, 252, 223, 244, 153, 121, 154, 92, 108, 72, 251, 184,
    // 70, 166, 134, 111, 165, 220, 84, 86, 184, 196, 55, 73,
    35, 153, 28, 126, 10, 228, 176, 244, 141, 44, 127, 232, 35, 149, 106, 117, 122, 30, 228, 24,
    162, 111, 254, 172, 91, 128, 129, 68, 223, 102, 102, 140,
];

// TODO we need to save the width of the glyph as well or else we don't know how to display it
// TODO we don't need to save minted, scraped and owned all separately. We just need to know if the glyph is scraped or not which is actually determined by if the width is > 0
#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("colors")]
pub struct ZephyrColor {
    miner: String,
    owner: String,
    color: u32,
    amount: u32,
}

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("glyphs")]
pub struct ZephyrGlyph {
    hash: Vec<u8>,
    owner: String,
    minter: String,
    width: u32,
    length: u32,
    colors: ScVal,
}

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("offers")]
pub struct ZephyrOffer {
    seller: String,
    selling: String,
    buying: String,
    amount: i128,
    active: bool, // TODO may need to switch to a number value if booleans aren't supported yet
}

// TODO would be nice to know what we can store in the above. Do Addresses work? Do BytesN<32> work? Can you store custom enum types?

#[no_mangle]
pub extern "C" fn on_close() {
    let env = EnvClient::new();
    // let contract_events: Vec<ContractEvent> = env
    //     .reader()
    //     .soroban_events()
    //     .into_iter()
    //     .filter(|event| event.contract_id == Some(Hash(CONTRACT_ADDRESS)))
    //     .collect();

    // env.reader().tx_processing().into_iter()

    for (tx_envelope, transaction_result_meta) in env.reader().envelopes_with_meta().into_iter() {
        process_on_close(&env, tx_envelope, transaction_result_meta);

        // let TransactionResultMeta {
        //     tx_apply_processing,
        //     ..
        // } = tx_result_meta;

        // match tx_apply_processing {
        //     TransactionMeta::V3(tx_meta) => for op in tx_meta.operations.into_iter() {},
        //     _ => {}
        // }
    }

    // let EntryChanges { state, removed, updated, created } = env.reader().v1_success_ledger_entries();

    // for change in state.into_iter() {}
    // for change in removed.into_iter() {}
    // for change in updated.into_iter() {}
    // for change in created.into_iter() {}

    // env.log().debug(
    //     format!(
    //         "Processing ledger {} events",
    //         env.reader().ledger_sequence()
    //     ),
    //     None,
    // );

    // for event in contract_events {
    //     let ContractEventBody::V0(event) = &event.body;
    //     let action: Symbol = env.from_scval(&event.topics[0]);

    //     if action == Symbol::new(&env.soroban(), "minted") {
    //         env.log().debug("minted", None);

    //         let glyph = get_glyph(&env, event.clone(), true);

    //         env.log().debug(format!("{:?}", glyph), None);

    //         insert_or_update_glyph(&env, glyph, event.topics[1].clone())
    //     } else if action == Symbol::new(&env.soroban(), "minting") {
    //         env.log().debug("minting", None);

    //         let glyph = get_glyph(&env, event.clone(), false);

    //         insert_or_update_glyph(&env, glyph, event.topics[1].clone())
    //     } else if action == Symbol::new(&env.soroban(), "scrape_glyph") {
    //         env.log().debug("scrape_glyph", None);

    //         let to_filter = ColorGlyph::read_to_rows(&env, None);
    //         let hash = event.data.clone();
    //         let mut glyph = {
    //             let glyphs: std::vec::Vec<ColorGlyph> = to_filter
    //                 .into_iter()
    //                 .filter(|glyph| glyph.hash == hash)
    //                 .collect();
    //             glyphs[0].clone()
    //         };

    //         glyph.width = env.to_scval(0);

    //         env.update()
    //             .column_equal_to_xdr("hash", &hash)
    //             .execute(&glyph);
    //     } else if action == Symbol::new(&env.soroban(), "transfer_glyph") {
    //         env.log().debug("transfer_glyph", None);

    //         let to_filter = ColorGlyph::read_to_rows(&env, None);
    //         let hash = event.data.clone();
    //         let mut glyph = {
    //             let glyphs: std::vec::Vec<ColorGlyph> = to_filter
    //                 .into_iter()
    //                 .filter(|glyph| glyph.hash == hash)
    //                 .collect();
    //             glyphs[0].clone()
    //         };

    //         glyph.owner = event.topics[2].clone();

    //         env.update()
    //             .column_equal_to_xdr("hash", &hash)
    //             .execute(&glyph);
    //     }
    // }
}

fn process_on_close(
    env: &EnvClient,
    tx_envelope: &TransactionEnvelope,
    transaction_result_meta: &TransactionResultMeta,
) {
    let TransactionResultMeta {
        result,
        tx_apply_processing,
        ..
    } = transaction_result_meta;
    let TransactionResultPair { result, .. } = result;
    let TransactionResult { result, .. } = result;

    match result {
        TransactionResultResult::TxFeeBumpInnerSuccess(tx) => {
            let InnerTransactionResultPair { result, .. } = tx;
            let InnerTransactionResult { result, .. } = result;

            match result {
                InnerTransactionResultResult::TxSuccess(results) => {
                    process_operation_result(&env, results, tx_envelope, tx_apply_processing)
                }
                _ => {}
            }
        }
        TransactionResultResult::TxSuccess(results) => {
            process_operation_result(&env, results, tx_envelope, tx_apply_processing)
        }
        _ => {}
    }
}

fn process_operation_result(
    env: &EnvClient,
    results: &VecM<OperationResult>,
    tx_envelope: &TransactionEnvelope,
    tx_apply_processing: &TransactionMeta,
) {
    for result in results.iter() {
        match result {
            OperationResult::OpInner(tr) => match tr {
                OperationResultTr::InvokeHostFunction(result) => match result {
                    InvokeHostFunctionResult::Success(_) => match tx_apply_processing {
                        TransactionMeta::V3(meta) => {
                            let TransactionMetaV3 { operations, .. } = meta;

                            for operation in operations.iter() {
                                let OperationMeta { changes, .. } = operation;

                                match tx_envelope {
                                    TransactionEnvelope::Tx(TransactionV1Envelope {
                                        tx, ..
                                    }) => {
                                        for Operation { body, .. } in tx.operations.iter() {
                                            match body {
                                                OperationBody::InvokeHostFunction(op) => {
                                                    process_invoke_host_function_op(
                                                        &env, op, changes,
                                                    )
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    TransactionEnvelope::TxFeeBump(
                                        FeeBumpTransactionEnvelope { tx, .. },
                                    ) => {
                                        let FeeBumpTransaction { inner_tx, .. } = tx;

                                        match inner_tx {
                                            FeeBumpTransactionInnerTx::Tx(
                                                TransactionV1Envelope { tx, .. },
                                            ) => {
                                                for Operation { body, .. } in tx.operations.iter() {
                                                    match body {
                                                        OperationBody::InvokeHostFunction(op) => {
                                                            process_invoke_host_function_op(
                                                                &env, op, changes,
                                                            )
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
    }
}

fn process_invoke_host_function_op(
    env: &EnvClient,
    op: &InvokeHostFunctionOp,
    changes: &LedgerEntryChanges,
) {
    let InvokeHostFunctionOp { host_function, .. } = op;

    match host_function {
        HostFunction::InvokeContract(op) => {
            let InvokeContractArgs {
                contract_address,
                function_name,
                args,
            } = op;

            if *contract_address == ScAddress::Contract(Hash(CONTRACT_ADDRESS)) {
                // colors_mine
                // colors_transfer

                // glyph_mint
                // glyph_transfer
                // glyph_scrape

                // offer_post
                // offer_delete

                // if function_name.to_string() == String::from_str("hello").unwrap() {

                // }

                for change in changes.iter() {
                    match change {
                        LedgerEntryChange::Removed(key) => process_ledger_key(&env, key, 0),
                        LedgerEntryChange::Created(LedgerEntry { data, .. }) => {
                            process_ledger_entry_data(&env, data, 1)
                        }
                        LedgerEntryChange::Updated(LedgerEntry { data, .. }) => {
                            process_ledger_entry_data(&env, data, 2)
                        }
                        _ => {}
                    }
                }
            }
        }
        _ => {}
    }
}

fn process_ledger_entry_data(env: &EnvClient, data: &LedgerEntryData, kind: u8) {
    match data {
        LedgerEntryData::ContractData(entry) => {
            let key = env.try_from_scval::<StorageKey>(&entry.key);

            match key {
                Ok(key) => {
                    match key {
                        StorageKey::Color(miner, owner, color) => {
                            let color = ZephyrColor {
                                miner: address_to_alloc_string(env, miner),
                                owner: address_to_alloc_string(env, owner),
                                color,
                                amount: env.from_scval(&entry.val),
                            };

                            env.put(&color);
                        }
                        StorageKey::Glyph(hash) => {
                            let glyph: Glyph = env.from_scval(&entry.val);

                            // env.log().debug(
                            //     format!("{} {} {:?} {:?}", kind, entry.contract, hash, glyph,),
                            //     None,
                            // );

                            let test = glyph.colors.to_xdr(&env.soroban()).to_alloc_vec();
                            let test = ScMap::from_xdr(test, Limits::none()).unwrap();

                            let glyph = ZephyrGlyph {
                                hash: hash.to_array().to_vec(),
                                owner: String::default(),
                                minter: String::default(),
                                width: glyph.width,
                                length: glyph.length,
                                colors: ScVal::Map(Some(test))
                            };

                            env.put(&glyph);
                        }
                        // GlyphOwner(BytesN<32>),
                        // GlyphMinter(BytesN<32>),
                        // GlyphOffer(BytesN<32>),
                        // AssetOffer(BytesN<32>, Address, i128), // glyph, sac, amount
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn process_ledger_key(env: &EnvClient, key: &LedgerKey, kind: u8) {
    match key {
        LedgerKey::ContractData(data) => {
            let key = env.try_from_scval::<StorageKey>(&data.key);

            match key {
                Ok(key) => {
                    match key {
                        StorageKey::Color(miner, owner, color) => {}
                        StorageKey::Glyph(hash) => {}
                        // GlyphOwner(BytesN<32>),
                        // GlyphMinter(BytesN<32>),
                        // GlyphOffer(BytesN<32>),
                        // AssetOffer(BytesN<32>, Address, i128), // glyph, sac, amount
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}

#[derive(Serialize, Deserialize)]
pub struct BackfillRequest {
    envelope_xdr: String,
    result_meta_xdr: String,
    result_xdr: String,
}

#[no_mangle]
pub extern "C" fn backfill() {
    let env = EnvClient::empty();
    let request: BackfillRequest = env.read_request_body();

    let tx_envelope =
        TransactionEnvelope::from_xdr_base64(request.envelope_xdr, Limits::none()).unwrap();
    let transaction_result_meta = TransactionResultMeta {
        result: TransactionResultPair {
            transaction_hash: Hash([0; 32]),
            result: TransactionResult::from_xdr_base64(request.result_xdr, Limits::none()).unwrap(),
        },
        fee_processing: LedgerEntryChanges(vec![].try_into().unwrap()),
        tx_apply_processing: TransactionMeta::from_xdr_base64(
            request.result_meta_xdr,
            Limits::none(),
        )
        .unwrap(),
    };

    process_on_close(&env, &tx_envelope, &transaction_result_meta);

    env.conclude("OK");
}

#[derive(Serialize, Deserialize)]
pub struct GetColorsRequest {
    owner: String,
}

#[no_mangle]
pub extern "C" fn get_colors() {
    let env = EnvClient::empty();
    let request: GetColorsRequest = env.read_request_body();

    let colors: Vec<ZephyrColor> = env
        .read_filter()
        .column_equal_to("owner", request.owner)
        .read()
        .unwrap();

    env.conclude(&colors);
}

// #[derive(Serialize, Deserialize)]
// pub struct ColorClient {
//     color: u32,
//     amount: u32,
// }

// #[derive(Serialize, Deserialize)]
// pub struct ColorMintRequest {
//     source: String,
//     colors: Vec<ColorClient>,
// }

// #[no_mangle]
// pub extern "C" fn simulate_color_mint() {
//     let env = EnvClient::empty();
//     let request: ColorMintRequest = env.read_request_body();

//     let function_name = Symbol::new(&env.soroban(), "colors_mine");
//     let source_addr =
//         Address::from_string(&SorobanString::from_str(&env.soroban(), &request.source));
//     let mut colors = Map::new(&env.soroban());

//     for color in request.colors {
//         colors.set(color.color, color.amount);
//     }

//     let resp = env.simulate_contract_call(
//         request.source,
//         CONTRACT_ADDRESS,
//         function_name,
//         vec![
//             &env.soroban(),
//             source_addr.into_val(env.soroban()),
//             colors.into_val(env.soroban()),
//             ().into_val(env.soroban()),
//             ().into_val(env.soroban()),
//         ],
//     );
//     env.conclude(resp.unwrap())
// }

// NB: the code below is experimental.

// #[derive(Serialize, Deserialize)]
// pub struct OffersByHashRequest {
//     hash: String,
// }

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

// #[derive(Serialize, Deserialize)]
// pub struct UnifiedResponse {
//     owned_colors: Vec<Color>,
//     // mined_colors: Vec<Color>,
//     glyph_offers: BTreeMap<String, Vec<OfferType>>,
// }

// #[derive(Serialize, Deserialize)]
// pub enum OfferType {
//     Glyph(String),
//     Asset(String, i128),
//     AssetSell(String, String, i128),
// }

// #[derive(Serialize, Deserialize, Clone)]
// pub struct Color(u32, u32);

// #[derive(Serialize, Deserialize)]
// pub struct UnifiedRequest {
//     user: String,
// }

// #[no_mangle]
// pub extern "C" fn unified_cg_query() {
//     let env = EnvClient::empty();
//     let from_ledger = env.read_contract_entries(CONTRACT_ADDRESS).unwrap();
//     let request: UnifiedRequest = env.read_request_body();

//     let mut owned_colors = Vec::new();
//     // let mut mined_colors = Vec::new();
//     let mut glyph_offers = BTreeMap::new();

//     for entry in from_ledger {
//         if let ScVal::Vec(Some(vec)) = entry.key {
//             if let Some(ScVal::Symbol(symbol)) = vec.get(0) {
//                 if symbol.to_string() == "Color" {
//                     // let ScVal::Address(miner) = vec.get(1).unwrap() else {
//                     //     panic!()
//                     // };
//                     let ScVal::Address(owner) = vec.get(2).unwrap() else {
//                         panic!()
//                     };
//                     let ScVal::U32(color) = vec.get(3).unwrap() else {
//                         panic!()
//                     };

//                     let LedgerEntryData::ContractData(data) = entry.entry.data else {
//                         panic!()
//                     };
//                     let ScVal::U32(val) = data.val else { panic!() };

//                     let color = Color(*color, val);

//                     if owner.to_string() == request.user && val > 0 {
//                         owned_colors.push(color.clone())
//                     }

//                     // if miner.to_string() == request.user {
//                     //     mined_colors.push(color)
//                     // }
//                 } else if symbol.to_string() == "GlyphOffer" {
//                     let hash = bytes_to_str(vec.get(1).unwrap());

//                     let LedgerEntryData::ContractData(data) = entry.entry.data else {
//                         panic!()
//                     };
//                     let ScVal::Vec(Some(offers)) = data.val else {
//                         panic!()
//                     };

//                     let mut mapped_offers = Vec::new();

//                     for offer in offers.to_vec() {
//                         let ScVal::Vec(Some(offer)) = offer else {
//                             panic!()
//                         };

//                         if let Some(ScVal::Symbol(symbol)) = offer.get(0) {
//                             if symbol.to_string() == "Glyph" {
//                                 let hash = bytes_to_str(offer.get(1).unwrap());

//                                 mapped_offers.push(OfferType::Glyph(hash.into()));
//                             } else if symbol.to_string() == "Asset" {
//                                 env.log().debug("offer_found_0", None);

//                                 let ScVal::Address(addr) = offer.get(1).unwrap() else {
//                                     panic!()
//                                 };
//                                 let ScVal::I128(parts) = offer.get(2).unwrap() else {
//                                     panic!()
//                                 };

//                                 mapped_offers
//                                     .push(OfferType::Asset(addr.to_string(), parts_to_i128(parts)));
//                             } else if symbol.to_string() == "AssetSell" {
//                                 let ScVal::Address(addr) = offer.get(1).unwrap() else {
//                                     panic!()
//                                 };
//                                 let ScVal::Address(addr1) = offer.get(2).unwrap() else {
//                                     panic!()
//                                 };
//                                 let ScVal::I128(parts) = offer.get(3).unwrap() else {
//                                     panic!()
//                                 };

//                                 mapped_offers.push(OfferType::AssetSell(
//                                     addr.to_string(),
//                                     addr1.to_string(),
//                                     parts_to_i128(parts),
//                                 ));
//                             }
//                         }
//                     }

//                     glyph_offers.insert(hash.to_string(), mapped_offers);
//                 }
//                 // TODO AssetOffer
//             }
//         }
//     }

//     env.conclude(UnifiedResponse {
//         // mined_colors,
//         owned_colors,
//         glyph_offers,
//     });
// }

// fn bytes_to_str(bytes: &ScVal) -> String {
//     let ScVal::Bytes(hash) = bytes else { panic!() };
//     let hash = hex::encode(hash.0.to_vec());

//     hash
// }
