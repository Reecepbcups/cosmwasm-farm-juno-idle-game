# CosmWasm Idle Game

This is a CosmWasm based idle game where you wait for your resources to grow over time. With your rewards you can buy upgrades to increase your rewards even more.

Uses Juno token
- Every block, they get +1 point (redeemable in the future for JUNO? or for its own TokenFactory idle coin?)
- Purcahse upgrades to get more tokens every block

Point Flow:
- user sends {start: {}} which saves addr->currentBlockHeight

- After X blocks, user sends {claim: {}} which saves their balance (currentBlockHeight - lastClaimBlockHeight) + currentBalance

- Multipliers run through and add more points over time.

- Prize Pool:
  - With enough points, get an NFT
  - Claim $JUNO itself? (some rate relative to number of accounts maybe?)  

---

Ideas:

Add a "mine" feature? where every block you can mine for a chance to get some points.
uses gas, but makes for better fees generation possibly?