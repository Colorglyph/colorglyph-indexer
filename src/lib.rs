use colorglyph::types::{Glyph, StorageKey};
use serde::{Deserialize, Serialize};
use zephyr_sdk::{
    prelude::*,
    soroban_sdk::xdr::{
        FeeBumpTransaction, FeeBumpTransactionEnvelope, FeeBumpTransactionInnerTx, Hash,
        HostFunction, InnerTransactionResult, InnerTransactionResultPair,
        InnerTransactionResultResult, InvokeContractArgs, InvokeHostFunctionOp,
        InvokeHostFunctionResult, LedgerEntry, LedgerEntryChange, LedgerEntryChanges,
        LedgerEntryData, LedgerKey, Operation, OperationBody, OperationMeta, OperationResult,
        OperationResultTr, ScAddress, ScVal, ToXdr, TransactionEnvelope, TransactionMeta,
        TransactionMetaV3, TransactionResult, TransactionResultMeta, TransactionResultPair,
        TransactionResultResult, TransactionV1Envelope, VecM,
    },
    DatabaseDerive, EnvClient,
};

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
    amount: i128,
    active: ScVal,
}

#[no_mangle]
pub extern "C" fn on_close() {
    let env = EnvClient::new();

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
                        // TODO this is a delete method, just keep that in mind, likely only need it for offers

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
