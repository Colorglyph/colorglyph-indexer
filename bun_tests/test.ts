import { Keypair } from "@stellar/stellar-sdk"

const keypair = Keypair.fromPublicKey('GBGP5SD75TDB2ZL7JDJEFPSWDBEQRDJ4757ZXL57TOOQJSMWROT5JYKD')

console.log(
    keypair.rawPublicKey()
);