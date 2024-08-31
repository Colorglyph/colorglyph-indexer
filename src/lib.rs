use colorglyph::types::{Glyph, Offer, StorageKey};
use serde::{Deserialize, Serialize};
use zephyr_sdk::{
    prelude::*,
    soroban_sdk::{
        xdr::{
            ContractDataEntry as SorobanContractDataEntry, FeeBumpTransaction,
            FeeBumpTransactionEnvelope, FeeBumpTransactionInnerTx, Hash, HostFunction,
            InnerTransactionResult, InnerTransactionResultPair, InnerTransactionResultResult,
            Int128Parts, InvokeContractArgs, InvokeHostFunctionOp, InvokeHostFunctionResult,
            LedgerEntry, LedgerEntryChange, LedgerEntryChanges, LedgerEntryData, LedgerKey,
            Operation, OperationBody, OperationMeta, OperationResult, OperationResultTr, ScAddress,
            ScVal, ToXdr, TransactionEnvelope, TransactionMeta, TransactionMetaV3,
            TransactionResult, TransactionResultMeta, TransactionResultPair,
            TransactionResultResult, TransactionV1Envelope, VecM,
        },
        Address, Vec as SorobanVec,
    },
    ContractDataEntry, DatabaseDerive, EnvClient,
};

// TODO ensure we're not duplicating rows anywhere
// Can do by not filtering and just re-running catchups and ensure the size doesn't change

// TODO update with the new way to simplify match hell

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
    // buyer: ScVal,
    buying: ScVal,
    amount: ScVal, // because currently i128 is broken
    active: ScVal,
}

