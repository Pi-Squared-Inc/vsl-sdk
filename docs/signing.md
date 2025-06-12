# Signing RPC messages

Messages that require signing are signed using the [EIP-191](https://eips.ethereum.org/EIPS/eip-191) standard:

First, they are encoded into bytes with [RLP encoding](https://ethereum.org/en/developers/docs/data-structures-and-encoding/rlp/).

This byte string is then prepended with a prefix which includes its length, according to the EIP-191 standard, and then hashed using Keccak256, and signed using Secp256k1 ECDSA.

For composing the signed RPC message, along with the other fields in the signed message, include the extra
fields:

- `hash`: hex encoding of the hashed encoded data.
- `r`: `r` of the signature, hex encoded and with `0x` prefix.
- `s`: `s` of the signature, hex encoded and with `0x` prefix.
- `v`: `v` of the signature, hex encoded and with `0x` prefix.

So for instance, if you want to call `vsl_pay` which takes a `Signed<PayMessage>`, create,
for example, the `PayMessage` like:

```json
{
  "from": "0x661403E07d8d910E45C21f3DD9303957a5D080c7",
  "to": "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
  "amount": "10",
  "nonce": "100"
}
```
Encode this using RLP and sign it using the private key corresponding to `"0x661403E07d8d910E45C21f3DD9303957a5D080c7"`. Say the signature is:

```
"hash": "0745906a6175337c4220c921c8e0bc8dfef5e25a58ab0dfa6edc7301e99edf45"
"r": "0xE53D9339D968314DF2EE1E7C0E661796EC25FA47F7AD92175DD318CC67B00957"
"s": "0x583A7DD9264D63ABB4097752FCC61E601D9700E2E3170D6A55321D8E82B97A0E"
"v": "0x01"
```

Then the full signed message would be:

```json
{
  "from": "0x661403E07d8d910E45C21f3DD9303957a5D080c7",
  "to": "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
  "amount": "10",
  "nonce": "100"
  "hash": "0745906a6175337c4220c921c8e0bc8dfef5e25a58ab0dfa6edc7301e99edf45",
  "r": "0xE53D9339D968314DF2EE1E7C0E661796EC25FA47F7AD92175DD318CC67B00957",
  "s": "0x583A7DD9264D63ABB4097752FCC61E601D9700E2E3170D6A55321D8E82B97A0E",
  "v": "0x01"
}
```

And the JSON RPC request might look like:

```
curl -X POST \
     -H 'Content-Type: application/json' \
     -d '{"jsonrpc":"2.0","id":"id","method":"vsl_pay","params":[{"from": "0x661403E07d8d910E45C21f3DD9303957a5D080c7", "to": "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "amount": "10", "nonce": "100", "hash": "0745906a6175337c4220c921c8e0bc8dfef5e25a58ab0dfa6edc7301e99edf45", "r": "0xE53D9339D968314DF2EE1E7C0E661796EC25FA47F7AD92175DD318CC67B00957", "s": "0x583A7DD9264D63ABB4097752FCC61E601D9700E2E3170D6A55321D8E82B97A0E", "v": "0x01"}]}' \
     http://localhost:44444
```
