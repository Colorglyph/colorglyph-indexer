import { Horizon } from '@stellar/stellar-sdk';

const horizon = new Horizon.Server('https://horizon-testnet.stellar.org');

const account = 'GBGP5SD75TDB2ZL7JDJEFPSWDBEQRDJ4757ZXL57TOOQJSMWROT5JYKD';

const res = await horizon
    .transactions()
    .forAccount(account)
    .limit(200)
    .includeFailed(false)
    .call()
    .then(({ records }) => records)

for (const { hash, envelope_xdr, result_meta_xdr, result_xdr } of res) {
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
