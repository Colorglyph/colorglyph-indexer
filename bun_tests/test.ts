import { Keypair } from "@stellar/stellar-sdk"

const keypair = Keypair.fromPublicKey('GBGP5SD75TDB2ZL7JDJEFPSWDBEQRDJ4757ZXL57TOOQJSMWROT5JYKD')

console.log(
    keypair.rawPublicKey()
);

console.log(
    Buffer.from('9eb925d1fe9970fc0e2e93ad1b4c8c1e92136600f9aac84b89dda44814d188cb', 'hex')
);
