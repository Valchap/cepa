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

A cepa packet can be encapsulated as a payload so that it is sent further by the next cepa node
When the destination address is localhost (127.0.0.1), this mean the cepa packet arrived at it's final destination and can be used by the node (after decryption)
```
