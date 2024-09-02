use colorglyph::types::{Glyph, Offer, StorageKey};
use serde::{Deserialize, Serialize};
use zephyr_sdk::{
    prelude::*,
    soroban_sdk::{
        xdr::{
            AccountId, ContractDataEntry as SorobanContractDataEntry, FeeBumpTransaction,
            FeeBumpTransactionEnvelope, FeeBumpTransactionInnerTx, Hash, HostFunction,
            InnerTransactionResult, InnerTransactionResultPair, InnerTransactionResultResult,
            Int128Parts, InvokeContractArgs, InvokeHostFunctionOp, InvokeHostFunctionResult,
            LedgerEntry, LedgerEntryChange, LedgerEntryChanges, LedgerEntryData, LedgerKey,
            LedgerKeyContractData, Operation, OperationBody, OperationMeta, OperationResult,
            OperationResultTr, PublicKey, ScAddress, ScVal, ToXdr, TransactionEnvelope,
            TransactionMeta, TransactionMetaV3, TransactionResult, TransactionResultMeta,
            TransactionResultPair, TransactionResultResult, TransactionV1Envelope, Uint256, VecM,
        },
        Address, Bytes, Vec as SorobanVec,
    },
    ContractDataEntry, DatabaseDerive, EnvClient,
};

/* TODO clean up the code
    with the new way to simplify match hell
*/

pub(crate) const CONTRACT_ADDRESS: [u8; 32] = [
    // 40, 76, 4, 220, 239, 185, 174, 223, 218, 252, 223, 244, 153, 121, 154, 92, 108, 72, 251, 184,
    // 70, 166, 134, 111, 165, 220, 84, 86, 184, 196, 55, 73,
    35, 153, 28, 126, 10, 228, 176, 244, 141, 44, 127, 232, 35, 149, 106, 117, 122, 30, 228, 24,
    162, 111, 254, 172, 91, 128, 129, 68, 223, 102, 102, 140,
];

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("colors")]
pub struct ZephyrColor {
    miner: ScVal,
    owner: ScVal,
    color: u32,
    amount: u32,
}

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("glyphs")]
pub struct ZephyrGlyph {
    hash: ScVal,
    owner: ScVal,
    minter: ScVal,
    width: u32,
    length: u32,
    colors: ScVal,
}

