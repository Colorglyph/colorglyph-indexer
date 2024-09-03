import { Horizon, Networks, Operation, Transaction } from '@stellar/stellar-sdk';

const horizon = new Horizon.Server('https://horizon-testnet.stellar.org');

const accounts = [
    'GBGP5SD75TDB2ZL7JDJEFPSWDBEQRDJ4757ZXL57TOOQJSMWROT5JYKD',
    'GAID7BB5TASKY4JBDBQX2IVD33CUYXUPDS2O5NAVAP277PLMHFE6AO3Y',
];

const transactions: Horizon.ServerApi.TransactionRecord[] = []

for (const account of accounts) {
    await get_transactions(account);
}

transactions.sort((a, b) => a.ledger_attr - b.ledger_attr)

for (const { hash, ledger_attr, envelope_xdr, result_meta_xdr, result_xdr } of transactions) {
    const tx = new Transaction(envelope_xdr, Networks.TESTNET);

    for (const op of tx.operations) {
        const fn = (op as Operation.InvokeHostFunction)?.func?.invokeContract()?.functionName().toString();

        if (
            fn 
            && fn.includes('colors')
        ) await backfill(hash, ledger_attr, envelope_xdr, result_meta_xdr, result_xdr)
    }
}

async function get_transactions(account: string, cursor?: string) {
    const { records } = await horizon
        .transactions()
        .forAccount(account)
        .limit(200)
        .includeFailed(false)
        .order('asc')
        .cursor(cursor || '')
        .call()

    transactions.push(...records)
    cursor = records[records.length - 1].paging_token

    if (records.length === 200)
        return get_transactions(account, cursor)
}

async function backfill(hash: string, ledger_attr: number, envelope_xdr: string, result_meta_xdr: string, result_xdr: string) {
    await fetch('https://api.mercurydata.app/zephyr/execute', {
        method: 'POST',
        headers: {
            Authorization: `Bearer ${Bun.env.MERCURY_JWT}`,
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            project_name: 'zephyr-colorglyph-ingestion',
            mode: {
                Function: {
                    fname: 'backfill',
                    arguments: JSON.stringify({
                        envelope_xdr,
                        result_meta_xdr,
                        result_xdr,
                    })
                }
            }
        })
    })
        .then(async (res) => {
            if (res.ok) {
                console.log(hash, ledger_attr, await res.text());
            } else {
                throw new Error(res.statusText)
            }
        })
}