# lighthouse
An implementation of a Bitcoin Lightning Network node. Just a disclaimer, I am new to Lightning and new to Rust. So expect progress on this project to be slow and riddled with bugs. I see this as a way for me to learn both and hopefully help for us learn together. Feel free to get in touch so that we can work on this together.

## My current understanding

Lightning is a P2P network built on top of the Bitcoin blockchain. A transaction is 'sent' to LN which 'locks' the bitcoin in the Lightning ecosystem. By 'sent' I mean a UTXO is sent to a multisig address generated during the opening of a channel between 2 peers. Bitcoin gets 'locked' (or rather freed) because it cannot be spent on-chain anymore until a closing transaction is initiated from the LN. 

Step 1 in the creation of the node will be to connect to bitcoin core (eg. `bitcoind`). 

Step 2 will be to connect to a peer node.

Step 3 will be to open a channel between the two nodes. This will entail creating a funding transaction.


## Some tools to help development

- Polar
- Lightning Development Kit (LDK)
