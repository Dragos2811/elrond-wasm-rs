# Crowdfunding eGLD

First [set up a node terminal](../../../../tutorial/src/interaction.md).

```javascript
let crowdfunding = await erdjs.ContractWrapper.loadProject(provider, "contracts/examples/crowdfunding-egld");

let aFewMinutesFromNow = await erdjs.currentNonce(provider) + erdjs.minutesNonce(5);
await crowdfunding.sender(alice).gas(30_000_000).deploy(erdjs.Egld(10), aFewMinutesFromNow);

await crowdfunding.sender(bob).value(erdjs.Egld(5)).fund();
await crowdfunding.sender(carol).value(erdjs.Egld(7)).fund();

let currentFunds = erdjs.Egld.raw(await crowdfunding.currentFunds());
erdjs.printBalance(currentFunds);

erdjs.printBalance(erdjs.Egld.raw(await crowdfunding.get_target()));

await crowdfunding.get_owner();

let aliceBalanceBefore = await erdjs.getAccountBalance(alice, provider);
erdjs.printBalance(aliceBalanceBefore);

await crowdfunding.sender(alice).claim();

let aliceBalanceAfter = await erdjs.getAccountBalance(alice, provider);
erdjs.printBalance(aliceBalanceAfter);
erdjs.printBalance(aliceBalanceBefore - aliceBalanceAfter);

```
