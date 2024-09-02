import { Horizon, Networks, Operation, Transaction } from '@stellar/stellar-sdk';

const horizon = new Horizon.Server('https://horizon-testnet.stellar.org');

const account = 'GBGP5SD75TDB2ZL7JDJEFPSWDBEQRDJ4757ZXL57TOOQJSMWROT5JYKD';

get_transactions();

async function get_transactions(cursor?: string) {
    const res = await horizon
        .transactions()
        .forAccount(account)
        .limit(200)
        .includeFailed(false)
        .order('asc')
        .cursor(cursor || '')
        .call()
        .then(({ records }) => records)

    for (const { hash, ledger_attr, envelope_xdr, result_meta_xdr, result_xdr, paging_token } of res) {
        const tx = new Transaction(envelope_xdr, Networks.TESTNET);

        for (const op of tx.operations) {
            const fn = (op as Operation.InvokeHostFunction)?.func?.invokeContract()?.functionName().toString();

            if (fn && fn.includes('offer'))
                await backfill(hash, ledger_attr, envelope_xdr, result_meta_xdr, result_xdr)
        }

        cursor = paging_token
    }

    if (res.length === 200)
        get_transactions(cursor)
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