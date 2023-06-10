# cepa
A toy implementation of a tor like anonymizing network

# Cepa packet
```text
               RSA 2048               AES 256
 ┌────────────────┴────────────────┐ ┌───┴───┐
┌─────────┬───────────┬─────────────┬─────────┐
│         │           │             │         │
│ AES Key │ AES Nonce │ Destination │ Payload │
│         │           │             │         │
└─────────┴───────────┴─────────────┴─────────┘
```
A cepa packet can be encapsulated as a payload so that it is sent further by the next cepa node.

When the destination address is 0.0.0.0, it means that the cepa packet arrived at it's final destination and doesn't need to be sent further

