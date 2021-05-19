# Setting up the local testnet


# How to start a node terminal

The following examples rely on having a (local testnet)[https://docs.elrond.com/developers/setup-local-testnet/] up and running.
Note - this section assumes that you have `erdjs` installed globally.
By exporting `NODE_PATH`, the node terminal should have access to `erdjs`.
Open a terminal and enter the following:

```bash
export NODE_PATH=$HOME/.nvm/versions/node/$(node --version)/lib/node_modules
cd ./elrond-wasm-rs
node --experimental-repl-await
```

Then, import `erdjs`, set up the provider and some test wallets.

```javascript
let erdjs = await require('@elrondnetwork/erdjs');
let provider = erdjs.getLocalTestnetProvider();
let { alice, bob, carol, dan, eve } = await erdjs.loadAndSyncTestWallets(provider);
```

# Smart contract examples

- Adder [interaction](../../contracts/examples/adder/interaction/Readme.md)
- Crowdfunding

# Notes

## Choosing a provider

The available providers are:
- The local testnet
Note: You have to first 
```
let provider = erdjs.getLocalTestnetProvider();
```

- The Elrond Testnet
```
```

- The Elrond Devnet

- The Elrond Mainnet

## On working with balances

There are two ways of thinking about a balance:
- as a denominated unit (eg. 1.5 EGLD)
- by its raw decimal representation (eg. "1500000000000000000")

When working with examples, it makes most sense to deal with the denominated unit, both when providing and when reading such values.
However, when EGLD amounts are returned by smart contracts they are always returned as raw decimal values.

The examples below build a `Balance` of 1.5 EGLD.
```
erdjs.Egld(1.5).toCurrencyString();
erdjs.Egld("1.5").toCurrencyString();
```

On the other hand, if you need to build a balance from a raw non-denominated value, use `Egld.raw` instead. Note that the examples below are also 1.5 EGLD.
```
erdjs.Egld.raw(1_500_000_000_000_000_000).toCurrencyString();
erdjs.Egld.raw("1500000000000000000").toCurrencyString();
```

### Notes

- Javascript allows writing numerical values with the underscore separator.

- Javascript numbers are internally floating point values and, as such, have precision issues with large values (eg. `1_500_000_000_000_000_000 + 10 == 1_500_000_000_000_000_000` is `true`). This is the reason balances are stored as integer values in smart contracts (as `BigUint`) as well as in Javascript code (through `BigNumber`, which is used by `Balance` internally).

- The number of EGLD decimals is 18. By using `erdjs.Egld` and `erdjs.Egld.raw` correctly you shouldn't have to care about this.

- When dealing with fungible or semi-fungible ESDT tokens, the number of decimals varies depending on what the token's creator chose when he made it.
