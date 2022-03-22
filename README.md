# Lighthouse

An implementation of a Bitcoin Lightning Network node. Just a disclaimer, I am new to Lightning and new to Rust. So expect progress on this project to be slow and riddled with bugs. I see this as a way for me to learn both and hopefully help for us learn together. Feel free to get in touch so that we can work on this together.

## Some tools to help development

- Polar
- Lightning Development Kit (LDK)

## My current understanding

Lightning is a P2P network built on top of the Bitcoin blockchain. A transaction is 'sent' to LN which 'locks' the bitcoin in the Lightning ecosystem. By 'sent' I mean a UTXO is sent to a multisig address generated during the opening of a channel between 2 peers. Bitcoin gets 'locked' (or rather freed) because it cannot be spent on-chain anymore until a closing transaction is initiated from the LN. It gets freed to be moved around in the LN, which is off-chain. For the most part, LN is more private because only the channels involved in the payment know about the transaction, whereas in Bitcoin each transaction is broadcasted on the blockchain.

- Step 1 in the creation of the node will be to connect to bitcoin core (eg. `bitcoind`). 
- Step 2 will be to connect to a peer node.
    - This is estabilishing an internet connection with the node so that a channel can be opened
    - Generate node secret key and corresponding public key
    - Gossip node public key and network address to announce node as public (v2)
- Step 3 will be to open a channel between the 2 nodes.
    - Create a 2-of-2 multisig address
    - Create presigned transaction that spends the multisig output to prevent loss of funds
    - Channel open initiator transfers funds to this address (called the funding transaction)


## Theory

- bitcoind (Bitcoin client)
    - [https://en.bitcoin.it/wiki/Clients](https://en.bitcoin.it/wiki/Clients)
    - [Bitcoin Core: The Reference Implementation](https://github.com/bitcoinbook/bitcoinbook/blob/develop/ch03.asciidoc)
    - [https://docs.keep.network/tbtc/appendix/spv/#\_stateless_spv](https://docs.keep.network/tbtc/appendix/spv/#_stateless_spv)
- Lightning node keys
    - [https://bitcoin.stackexchange.com/questions/90948/what-does-a-bitcoin-lightning-private-key-look-like](https://bitcoin.stackexchange.com/questions/90948/what-does-a-bitcoin-lightning-private-key-look-like)
    - [https://en.wikipedia.org/wiki/Endianness](https://en.wikipedia.org/wiki/Endianness)
    - [https://en.bitcoin.it/wiki/Secp256k1](https://en.bitcoin.it/wiki/Secp256k1)