#[no_mangle]
pub extern "C" fn on_close() {
    let env = EnvClient::new();

    env.log().debug(format!("Processing ledger {}", env.reader().ledger_sequence()), None);

    for (transaction_envelope, transaction_result_meta) in
        env.reader().envelopes_with_meta().into_iter()
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
    env.log().debug(format!("Found {} results", results.len()), None);

    for result in results.iter() {
        match result {
            OperationResult::OpInner(tr) => match tr {
                OperationResultTr::InvokeHostFunction(result) => match result {
                    InvokeHostFunctionResult::Success(_) => match tx_apply_processing {
                        TransactionMeta::V3(meta) => {
                            env.log().debug(format!("Found meta"), None);

                            let TransactionMetaV3 { operations, .. } = meta;

                            env.log().debug(format!("Found {} operations", operations.len()), None);

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
                            let amount = env.from_scval(&entry.val);
                            let existing = &env
                                .read_filter()
                                .column_equal_to_xdr("miner", &env.to_scval(miner.clone()))
                                .column_equal_to_xdr("owner", &env.to_scval(owner.clone()))
                                .column_equal_to("color", color)
                                .read::<ZephyrColor>()
                                .unwrap();

                            if existing.len() == 0 {
                                let color = ZephyrColor {
                                    miner: env.to_scval(miner),
                                    owner: env.to_scval(owner),
                                    color,
                                    amount,
                                };

                                env.put(&color);
                            } else {
                                let mut existing = existing[0].clone();

                                existing.amount = amount;

                                env.update()
                                    .column_equal_to_xdr("miner", &env.to_scval(miner.clone()))
                                    .column_equal_to_xdr("owner", &env.to_scval(owner.clone()))
                                    .column_equal_to("color", color)
                                    .execute(&existing)
                                    .unwrap();
                            }
                        }
                        StorageKey::Glyph(hash) => {
                            let glyph: Glyph = env.from_scval(&entry.val);
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
                                // TODO compress this block and the GlyphOwner and GlyphMinter into a single function

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

                                existing.owner = entry.val.clone();

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

                                existing.minter = entry.val.clone();

                                env.update()
                                    .column_equal_to_xdr("hash", &env.to_scval(hash.clone()))
                                    .execute(&existing)
                                    .unwrap();
                            }
                        }

                        // TODO for the offers we need to keep in mind offers are stored in vectors
                        // meaning it will be difficult to dynamically remove offers unless we keep an eye on the before and after diff

                        // TODO test all the offer types
                            // Sell a glyph for a glyph
                            // Sell an asset for a glyph

                        // TODO implement update logic alongside put logic

                        StorageKey::GlyphOffer(hash) => {
                            let offers: SorobanVec<Offer> = env.from_scval(&entry.val);

                            for offer in offers.iter() {
                                match offer {
                                    // Selling a glyph for a glyph
                                    Offer::Glyph(buying_hash) => {
                                        let owner = env.read_contract_entry_by_scvalkey(
                                            CONTRACT_ADDRESS,
                                            env.to_scval(StorageKey::GlyphOwner(hash.clone())),
                                        );

                                        if owner.is_ok() && owner.clone().unwrap().is_some() {
                                            let ContractDataEntry { entry, .. } =
                                                owner.unwrap().unwrap();
                                            let LedgerEntry { data, .. } = entry;

                                            if let LedgerEntryData::ContractData(
                                                SorobanContractDataEntry { val, .. },
                                            ) = data
                                            {
                                                let offer = ZephyrOffer {
                                                    seller: val,                         // glyph owner
                                                    selling: env.to_scval(hash.clone()), // sell glyph hash
                                                    // buyer: ScVal::Void, // TODO Should we set this? Would require updating it whenever the glyph owner changed
                                                    buying: env.to_scval(buying_hash), // buy glyph hash
                                                    amount: ScVal::Void,
                                                    active: ScVal::Bool(true),
                                                };

                                                env.put(&offer);
                                            }
                                        }
                                    }
                                    // Selling a glyph for an asset
                                    Offer::Asset(sac, amount) => {
                                        let owner = env.read_contract_entry_by_scvalkey(
                                            CONTRACT_ADDRESS,
                                            env.to_scval(StorageKey::GlyphOwner(hash.clone())),
                                        );

                                        if owner.is_ok() && owner.clone().unwrap().is_some() {
                                            let ContractDataEntry { entry, .. } =
                                                owner.unwrap().unwrap();
                                            let LedgerEntry { data, .. } = entry;

                                            if let LedgerEntryData::ContractData(
                                                SorobanContractDataEntry { val, .. },
                                            ) = data
                                            {
                                                let offer = ZephyrOffer {
                                                    seller: val,                         // The owner of the glyph
                                                    selling: env.to_scval(hash.clone()), // Should be the glyph
                                                    // buyer: ScVal::Void, // There is no buyer
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

                                                env.put(&offer);
                                            }
                                        }
                                    }
                                    _ => {
                                        // NOTE: This should never happen
                                        env.log()
                                            .debug("Invalid offer found", None);
                                    }
                                }
                            }
                        }
                        // All the folks (addresses) who want to buy a specific glyph (hash) with a specific asset (sac) for a specific amount
                        StorageKey::AssetOffer(hash, sac, amount) => {
                            let offers: SorobanVec<Address> = env.from_scval(&entry.val);

                            for owner in offers.iter() {
                                let offer = ZephyrOffer {
                                    seller: env.to_scval(owner),
                                    selling: env.to_scval(sac.clone()),
                                    // buyer: ScVal::Void,
                                    buying: env.to_scval(hash.clone()),
                                    amount: ScVal::I128(Int128Parts {
                                        hi: (amount >> 64) as i64,
                                        lo: amount as u64,
                                    }),
                                    active: ScVal::Bool(true),
                                };

                                env.put(&offer);
                            }
                        }
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
                        // TODO this is a delete method, just keep that in mind, likely only need it for offers
                        // Might be a little tricky as offers are stored as Vectors but in the db they're individual rows

                        // StorageKey::Color(miner, owner, color) => {}
                        // StorageKey::Glyph(hash) => {}
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

// TODO in the get methods consider processing the data a bit more to tighten up the amount of data we're returning

#[no_mangle]
pub extern "C" fn get_colors() {
    let env = EnvClient::empty();
    let request: GetColorsRequest = env.read_request_body();
    let owner = ScVal::from_xdr_base64(request.owner, Limits::none()).unwrap();

    let colors = env
        .read_filter()
        .column_equal_to_xdr("owner", &owner)
        .read::<ZephyrColor>()
        .unwrap();

    env.conclude(&colors);
}

#[derive(Serialize, Deserialize)]
pub struct GetGlyphsRequest {
    owner: String,
}

#[no_mangle]
pub extern "C" fn get_glyphs() {
    let env = EnvClient::empty();
    let request: GetGlyphsRequest = env.read_request_body();
    let owner = ScVal::from_xdr_base64(request.owner, Limits::none()).unwrap();

    let glyphs = env
        .read_filter()
        .column_equal_to_xdr("owner", &owner)
        .read::<ZephyrGlyph>()
        .unwrap();

    env.conclude(&glyphs);
}

#[derive(Serialize, Deserialize)]
pub struct GetOffersRequest {
    active: String,
}

#[no_mangle]
pub extern "C" fn get_offers() {
    let env = EnvClient::empty();
    // let request: GetOffersRequest = env.read_request_body();
    // let active = ScVal::from_xdr_base64(request.owner, Limits::none()).unwrap();

    // request.active;

    let offers = env
        // .read_filter()
        // .column_equal_to_xdr("active", &ScVal::Bool(true))
        .read::<ZephyrOffer>();
    // .unwrap();

    env.conclude(&offers);
}
