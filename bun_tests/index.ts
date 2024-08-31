import { Horizon, Networks, Operation, Transaction } from '@stellar/stellar-sdk';

const horizon = new Horizon.Server('https://horizon-testnet.stellar.org');

const account = 'GBGP5SD75TDB2ZL7JDJEFPSWDBEQRDJ4757ZXL57TOOQJSMWROT5JYKD';

const res = await horizon
    .transactions()
    .forAccount(account)
    .limit(200)
    .includeFailed(false)
    .order('asc')
    .call()
    .then(({ records }) => records)

for (const { hash, envelope_xdr, result_meta_xdr, result_xdr } of res) {
    const tx = new Transaction(envelope_xdr, Networks.TESTNET);

    for (const op of tx.operations) {
        const fn = (op as Operation.InvokeHostFunction)?.func?.invokeContract()?.functionName().toString();

        if (fn && fn.includes('offer'))
            await backfill(hash, envelope_xdr, result_meta_xdr, result_xdr)
    }
}

async function backfill(hash: string, envelope_xdr: string, result_meta_xdr: string, result_xdr: string) {
    await fetch('https://api.mercurydata.app/zephyr/execute', {
        method: 'POST',
        headers: {
            Authorization: `Bearer ${Bun.env.MERCURY_JWT}`,
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
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
                console.log(hash, await res.text());
            } else {
                throw new Error(res.statusText)
            }
        })
}