#[derive(DatabaseDerive, Clone, Serialize, Debug)]
#[with_name("offers")]
pub struct ZephyrOffer {
    seller: ScVal,
    selling: ScVal,
    buying: ScVal,
    amount: ScVal, // because currently i128 is broken
    active: ScVal,
}

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
                            process_ledger_entry_data(&env, data, None)
                        }
                        LedgerEntryChange::Updated(LedgerEntry { data, .. }) => {
                            process_ledger_entry_data(&env, data, Some(changes))
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

fn process_ledger_entry_data(
    env: &EnvClient,
    data: &LedgerEntryData,
    changes: Option<&LedgerEntryChanges>,
) {
    match data {
        LedgerEntryData::ContractData(SorobanContractDataEntry { key, val, .. }) => {
            if let Ok(key) = env.try_from_scval::<StorageKey>(key) {
                match &key {
                    StorageKey::Color(miner, owner, color) => {
                        let amount = env.from_scval(val);
                        let color = ZephyrColor {
                            miner: env.to_scval(miner),
                            owner: env.to_scval(owner),
                            color: *color,
                            amount,
                        };
                        let existing = &env
                            .read_filter()
                            .column_equal_to_xdr("miner", &color.miner)
                            .column_equal_to_xdr("owner", &color.owner)
                            .column_equal_to("color", color.color)
                            .read::<ZephyrColor>()
                            .unwrap();

                        if existing.len() == 0 {
                            env.put(&color);
                        } else {
                            let mut existing = existing[0].clone();

                            existing.amount = amount;

                            env.update()
                                .column_equal_to_xdr("miner", &color.miner)
                                .column_equal_to_xdr("owner", &color.owner)
                                .column_equal_to("color", color.color)
                                .execute(&existing)
                                .unwrap();
                        }
                    }
                    StorageKey::Glyph(hash) => {
                        let glyph: Glyph = env.from_scval(val);
                        let colors = env.to_scval(glyph.colors);

                        let existing = &env
                            .read_filter()
                            .column_equal_to_xdr("hash", &env.to_scval(hash.clone()))
                            .read::<ZephyrGlyph>()
                            .unwrap();

                        if existing.len() == 0 {
                            let glyph = ZephyrGlyph {
                                hash: env.to_scval(hash.clone()),
                                owner: ScVal::Void,
                                minter: ScVal::Void,
                                width: glyph.width,
                                length: glyph.length,
                                colors,
                            };

                            env.put(&glyph);
                        } else {
                            let mut existing = existing[0].clone();

                            existing.colors = colors;
                            existing.width = glyph.width;
                            existing.length = glyph.length;

                            env.update()
                                .column_equal_to_xdr("hash", &env.to_scval(hash.clone()))
                                .execute(&existing)
                                .unwrap();
                        }
                    }
                    StorageKey::GlyphOwner(hash) => {
                        let existing = &env
                            .read_filter()
                            .column_equal_to_xdr("hash", &env.to_scval(hash.clone()))
                            .read::<ZephyrGlyph>()
                            .unwrap();

                        if existing.len() > 0 {
                            let mut existing = existing[0].clone();

                            existing.owner = val.clone();

                            env.update()
                                .column_equal_to_xdr("hash", &env.to_scval(hash.clone()))
                                .execute(&existing)
                                .unwrap();
                        }
                    }
                    StorageKey::GlyphMinter(hash) => {
                        let existing = &env
                            .read_filter()
                            .column_equal_to_xdr("hash", &env.to_scval(hash.clone()))
                            .read::<ZephyrGlyph>()
                            .unwrap();

                        if existing.len() > 0 {
                            let mut existing = existing[0].clone();

                            existing.minter = val.clone();

                            env.update()
                                .column_equal_to_xdr("hash", &env.to_scval(hash.clone()))
                                .execute(&existing)
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
                                    let o: ZephyrOffer;

                                    match offer {
                                        // Selling a glyph for a glyph
                                        Offer::Glyph(buying_hash) => {
                                            o = ZephyrOffer {
                                                seller: owner.clone(),               // glyph owner
                                                selling: env.to_scval(hash.clone()), // sell glyph hash
                                                buying: env.to_scval(buying_hash), // buy glyph hash
                                                amount: ScVal::Void,
                                                active: ScVal::Bool(true),
                                            };
                                        }
                                        // Selling a glyph for an asset
                                        Offer::Asset(sac, amount) => {
                                            o = ZephyrOffer {
                                                seller: owner.clone(),               // The owner of the glyph
                                                selling: env.to_scval(hash.clone()), // Should be the glyph
                                                buying: env.to_scval(sac), // The asset the seller wants
                                                amount: ScVal::I128(
                                                    // The amount of the buying asset the seller wants
                                                    Int128Parts {
                                                        hi: (amount >> 64) as i64,
                                                        lo: amount as u64,
                                                    },
                                                ),
                                                active: ScVal::Bool(true),
                                            };
                                        }
                                        _ => {
                                            panic!("Invalid offer type")
                                        }
                                    }

                                    // update if exists, otherwise put
                                    let existing = env
                                        .read_filter()
                                        .column_equal_to_xdr("seller", &o.seller)
                                        .column_equal_to_xdr("selling", &o.selling)
                                        .column_equal_to_xdr("buying", &o.buying)
                                        .column_equal_to_xdr("amount", &o.amount)
                                        .read::<ZephyrOffer>()
                                        .unwrap();

                                    if existing.len() == 0 {
                                        env.put(&o);
                                    } else {
                                        env.update()
                                            .column_equal_to_xdr("seller", &o.seller)
                                            .column_equal_to_xdr("selling", &o.selling)
                                            .column_equal_to_xdr("buying", &o.buying)
                                            .column_equal_to_xdr("amount", &o.amount)
                                            .execute(&o)
                                            .unwrap();
                                    }
                                }

                                // Remove if exists
                                if let Some(offers) = diff_offers {
                                    if let Offers::Offers(offers) = offers {
                                        for offer in offers.iter() {
                                            let o: ZephyrOffer;

                                            match offer {
                                                Offer::Glyph(buying_hash) => {
                                                    o = ZephyrOffer {
                                                        seller: owner.clone(),               // glyph owner
                                                        selling: env.to_scval(hash.clone()), // sell glyph hash
                                                        buying: env.to_scval(buying_hash), // buy glyph hash
                                                        amount: ScVal::Void,
                                                        active: ScVal::Bool(false),
                                                    };
                                                }
                                                Offer::Asset(sac, amount) => {
                                                    o = ZephyrOffer {
                                                        seller: owner.clone(),               // The owner of the glyph
                                                        selling: env.to_scval(hash.clone()), // Should be the glyph
                                                        buying: env.to_scval(sac), // The asset the seller wants
                                                        amount: ScVal::I128(
                                                            // The amount of the buying asset the seller wants
                                                            Int128Parts {
                                                                hi: (amount >> 64) as i64,
                                                                lo: amount as u64,
                                                            },
                                                        ),
                                                        active: ScVal::Bool(false),
                                                    };
                                                }
                                                _ => {
                                                    panic!("Invalid offer type")
                                                }
                                            }

                                            env.update()
                                                .column_equal_to_xdr("seller", &o.seller)
                                                .column_equal_to_xdr("selling", &o.selling)
                                                .column_equal_to_xdr("buying", &o.buying)
                                                .column_equal_to_xdr("amount", &o.amount)
                                                .execute(&o)
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
                            let offer = ZephyrOffer {
                                seller: env.to_scval(owner),
                                selling: env.to_scval(sac.clone()),
                                buying: env.to_scval(hash.clone()),
                                amount: ScVal::I128(Int128Parts {
                                    hi: (amount >> 64) as i64,
                                    lo: *amount as u64,
                                }),
                                active: ScVal::Bool(true),
                            };

                            let existing = env
                                .read_filter()
                                .column_equal_to_xdr("seller", &offer.seller)
                                .column_equal_to_xdr("selling", &offer.selling)
                                .column_equal_to_xdr("buying", &offer.buying)
                                .column_equal_to_xdr("amount", &offer.amount)
                                .read::<ZephyrOffer>()
                                .unwrap();

                            if existing.len() == 0 {
                                env.put(&offer);
                            } else {
                                env.update()
                                    .column_equal_to_xdr("seller", &offer.seller)
                                    .column_equal_to_xdr("selling", &offer.selling)
                                    .column_equal_to_xdr("buying", &offer.buying)
                                    .column_equal_to_xdr("amount", &offer.amount)
                                    .execute(&offer)
                                    .unwrap();
                            }
                        }

                        // Remove if exists
                        if let Some(offers) = diff_offers {
                            if let Offers::Addresses(offers) = offers {
                                for owner in offers.iter() {
                                    let offer = ZephyrOffer {
                                        seller: env.to_scval(owner),
                                        selling: env.to_scval(sac.clone()),
                                        buying: env.to_scval(hash.clone()),
                                        amount: ScVal::I128(Int128Parts {
                                            hi: (amount >> 64) as i64,
                                            lo: *amount as u64,
                                        }),
                                        active: ScVal::Bool(false),
                                    };

                                    env.update()
                                        .column_equal_to_xdr("seller", &offer.seller)
                                        .column_equal_to_xdr("selling", &offer.selling)
                                        .column_equal_to_xdr("buying", &offer.buying)
                                        .column_equal_to_xdr("amount", &offer.amount)
                                        .execute(&offer)
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

#[derive(Clone, Debug)]
pub enum Offers {
    Offers(SorobanVec<Offer>),
    Addresses(SorobanVec<Address>),
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
                                .column_equal_to_xdr("selling", &selling)
                                .read::<ZephyrOffer>()
                                .unwrap();

                            for mut offer in offers {
                                offer.seller = owner.clone();
                                offer.active = ScVal::Bool(false);

                                env.update()
                                    .column_equal_to_xdr("selling", &selling)
                                    .execute(&offer)
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
                        .read::<ZephyrOffer>()
                        .unwrap();

                    for mut offer in offers {
                        offer.active = ScVal::Bool(false);

                        env.update()
                            .column_equal_to_xdr("selling", &selling)
                            .column_equal_to_xdr("buying", &buying)
                            .column_equal_to_xdr("amount", &amount)
                            .execute(&offer)
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

    let public_key_bytes = address.to_string();
    let public_key_bytes = Address::from_string_bytes(&Bytes::from_slice(
        &env.soroban(),
        public_key_bytes.as_bytes(),
    ));
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
#[derive(Serialize, Deserialize)]
pub struct GetColorsResponse {
    miner: String,
    owner: String,
    color: u32,
    amount: u32,
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
    // This actually takes the request from ~ 1 sec to ~ 12 seconds
    // .into_iter()
    // .map(|color| {
    //     GetColorsResponse {
    //         miner: soroban_string_to_alloc_string(&env, env.from_scval::<Address>(&color.miner).to_string()),
    //         owner: soroban_string_to_alloc_string(&env, env.from_scval::<Address>(&color.owner).to_string()),
    //         color: color.color,
    //         amount: color.amount
    //     }
    // }).collect::<Vec<GetColorsResponse>>();

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
                .read::<ZephyrGlyph>()
                .unwrap();

            env.conclude(&glyphs);
        }
        None => {
            let glyphs = env
                .read::<ZephyrGlyph>();

            env.conclude(&glyphs);
        }
    }
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
        .read::<ZephyrOffer>()
        .unwrap();

    env.conclude(&offers);
}
