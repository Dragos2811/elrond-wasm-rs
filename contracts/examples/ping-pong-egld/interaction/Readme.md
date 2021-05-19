# Ping-pong

First [set up a node terminal with erdjs and several test wallets](../../../../tutorial/src/interaction.md).

```javascript
let pingPong = await erdjs.ContractWrapper.loadProject(provider, "contracts/examples/ping-pong-egld");

await pingPong.sender(alice).gas(150_000_000).deploy(erdjs.Egld(0.5), 2 * 60, null, erdjs.Egld(1.5));

await pingPong.gas(20_000_000).sender(alice).value(erdjs.Egld(0.5)).ping("note 1");

await pingPong.sender(bob).value(erdjs.Egld(0.5).ping(null);
await pingPong.sender(carol).value(erdjs.Egld(0.5).ping(null);

// this fails because of the balance limit of 1.5 egld
await pingPong.sender(dan).value(erdjs.Egld(0.5).ping(null);

await pingPong.pongAll();

```
