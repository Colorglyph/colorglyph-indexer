use colorglyph::types::{Glyph, Offer, StorageKey};
use serde::{Deserialize, Serialize};
use types::{
    Body, Change, Data, DataColor, DataGlyph, DataGlyphMinter, DataGlyphOwner, DataOffer,
    DataOfferSellerSelling, DataOfferSellingBuyingAmount, Offers,
};
use zephyr_sdk::{
    prelude::*,
    soroban_sdk::{
        xdr::{
            ContractDataEntry as SorobanContractDataEntry, FeeBumpTransaction,
            FeeBumpTransactionEnvelope, FeeBumpTransactionInnerTx, Hash, HostFunction,
            InnerTransactionResult, InnerTransactionResultPair, InnerTransactionResultResult,
            Int128Parts, InvokeContractArgs, InvokeHostFunctionOp, InvokeHostFunctionResult,
            LedgerEntry, LedgerEntryChange, LedgerEntryChanges, LedgerEntryData, LedgerKey,
            LedgerKeyContractData, Operation, OperationBody, OperationMeta, OperationResult,
            OperationResultTr, ScAddress, ScVal, TransactionEnvelope, TransactionMeta,
            TransactionMetaV3, TransactionResult, TransactionResultMeta, TransactionResultPair,
            TransactionResultResult, TransactionV1Envelope, VecM,
        },
        Address, BytesN, Vec as SorobanVec,
    },
    utils::address_to_alloc_string,
    AgnosticRequest, ContractDataEntry, EnvClient, Method,
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
    let mut cf_seq_num: i64 = 0;
    let mut cf_data: Vec<Data> = vec![];

    for (transaction_envelope, transaction_result_meta) in env.reader().envelopes_with_meta().iter()
    {
        process_transaction(
            &env,
            &mut cf_seq_num,
            &mut cf_data,
            transaction_envelope,
            transaction_result_meta,
        );
    }

    if !cf_data.is_empty() {
        let body = serde_json::to_string(&Body {
            seq_num: cf_seq_num,
            data: cf_data,
        })
        .unwrap();

        env.send_web_request(AgnosticRequest {
            body: Some(body),
            url: "https://colorglyph-worker.sdf-ecosystem.workers.dev/zephyr".into(),
            method: Method::Post,
            headers: vec![("Content-Type".into(), "application/json".into())],
        });
    }
}

fn process_transaction(
    env: &EnvClient,
    cf_seq_num: &mut i64,
    cf_data: &mut Vec<Data>,
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
                    cf_seq_num,
                    cf_data,
                    results,
                    transaction_envelope,
                    tx_apply_processing,
                ),
                _ => {}
            }
        }
        TransactionResultResult::TxSuccess(results) => process_operation_result(
            &env,
            cf_seq_num,
            cf_data,
            results,
            transaction_envelope,
            tx_apply_processing,
        ),
        _ => {}
    }
}

