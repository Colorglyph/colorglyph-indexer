use core::str::FromStr;

use colorglyph::types::{Glyph, Offer, StorageKey};
use serde::{Deserialize, Serialize};
use types::{
    Change, Offers, Kind, ZephyrColor, ZephyrColorAmount, ZephyrColorEmpty, ZephyrGlyph, ZephyrGlyphEmpty, ZephyrGlyphMinter, ZephyrGlyphNoColors, ZephyrGlyphOwner, ZephyrGlyphWidthLengthColors, ZephyrOffer, ZephyrOfferActive, ZephyrOfferEmpty, ZephyrOfferNoActive
};
use zephyr_sdk::{
    prelude::*, soroban_sdk::{
        xdr::{
            AccountId, BytesM, ContractDataEntry as SorobanContractDataEntry, FeeBumpTransaction,
            FeeBumpTransactionEnvelope, FeeBumpTransactionInnerTx, Hash, HostFunction,
            InnerTransactionResult, InnerTransactionResultPair, InnerTransactionResultResult,
            Int128Parts, InvokeContractArgs, InvokeHostFunctionOp, InvokeHostFunctionResult,
            LedgerEntry, LedgerEntryChange, LedgerEntryChanges, LedgerEntryData, LedgerKey,
            LedgerKeyContractData, Operation, OperationBody, OperationMeta, OperationResult,
            OperationResultTr, PublicKey, ScAddress, ScBytes, ScVal, ToXdr, TransactionEnvelope,
            TransactionMeta, TransactionMetaV3, TransactionResult, TransactionResultMeta,
            TransactionResultPair, TransactionResultResult, TransactionV1Envelope, Uint256, VecM,
        },
        Address, Bytes, Vec as SorobanVec,
    }, utils::soroban_string_to_alloc_string, AgnosticRequest, ContractDataEntry, EnvClient, Method
};

mod types;

/* TODO clean up the code
    with the new way to simplify match hell
*/

pub const CONTRACT_ADDRESS: [u8; 32] = [
    // 40, 76, 4, 220, 239, 185, 174, 223, 218, 252, 223, 244, 153, 121, 154, 92, 108, 72, 251, 184,
    // 70, 166, 134, 111, 165, 220, 84, 86, 184, 196, 55, 73,
    35, 153, 28, 126, 10, 228, 176, 244, 141, 44, 127, 232, 35, 149, 106, 117, 122, 30, 228, 24,
    162, 111, 254, 172, 91, 128, 129, 68, 223, 102, 102, 140,
];

#[no_mangle]
pub extern "C" fn on_close() {
    let env = EnvClient::new();

    for (transaction_envelope, transaction_result_meta) in env.reader().envelopes_with_meta().iter()
    {
        process_transaction(&env, transaction_envelope, transaction_result_meta);
    }
}

fn process_transaction(
    env: &EnvClient,
    transaction_envelope: &TransactionEnvelope,
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
                InnerTransactionResultResult::TxSuccess(results) => process_operation_result(
                    &env,
                    results,
                    transaction_envelope,
                    tx_apply_processing,
                ),
                _ => {}
            }
        }
        TransactionResultResult::TxSuccess(results) => {
            process_operation_result(&env, results, transaction_envelope, tx_apply_processing)
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
                ..
                // function_name,
                // args,
            } = op;

            if *contract_address == ScAddress::Contract(Hash(CONTRACT_ADDRESS)) {
                // colors_mine
                // colors_transfer

                // glyph_mint
                // glyph_transfer
                // glyph_scrape

                // offer_post
                // offer_delete

                // if function_name.to_string() == String::from_str("hello").unwrap() {}

                for change in changes.iter() {
                    match change {
                        LedgerEntryChange::Created(LedgerEntry { data, .. }) => {
                            process_ledger_entry_data(&env, data, None, Change::Create)
                        }
                        LedgerEntryChange::Updated(LedgerEntry { data, .. }) => {
                            process_ledger_entry_data(&env, data, Some(changes), Change::Update)
                        }
                        LedgerEntryChange::Removed(key) => process_ledger_key(&env, key),
                        _ => {}
                    }
                }
            }
        }
        _ => {}
    }
}

#[derive(Serialize, Deserialize)]
pub struct CloudflareColor {
    kind: Kind,
    change: Change,
    miner: String,
    owner: String,
    color: u32,
    amount: u32
}

