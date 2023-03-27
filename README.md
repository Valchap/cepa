# cepa
A toy implementation of a tor like anonymizing network

# Cepa packet
```text
       RSA 2048                AES 256
 ┌─────────┴─────────┐ ┌──────────┴──────────┐
┌─────────┬───────────┬─────────────┬─────────┐
│         │           │             │         │
│ AES Key │ AES Nonce │ Destination │ Payload │
│         │           │             │         │
└─────────┴───────────┴─────────────┴─────────┘

A cepa packet can be encapsulated as a payload so that it is sent further by the next cepa node
When the destination address is the address of the node, that means it is meant to be the recipient of the packet
```
