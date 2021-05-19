# Lottery eGLD

First [set up a node terminal with erdjs and several test wallets](../../../../tutorial/src/interaction.md).

```javascript
let lottery = await erdjs.ContractWrapper.loadProject(provider, "contracts/examples/lottery-egld");

await lottery.sender(alice).gas(100_000_000).deploy();

// enough to cover any of the gas limits for the methods: start, status, buy_ticket, determine_winner
lottery.gas(30_000_000);

let ticketPrice = erdjs.Egld(1);
let total_tickets = 5;
let deadline = await erdjs.now() + erdjs.minutes(5);
let max_entries_per_user = 2;
let prize_distribution = [70, 20, 10];
let whitelist = null;
await lottery.start("my-lottery", ticketPrice, total_tickets, deadline, max_entries_per_user, prize_distribution, whitelist);

await lottery.status("my-lottery");

await lottery.value(ticketPrice).buy_ticket("my-lottery");
await lottery.sender(bob).value(ticketPrice).buy_ticket("my-lottery");
await lottery.sender(carol).value(ticketPrice).buy_ticket("my-lottery");

await lottery.status("my-lottery");

await lottery.sender(alice).determine_winner("my-lottery");


```
