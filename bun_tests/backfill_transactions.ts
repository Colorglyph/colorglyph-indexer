import { Horizon, Networks, Operation, Transaction } from '@stellar/stellar-sdk';

const horizon = new Horizon.Server('https://horizon-testnet.stellar.org');

const hashes = [
    '058d6f0ca988e616d0c5b61dbf9c00b6de4a1d7f88a79d11729cf78d751551a5',
    'e0bde5cedb00c9556111ef482595bc43e0fe52a43e5b36795fd1012472d4cf88',
    '162e613ffecd471ca283ade4cb3917f5d7c17b1abd46fb78b777f0c41e96fcef',
    'fa770e77074690d592d1ea162d26d8789d69ce5139410f5fdc3c9226da945ebd',
];

const transactions: Horizon.ServerApi.TransactionRecord[] = []

for (const hash of hashes) {
    await get_transaction(hash);
}

transactions.sort((a, b) => a.ledger_attr - b.ledger_attr)

for (const { hash, ledger_attr, envelope_xdr, result_meta_xdr, result_xdr } of transactions) {
    const tx = new Transaction(envelope_xdr, Networks.TESTNET);

    for (const op of tx.operations) {
        const fn = (op as Operation.InvokeHostFunction)?.func?.invokeContract()?.functionName().toString();

        if (
            fn 
            // && fn.includes('glyph_mint')
        ) await backfill(hash, ledger_attr, envelope_xdr, result_meta_xdr, result_xdr)
    }
}

async function get_transaction(hash: string) {
    const transaction = await horizon
        .transactions()
        .transaction(hash)
        .call()

    transactions.push(transaction)
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