fn process_operation_result(
    env: &EnvClient,
    cf_seq_num: &mut i64,
    cf_data: &mut Vec<Data>,
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
                                        *cf_seq_num = tx.seq_num.0;

                                        for Operation { body, .. } in tx.operations.iter() {
                                            match body {
                                                OperationBody::InvokeHostFunction(op) => {
                                                    process_invoke_host_function_op(
                                                        env, cf_data, op, changes,
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
                                                *cf_seq_num = tx.seq_num.0;

                                                for Operation { body, .. } in tx.operations.iter() {
                                                    match body {
                                                        OperationBody::InvokeHostFunction(op) => {
                                                            process_invoke_host_function_op(
                                                                env, cf_data, op, changes,
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
    cf_data: &mut Vec<Data>,
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
                            process_ledger_entry_data(env, cf_data, data, None, Change::Create)
                        }
                        LedgerEntryChange::Updated(LedgerEntry { data, .. }) => {
                            process_ledger_entry_data(
                                env,
                                cf_data,
                                data,
                                Some(changes),
                                Change::Update,
                            )
                        }
                        LedgerEntryChange::Removed(key) => process_ledger_key(&env, cf_data, key),
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
    cf_data: &mut Vec<Data>,
    data: &LedgerEntryData,
    changes: Option<&LedgerEntryChanges>,
    change: Change,
) {
    match data {
        LedgerEntryData::ContractData(SorobanContractDataEntry { key, val, .. }) => {
            if let Ok(key) = env.try_from_scval::<StorageKey>(key) {
                // using try_ so we don't panic if the key isn't a StorageKey
                match &key {
                    StorageKey::Color(miner, owner, color) => {
                        let data = DataColor {
                            change,
                            miner: address_to_alloc_string(env, miner.clone()),
                            owner: address_to_alloc_string(env, owner.clone()),
                            color: *color,
                            amount: env.from_scval(val),
                        };

                        cf_data.push(Data::Color(data));
                    }
                    StorageKey::Glyph(hash) => {
                        let glyph: Glyph = env.from_scval(val);
                        let colors = env.to_scval(glyph.colors);

                        let data = DataGlyph {
                            change,
                            hash: hex::encode(hash.to_array()),
                            width: glyph.width,
                            length: glyph.length,
                            colors: colors.to_xdr_base64(Limits::none()).unwrap(),
                        };

                        cf_data.push(Data::Glyph(data));
                    }
                    StorageKey::GlyphOwner(hash) => {
                        let data = DataGlyphOwner {
                            change,
                            hash: hex::encode(hash.to_array()),
                            owner: address_to_alloc_string(env, env.from_scval::<Address>(val)),
                        };

                        cf_data.push(Data::GlyphOwner(data));
                    }
                    StorageKey::GlyphMinter(hash) => {
                        let data = DataGlyphMinter {
                            change,
                            hash: hex::encode(hash.to_array()),
                            minter: address_to_alloc_string(env, env.from_scval::<Address>(val)),
                        };

                        cf_data.push(Data::GlyphMinter(data));
                    }
                    StorageKey::GlyphOffer(hash) => {
                        let offers: SorobanVec<Offer> = env.from_scval(val);
                        let diff_offers =
                            get_diff_offers(&env, &key, changes, &Offers::Offers(offers.clone()));
                        if let Some(owner) = get_glyph_owner(env, hash) {
                            // Add or update
                            for offer in offers.iter() {
                                let buying: String;
                                let amount: Option<ScVal>;

                                match offer {
                                    // Selling a glyph for a glyph
                                    Offer::Glyph(buying_hash) => {
                                        buying = hex::encode(buying_hash.to_array()); // env.to_scval(buying_hash);
                                        amount = None;
                                    }
                                    // Selling a glyph for an asset
                                    Offer::Asset(sac, a) => {
                                        buying = address_to_alloc_string(env, sac);
                                        amount = Some(ScVal::I128(Int128Parts {
                                            hi: (a >> 64) as i64,
                                            lo: a as u64,
                                        }));
                                    }
                                    _ => {
                                        panic!("Invalid offer type")
                                    }
                                }

                                let data = DataOffer {
                                    change: change.clone(),
                                    seller: address_to_alloc_string(
                                        env,
                                        env.from_scval::<Address>(&owner),
                                    ),
                                    selling: hex::encode(hash.to_array()),
                                    buying,
                                    amount,
                                };

                                cf_data.push(Data::Offer(data));
                            }

                            // Remove if exists
                            if let Some(offers) = diff_offers {
                                if let Offers::Offers(offers) = offers {
                                    for offer in offers.iter() {
                                        let buying: String;
                                        let amount: Option<ScVal>;

                                        match offer {
                                            Offer::Glyph(buying_hash) => {
                                                buying = hex::encode(buying_hash.to_array());
                                                amount = None;
                                            }
                                            Offer::Asset(sac, a) => {
                                                buying = address_to_alloc_string(env, sac); // The asset the seller wants
                                                amount = Some(ScVal::I128(
                                                    // The amount of the buying asset the seller wants
                                                    Int128Parts {
                                                        hi: (a >> 64) as i64,
                                                        lo: a as u64,
                                                    },
                                                ));
                                            }
                                            _ => {
                                                panic!("Invalid offer type")
                                            }
                                        }

                                        let data = DataOffer {
                                            change: Change::Remove,
                                            seller: address_to_alloc_string(
                                                env,
                                                env.from_scval::<Address>(&owner),
                                            ),
                                            selling: hex::encode(hash.to_array()),
                                            buying,
                                            amount,
                                        };

                                        cf_data.push(Data::Offer(data));
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
                            let data = DataOffer {
                                change: change.clone(),
                                seller: address_to_alloc_string(env, owner),
                                selling: address_to_alloc_string(env, sac.clone()),
                                buying: hex::encode(hash.to_array()),
                                amount: Some(ScVal::I128(Int128Parts {
                                    hi: (amount >> 64) as i64,
                                    lo: *amount as u64,
                                })),
                            };

                            cf_data.push(Data::Offer(data));
                        }

                        // Remove if exists
                        if let Some(offers) = diff_offers {
                            if let Offers::Addresses(offers) = offers {
                                for owner in offers.iter() {
                                    let data = DataOffer {
                                        change: Change::Remove,
                                        seller: address_to_alloc_string(env, owner),
                                        selling: address_to_alloc_string(env, sac.clone()),
                                        buying: hex::encode(hash.to_array()),
                                        amount: Some(ScVal::I128(Int128Parts {
                                            hi: (amount >> 64) as i64,
                                            lo: *amount as u64,
                                        })),
                                    };

                                    cf_data.push(Data::Offer(data));
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

fn process_ledger_key(env: &EnvClient, cf_data: &mut Vec<Data>, key: &LedgerKey) {
    if let LedgerKey::ContractData(LedgerKeyContractData { key, .. }) = key {
        if let Ok(key) = env.try_from_scval::<StorageKey>(key) {
            match key {
                // StorageKey::Color(miner, owner, color) => {}
                // StorageKey::Glyph(hash) => {}
                // StorageKey::GlyphOwner(hash),
                // StorageKey::GlyphMinter(hash),
                StorageKey::GlyphOffer(hash) => {
                    if let Some(owner) = get_glyph_owner(env, &hash) {
                        let data = DataOfferSellerSelling {
                            change: Change::Remove,
                            seller: address_to_alloc_string(env, env.from_scval::<Address>(&owner)),
                            selling: hex::encode(hash.to_array()),
                        };

                        cf_data.push(Data::OfferSellerSelling(data));
                    }
                }
                StorageKey::AssetOffer(hash, sac, amount) => {
                    let data = DataOfferSellingBuyingAmount {
                        change: Change::Remove,
                        selling: hex::encode(hash.to_array()),
                        buying: address_to_alloc_string(env, sac),
                        amount: Some(ScVal::I128(Int128Parts {
                            hi: (amount >> 64) as i64,
                            lo: amount as u64,
                        })),
                    };

                    cf_data.push(Data::OfferSellingBuyingAmount(data));
                }
                _ => {}
            }
        }
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

fn get_glyph_owner(env: &EnvClient, hash: &BytesN<32>) -> Option<ScVal> {
    let owner = &env.read_contract_entry_by_scvalkey(
        CONTRACT_ADDRESS,
        env.to_scval(StorageKey::GlyphOwner(hash.clone())),
    );

    if owner.is_ok() && owner.clone().unwrap().is_some() {
        let ContractDataEntry { entry, .. } = owner.clone().unwrap().unwrap();
        let LedgerEntry { data, .. } = entry;

        if let LedgerEntryData::ContractData(SorobanContractDataEntry { val, .. }) = data {
            return Some(val);
        }
    }

    None
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
    let mut cf_seq_num: i64 = 0;
    let mut cf_data: Vec<Data> = vec![];

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

    process_transaction(
        &env,
        &mut cf_seq_num,
        &mut cf_data,
        &transaction_envelope,
        &transaction_result_meta,
    );

    if !cf_data.is_empty() {
        let body = serde_json::to_string(&Body {
            seq_num: cf_seq_num,
            data: cf_data,
        })
        .unwrap();

        env.send_web_request(AgnosticRequest {
            body: Some(body),
            url: "https://colorglyph-worker.sdf-ecosystem.workers.dev/zephyr".into(), // TODO make this an env var
            method: Method::Post,
            headers: vec![("Content-Type".into(), "application/json".into())],
        });
    }

    env.conclude("OK");
}
