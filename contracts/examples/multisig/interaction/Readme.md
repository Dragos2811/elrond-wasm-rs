# Multisig

First [set up a node terminal with erdjs and several test wallets](../../../../tutorial/src/interaction.md).

```javascript
let multisig = await erdjs.ContractWrapper.loadProject(provider, "contracts/examples/multisig");
await multisig.sender(alice).gas(150_000_000).deploy(3, [alice, bob, carol, dan]);

await multisig.gas(20_000_000).sender(alice).value(erdjs.Egld(10)).deposit();

// A proposal to send 3 egld from the multisig contract to eve's wallet
var id = await multisig.sender(alice).proposeSendEgld(eve, erdjs.Egld(3), null);

await multisig.sender(bob).sign(id);
await multisig.sender(carol).sign(id);

await multisig.getActionValidSignerCount(id);

await multisig.performAction(id);

let adder = await erdjs.ContractWrapper.loadProject(provider, "contracts/examples/adder");

let arguments = adder.argBuffers.deploy(42);

// A proposal to deploy the adder smart contract
var id = await multisig.sender(alice).gas(1_000_000_000).proposeSCDeploy(erdjs.Balance.Zero(), await adder.getCode(), false, false, false, arguments);

await multisig.sender(bob).gas(20_000_000).sign(id);
await multisig.sender(carol).sign(id);

let deployAddress = await multisig.performAction(id);

await adder.address(deployAddress).sender(alice).gas(20_000_000).getSum();

// A proposal on calling the "add" method on the previously deployed smart contract
let myAdd = adder.add(1000);
let myNftTransfer = NFT.transfer(MyNFT(42)).wrap(myAdd);
let myMultisig = multisig.proposeSCCall(adder, Egld(0)).wrap(myNftTransfer);
handleCall(myMultisig)

let { func, args } = adder.value(MyNFT(42)).callBuffers.add(1000);
var id = await multisig.sender(alice).gas(20_000_000).proposeSCCall(adder, erdjs.Balance.Zero(), func, ...args);

await multisig.sender(bob).sign(id);
await multisig.sender(carol).sign(id);

await multisig.gas(40_000_000).performAction(id);

await adder.sender(alice).getSum();

```
