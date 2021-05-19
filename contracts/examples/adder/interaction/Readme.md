# Adder

First [set up a node terminal with erdjs and several test wallets](../../../../tutorial/src/interaction.md).

```javascript
let adder = await erdjs.ContractWrapper.loadProject(provider, "contracts/examples/adder");

await adder.sender(alice).gas(20_000_000).deploy(42);

adder.gas(3_000_000);

await adder.getSum();
await adder.add(30);
await adder.getSum();
```