fn process_ledger_entry_data(
    env: &EnvClient,
    data: &LedgerEntryData,
    changes: Option<&LedgerEntryChanges>,
    change: Change
) {
    match data {
        LedgerEntryData::ContractData(SorobanContractDataEntry { key, val, .. }) => {
            if let Ok(key) = env.try_from_scval::<StorageKey>(key) {
                match &key {
                    StorageKey::Color(miner, owner, color) => {
                        // let miner = env.to_scval(miner);
                        // let owner = env.to_scval(owner);
                        // let amount: u32 = env.from_scval(val);
                        // let existing = &env
                        //     .read_filter()
                        //     .column_equal_to_xdr("miner", &miner)
                        //     .column_equal_to_xdr("owner", &owner)
                        //     .column_equal_to("color", *color)
                        //     .read::<ZephyrColorEmpty>()
                        //     .unwrap();

                        // if existing.len() == 0 {
                            // let color = ZephyrColor {
                            //     miner,
                            //     owner,
                            //     color: *color,
                            //     amount,
                            // };

                            // env.put(&color);

                            // let body = format!(
                            //     r#"{{"type": "color", "change": "{:?}", "miner": "{:?}", "owner": "{:?}", "color": "{:?}", "amount": {}}}"#,
                            //     kind,
                            //     miner.to_string(),
                            //     owner.to_string(),
                            //     color,
                            //     amount
                            // );

                            let data = CloudflareColor {
                                kind: Kind::Color,
                                change,
                                miner: soroban_string_to_alloc_string(env, miner.to_string()),
                                owner: soroban_string_to_alloc_string(env, owner.to_string()),
                                color: *color,
                                amount: env.from_scval(val)
                            };

                            let body = serde_json::to_string(&data).unwrap();

                            // let p: CloudflareColor = serde_json::from_str(data)?;

                            env.send_web_request(AgnosticRequest {
                                body: Some(body),
                                url: "https://colorglyph-worker.sdf-ecosystem.workers.dev/zephyr".into(),
                                method: Method::Post,
                                headers: vec![
                                    ("Content-Type".into(), "application/json".into()),
                                ]
                            })
                        // } else {
                        //     env.update()
                        //         .column_equal_to_xdr("miner", &miner)
                        //         .column_equal_to_xdr("owner", &owner)
                        //         .column_equal_to("color", *color)
                        //         .execute(&ZephyrColorAmount { amount })
                        //         .unwrap();
                        // }
                    }
                    StorageKey::Glyph(hash) => {
                        let hash = env.to_scval(hash.clone());
                        let glyph: Glyph = env.from_scval(val);
                        let colors = env.to_scval(glyph.colors);

                        let existing = &env
                            .read_filter()
                            .column_equal_to_xdr("hash", &hash)
                            .read::<ZephyrGlyphEmpty>()
                            .unwrap();

                        if existing.len() == 0 {
                            env.put(&ZephyrGlyph {
                                hash,
                                owner: ScVal::Void,
                                minter: ScVal::Void,
                                width: glyph.width,
                                length: glyph.length,
                                colors,
                            });
                        } else {
                            let glyph = ZephyrGlyphWidthLengthColors {
                                width: glyph.width,
                                length: glyph.length,
                                colors,
                            };

                            env.update()
                                .column_equal_to_xdr("hash", &hash)
                                .execute(&glyph)
                                .unwrap();
                        }
                    }
                    StorageKey::GlyphOwner(hash) => {
                        let hash = env.to_scval(hash.clone());
                        let existing = &env
                            .read_filter()
                            .column_equal_to_xdr("hash", &hash)
                            .read::<ZephyrGlyphEmpty>()
                            .unwrap();

                        if existing.len() > 0 {
                            env.update()
                                .column_equal_to_xdr("hash", &hash)
                                .execute(&ZephyrGlyphOwner { owner: val.clone() })
                                .unwrap();
                        }
                    }
                    StorageKey::GlyphMinter(hash) => {
                        let hash = env.to_scval(hash.clone());
                        let existing = &env
                            .read_filter()
                            .column_equal_to_xdr("hash", &hash)
                            .read::<ZephyrGlyphEmpty>()
                            .unwrap();

                        if existing.len() > 0 {
                            env.update()
                                .column_equal_to_xdr("hash", &hash)
                                .execute(&ZephyrGlyphMinter {
                                    minter: val.clone(),
                                })
                                .unwrap();
                        }
                    }
                    StorageKey::GlyphOffer(hash) => {
                        let offers: SorobanVec<Offer> = env.from_scval(val);
                        let diff_offers =
                            get_diff_offers(&env, &key, changes, &Offers::Offers(offers.clone()));
                        let owner = &env.read_contract_entry_by_scvalkey(
                            CONTRACT_ADDRESS,
                            env.to_scval(StorageKey::GlyphOwner(hash.clone())),
                        );

                        if owner.is_ok() && owner.clone().unwrap().is_some() {
                            let ContractDataEntry { entry, .. } = owner.clone().unwrap().unwrap();
                            let LedgerEntry { data, .. } = entry;

                            if let LedgerEntryData::ContractData(SorobanContractDataEntry {
                                val: owner,
                                ..
                            }) = data
                            {
                                // Add or update
                                for offer in offers.iter() {
                                    let seller: ScVal = owner.clone();
                                    let selling: ScVal = env.to_scval(hash.clone());
                                    let buying: ScVal;
                                    let amount: ScVal;

                                    match offer {
                                        // Selling a glyph for a glyph
                                        Offer::Glyph(buying_hash) => {
                                            buying = env.to_scval(buying_hash);
                                            amount = ScVal::Void;
                                        }
                                        // Selling a glyph for an asset
                                        Offer::Asset(sac, a) => {
                                            buying = env.to_scval(sac);
                                            amount = ScVal::I128(
                                                // The amount of the buying asset the seller wants
                                                Int128Parts {
                                                    hi: (a >> 64) as i64,
                                                    lo: a as u64,
                                                },
                                            );
                                        }
                                        _ => {
                                            panic!("Invalid offer type")
                                        }
                                    }

                                    // update if exists, otherwise put
                                    let existing = env
                                        .read_filter()
                                        .column_equal_to_xdr("seller", &seller)
                                        .column_equal_to_xdr("selling", &selling)
                                        .column_equal_to_xdr("buying", &buying)
                                        .column_equal_to_xdr("amount", &amount)
                                        .read::<ZephyrOfferEmpty>()
                                        .unwrap();

                                    if existing.len() == 0 {
                                        env.put(&ZephyrOffer {
                                            seller,
                                            selling,
                                            buying,
                                            amount,
                                            active: ScVal::Bool(true),
                                        });
                                    } else {
                                        env.update()
                                            .column_equal_to_xdr("seller", &seller)
                                            .column_equal_to_xdr("selling", &selling)
                                            .column_equal_to_xdr("buying", &buying)
                                            .column_equal_to_xdr("amount", &amount)
                                            .execute(&ZephyrOfferActive {
                                                active: ScVal::Bool(true),
                                            })
                                            .unwrap();
                                    }
                                }

                                // Remove if exists
                                if let Some(offers) = diff_offers {
                                    if let Offers::Offers(offers) = offers {
                                        for offer in offers.iter() {
                                            let buying: ScVal;
                                            let amount: ScVal;

                                            match offer {
                                                Offer::Glyph(buying_hash) => {
                                                    buying = env.to_scval(buying_hash);
                                                    amount = ScVal::Void;
                                                }
                                                Offer::Asset(sac, a) => {
                                                    buying = env.to_scval(sac); // The asset the seller wants
                                                    amount = ScVal::I128(
                                                        // The amount of the buying asset the seller wants
                                                        Int128Parts {
                                                            hi: (a >> 64) as i64,
                                                            lo: a as u64,
                                                        },
                                                    );
                                                }
                                                _ => {
                                                    panic!("Invalid offer type")
                                                }
                                            }

                                            env.update()
                                                .column_equal_to_xdr("seller", &owner.clone())
                                                .column_equal_to_xdr("selling", &env.to_scval(hash.clone()))
                                                .column_equal_to_xdr("buying", &buying)
                                                .column_equal_to_xdr("amount", &amount)
                                                .execute(&ZephyrOfferActive { active: ScVal::Bool(false) })
                                                .unwrap();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // All the folks (addresses) who want to buy a specific glyph (hash) with a specific asset (sac) for a specific amount
                    StorageKey::AssetOffer(hash, sac, amount) => {
                        let offers: SorobanVec<Address> = env.from_scval(val);
                        let diff_offers = get_diff_offers(
                            &env,
                            &key,
                            changes,
                            &Offers::Addresses(offers.clone()),
                        );

                        // Add or update
                        for owner in offers.iter() {
                            let seller = env.to_scval(owner);
                            let selling = env.to_scval(sac.clone());
                            let buying = env.to_scval(hash.clone());
                            let amount = ScVal::I128(Int128Parts {
                                hi: (amount >> 64) as i64,
                                lo: *amount as u64,
                            });

                            let existing = env
                                .read_filter()
                                .column_equal_to_xdr("seller", &seller)
                                .column_equal_to_xdr("selling", &selling)
                                .column_equal_to_xdr("buying", &buying)
                                .column_equal_to_xdr("amount", &amount)
                                .read::<ZephyrOfferEmpty>()
                                .unwrap();

                            if existing.len() == 0 {
                                env.put(&ZephyrOffer {
                                    seller,
                                    selling,
                                    buying,
                                    amount,
                                    active: ScVal::Bool(true),
                                });
                            } else {
                                env.update()
                                    .column_equal_to_xdr("seller", &seller)
                                    .column_equal_to_xdr("selling", &selling)
                                    .column_equal_to_xdr("buying", &buying)
                                    .column_equal_to_xdr("amount", &amount)
                                    .execute(&ZephyrOfferActive {
                                        active: ScVal::Bool(true),
                                    })
                                    .unwrap();
                            }
                        }

                        // Remove if exists
                        if let Some(offers) = diff_offers {
                            if let Offers::Addresses(offers) = offers {
                                for owner in offers.iter() {
                                    let amount = ScVal::I128(Int128Parts {
                                        hi: (amount >> 64) as i64,
                                        lo: *amount as u64,
                                    });

                                    env.update()
                                        .column_equal_to_xdr("seller", &env.to_scval(owner))
                                        .column_equal_to_xdr("selling", &env.to_scval(sac.clone()))
                                        .column_equal_to_xdr("buying", &env.to_scval(hash.clone()))
                                        .column_equal_to_xdr("amount", &amount)
                                        .execute(&ZephyrOfferActive { active: ScVal::Bool(false) })
                                        .unwrap();
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn get_diff_offers(
    env: &EnvClient,
    key: &StorageKey,
    changes: Option<&LedgerEntryChanges>,
    offers: &Offers,
) -> Option<Offers> {
    if changes.is_none() {
        return None;
    }

    for change in changes.unwrap().iter() {
        if let LedgerEntryChange::State(LedgerEntry { data, .. }) = change {
            if let LedgerEntryData::ContractData(SorobanContractDataEntry { key: k, val, .. }) =
                data
            {
                if let Ok(k) = env.try_from_scval::<StorageKey>(k) {
                    if &k == key {
                        match offers {
                            Offers::Offers(offers) => {
                                let mut change_offers = SorobanVec::new(env.soroban());

                                for offer in env.from_scval::<SorobanVec<Offer>>(val).iter() {
                                    if !offers.contains(offer.clone()) {
                                        change_offers.push_back(offer);
                                    }
                                }

                                return Some(Offers::Offers(change_offers));
                            }
                            Offers::Addresses(offers) => {
                                let mut change_offers = SorobanVec::new(env.soroban());

                                for offer in env.from_scval::<SorobanVec<Address>>(val).iter() {
                                    if !offers.contains(offer.clone()) {
                                        change_offers.push_back(offer);
                                    }
                                }

                                return Some(Offers::Addresses(change_offers));
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn process_ledger_key(env: &EnvClient, key: &LedgerKey) {
    if let LedgerKey::ContractData(LedgerKeyContractData { key, .. }) = key {
        if let Ok(key) = env.try_from_scval::<StorageKey>(key) {
            match key {
                // StorageKey::Color(miner, owner, color) => {}
                // StorageKey::Glyph(hash) => {}
                // StorageKey::GlyphOwner(hash),
                // StorageKey::GlyphMinter(hash),
                StorageKey::GlyphOffer(hash) => {
                    let selling = env.to_scval(hash.clone());
                    let owner = &env.read_contract_entry_by_scvalkey(
                        CONTRACT_ADDRESS,
                        env.to_scval(StorageKey::GlyphOwner(hash.clone())),
                    );

                    if owner.is_ok() && owner.clone().unwrap().is_some() {
                        let ContractDataEntry { entry, .. } = owner.clone().unwrap().unwrap();
                        let LedgerEntry { data, .. } = entry;

                        if let LedgerEntryData::ContractData(SorobanContractDataEntry {
                            val: owner,
                            ..
                        }) = data
                        {
                            let offers = env
                                .read_filter()
                                .column_equal_to_xdr("seller", &owner)
                                .column_equal_to_xdr("selling", &selling)
                                .read::<ZephyrOfferEmpty>()
                                .unwrap();

                            for _ in offers {
                                env.update()
                                    .column_equal_to_xdr("seller", &owner)
                                    .column_equal_to_xdr("selling", &selling)
                                    .execute(&ZephyrOfferActive {
                                        active: ScVal::Bool(false),
                                    })
                                    .unwrap();
                            }
                        }
                    }
                }
                StorageKey::AssetOffer(hash, sac, amount) => {
                    let selling = env.to_scval(hash.clone());
                    let buying = env.to_scval(sac.clone());
                    let amount = ScVal::I128(Int128Parts {
                        hi: (amount >> 64) as i64,
                        lo: amount as u64,
                    });

                    let offers = env
                        .read_filter()
                        .column_equal_to_xdr("selling", &selling)
                        .column_equal_to_xdr("buying", &buying)
                        .column_equal_to_xdr("amount", &amount)
                        .read::<ZephyrOfferEmpty>()
                        .unwrap();

                    for _ in offers {
                        env.update()
                            .column_equal_to_xdr("selling", &selling)
                            .column_equal_to_xdr("buying", &buying)
                            .column_equal_to_xdr("amount", &amount)
                            .execute(&ZephyrOfferActive {
                                active: ScVal::Bool(false),
                            })
                            .unwrap();
                    }
                }
                _ => {}
            }
        }
    }
}

fn address_string_to_scval(env: &EnvClient, address: &String) -> ScVal {
    let mut public_key = [0u8; 32];

    let public_key_bytes =
        Address::from_string_bytes(&Bytes::from_slice(&env.soroban(), address.as_bytes()));
    let public_key_bytes = public_key_bytes.to_xdr(env.soroban());

    public_key_bytes
        .slice(public_key_bytes.len() - 32..)
        .copy_into_slice(&mut public_key);

    ScVal::Address(ScAddress::Account(AccountId(
        PublicKey::PublicKeyTypeEd25519(Uint256(public_key)),
    )))
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

    let transaction_envelope =
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

    process_transaction(&env, &transaction_envelope, &transaction_result_meta);

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
    let owner = address_string_to_scval(&env, &request.owner);

    let colors = env
        .read_filter()
        .column_equal_to_xdr("owner", &owner)
        .read::<ZephyrColor>()
        .unwrap();

    env.conclude(colors);
}

#[derive(Serialize, Deserialize)]
pub struct GetGlyphsRequest {
    owner: Option<String>,
}

#[no_mangle]
pub extern "C" fn get_glyphs() {
    let env = EnvClient::empty();
    let request: GetGlyphsRequest = env.read_request_body();
    
    match request.owner {
        Some(owner) => {
            let owner = address_string_to_scval(&env, &owner);
            let glyphs = env
                .read_filter()
                .column_equal_to_xdr("owner", &owner)
                .read::<ZephyrGlyphNoColors>()
                .unwrap();

            env.conclude(&glyphs);
        }
        None => {
            let glyphs = env.read::<ZephyrGlyphNoColors>();

            env.conclude(&glyphs);
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetGlyphRequest {
    hash: String,
}

#[no_mangle]
pub extern "C" fn get_glyph() {
    let env = EnvClient::empty();
    let request: GetGlyphRequest = env.read_request_body();
    let hash = ScVal::Bytes(ScBytes(BytesM::from_str(request.hash.as_str()).unwrap()));

    let glyphs = env
        .read_filter()
        .column_equal_to_xdr("hash", &hash)
        .read::<ZephyrGlyph>()
        .unwrap();

    env.conclude(&glyphs);
}

#[derive(Serialize, Deserialize)]
pub struct GetOffersRequest {
    seller: String,
}

#[no_mangle]
pub extern "C" fn get_offers() {
    let env = EnvClient::empty();
    let request: GetOffersRequest = env.read_request_body();
    let seller = address_string_to_scval(&env, &request.seller);

    let offers = env
        .read_filter()
        .column_equal_to_xdr("seller", &seller)
        .column_equal_to_xdr("active", &ScVal::Bool(true))
        .read::<ZephyrOfferNoActive>()
        .unwrap();

    env.conclude(&offers);
}

#[no_mangle]
pub extern "C" fn debug_offers() {
    let env = EnvClient::empty();

    let offers = env.read::<ZephyrOffer>();

    env.conclude(&offers);
}
