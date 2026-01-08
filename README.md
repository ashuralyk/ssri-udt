# `ssri-udt`

> [`[EN/CN] Script-Sourced Rich Information - 来源于 Script 的富信息`](https://talk.nervos.org/t/en-cn-script-sourced-rich-information-script/8256): General introduction to SSRI.
>
> [`pausable-udt`](https://github.com/Alive24/pausable-udt): The first fully SSRI compliant and production ready contract that exemplifies all use cases that SSRI protocool covers.
>
> [`pausable-udt` - Audit Report](https://github.com/Alive24/pausable-udt/blob/master/20241224-Pausable-UDT-Final-Audit-Report.pdf)
>
> [`ssri-server`](https://github.com/ckb-devrel/ssri-server): Server for calling SSRI methods.
>
> [`ckb_ssri_sdk`](https://github.com/Alive24/ckb_ssri_sdk) : Toolkit to help developers build SSRI-Compliant smart contracts on CKB by providing public Module Traits which would receive first party infrastructure support across the ecosystem, such as CKB Explorer, JoyID wallet, etc, and useful utility functions and macros to simplify the experience of building SSRI-Compliant contract.
>
> [`ckb_ssri_cli`](https://github.com/Alive24/ckb_ssri_cli): Command Line Interface for general users, moderators, and devs to interact with SSRI-Compliant Contracts deployed on CKB Network. Also exemplifies how to interact with SSRI compliant contract in Node.js.
>
> [`ssri-test`](https://github.com/Hanssen0/ssri-test): First prototype of SSRI-Compliant contract.

This is a streamlined SSRI-compliant smart contract that implements a UDT (User-Defined Token) with the SSRI protocol. The contract provides core UDT functionality including minting, transferring, and metadata management, while maintaining full SSRI compliance for seamless integration with CKB ecosystem infrastructure.

## Background

As xUDT is in effect deprecated in terms of providing extensibility for UDT, the need to extend UDT contracts are still to be satisfied; while the programmability of CKB allows great diversities in the way to implement, the inevitable need to index and interact in activities involving UDT assets requires a unified protocol to provide discoverability and predictability for both generic users and developers to explore new possibilities on the basis of trust on the behaviors of the infrastructures and other actors within the CKB and the greater connect ecology.

This project provides a streamlined, SSRI-compliant UDT implementation that focuses on core functionality while maintaining full compatibility with the SSRI protocol. By implementing the standard `UDT` trait, the contract ensures seamless integration with CKB ecosystem infrastructure such as CKB Explorer, JoyID wallet, and other dApps.

Based on the experience and insights from the `pausable-udt` project, as well as the latest updates of utilities and framework including `SSRI`, this contract is designed to be public, intuitive, predictable, and extensible, exemplifying a reliable and intuitive way of building smart contracts on CKB-VM.

## Quick Note on SSRI

SSRI stands for `Script Sourced Rich Information`; it is a protocol for strong bindings of relevant information and conventions to the Script itself on CKB. For more information, please read [[EN/CN] Script-Sourced Rich Information - 来源于 Script 的富信息](https://talk.nervos.org/t/en-cn-script-sourced-rich-information-script/8256).

Such bindings would take place in a progressive pattern:

1. On the level of validating transactions, by specifically using Rust Traits, we recognize the purpose (or more specifically, the `Intent` of running the script) (e.g., `minting UDT`, `transferring`) and build relevant validation logics within the scope of the corresponding method.
2. On the level of reading and organizing contract code, by selectively implementing methods of public module traits (e.g. `UDT`) in combinations, generic users and devs would be able to quickly understand and organize functionalities of contracts as well as the relevant adaptations / integrations in dApps, especially in use cases involving multiple distinct contracts (and very likely from different projects) within same transactions.
3. On the level of dApp integration and interactions with `ckb_ssri_cli`, SSRI-Compliant contracts provide predictable interfaces for information query (e.g. generic metadata source for explorer, CCC integration for public trait methods such as UDT), transaction generation/completion, and output data calculations which reduces engineering workload significantly by sharing code effectively.

## Interfaces

We implement the public module trait `UDT` defined in `ckb_ssri_sdk`. This is the basis of code organizing, public presenting, and generic integrations to dApps at the moment, and method reflections for SSRI-Calling in the future.

- For methods that we do not plan to implement, we will simply return `SSRIError::SSRIMethodsNotImplemented`.
- Methods that correspond to a behavior (e.g. mint, transfer) return an incomplete `Transaction` while you need to fill in the missing inputs and `CellDeps` with CCC. It can also be provided in the parameters in a way that allows chaining multiple actions.

```rust
pub trait UDT {
    type Error;
    fn balance() -> Result<u128, Self::Error>;
    fn transfer(
        tx: Option<Transaction>,
        to_lock_vec: Vec<Script>,
        to_amount_vec: Vec<u128>,
    ) -> Result<Transaction, Self::Error>;
    fn verify_transfer() -> Result<(), Self::Error>;
    fn name() -> Result<Bytes, Self::Error>;
    fn symbol() -> Result<Bytes, Self::Error>;
    fn decimals() -> Result<u8, Self::Error>;
    fn icon() -> Result<Bytes, Self::Error>;
    fn mint(
        tx: Option<Transaction>,
        to_lock_vec: Vec<Script>,
        to_amount_vec: Vec<u128>,
    ) -> Result<Transaction, Self::Error>;
    fn verify_mint() -> Result<(), Self::Error>;
}

pub enum UDTError {
    InsufficientBalance,
    NoMintPermission,
    NoBurnPermission,
}
```

Additionally, this contract implements a custom `SSRIUDT.create` method for contract creation and initialization.

## Script `<ssri-udt>`

- This project introduces one new `Script` as the asset type script.
- To be compatible with those UDT issuance that would take place before Script `<ssri-udt>` and scheduled to upgrade when it becomes available, we use the same rule for args definition as what sUDT/xUDT requires: if at least one input cell in the transaction uses owner lock specified by the `ssri-udt` as its cell lock, it enters governance operation and minting would be allowed.
- The contract uses Type ID to reference an external `SSRIMetadata` cell that stores token metadata (name, symbol, decimals, icon). This metadata is publicly accessible and can be queried via SSRI methods.

## Data Structures

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct SSRIMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub icon: String,
}
```

The `SSRIMetadata` structure is stored in a separate cell with Type ID, allowing the contract to reference token metadata while keeping the contract code itself minimal and focused on core UDT functionality.

## User and Admin Experience

### Recipes

### Transfer

```yaml
Inputs:
    ssri-udt-cell:
        Type:
            code: <ssri-udt>
            args: <Type ID args (SSRI metadata cell type ID)>
        Lock: <User Lock A>
        Data: <amount>
Dependencies:
    ssri-metadata-cell:
        Type:
            code: <Type ID Type>
            args: <Type ID>
        Data: SSRIMetadata
Outputs:
    ssri-udt-cell:
        Type:
            code: <ssri-udt>
            args: <Type ID args>
        Lock: <User Lock B>
        Data: <transferred-amount>
    ssri-udt-cell:
        Type:
            code: <ssri-udt>
            args: <Type ID args>
        Lock: <User Lock A>
        Data: <change-amount>
```

- Transfer transactions validate that input amounts are sufficient for the output amounts. The contract uses the fallback function to automatically detect whether a transaction is a transfer (input amount equals output amount) or a mint (input amount is less than output amount).

### Mint

```yaml
Inputs:
  owner-lock-cell:
    Lock: <Owner Lock>
Dependencies:
  ssri-metadata-cell:
    Type:
      code: <Type ID Type>
      args: <Type ID>
    Data: SSRIMetadata
Outputs:
  ssri-udt-cell:
    Type:
      code: <ssri-udt>
      args: <Type ID args>
    Lock: <Recipient Lock>
    Data: <mint-amount>
```

- Minting is only allowed when at least one input cell uses the owner lock specified in the SSRI metadata cell. The contract verifies this during `verify_mint()`.

## Interacting with `ckb-ssri-cli` (or anything with TypeScript)

- See examples in <https://github.com/Alive24/ckb_ssri_cli>. It would be transferrable to any TypeScript project.
- You would need to run an <https://github.com/Alive24/ssri-server> locally at the moment.

```tsx
// Mint
const payload = {
  id: 2,
  jsonrpc: "2.0",
  method: "run_script_level_script",
  params: [
    codeCellDep.outPoint.txHash,
    Number(codeCellDep.outPoint.index),
    [
      mintPathHex,
      `0x${heldTxEncodedHex}`,
      `0x${toLockArrayEncodedHex}`,
      `0x${toAmountArrayEncodedHex}`,
    ],
    // NOTE: field names are wrong when using udtTypeScript.toBytes()
    {
      code_hash: udtTypeScript.codeHash,
      hash_type: udtTypeScript.hashType,
      args: udtTypeScript.args,
    },
  ],
};

// Transfer
const payload = {
  id: 2,
  jsonrpc: "2.0",
  method: "run_script_level_script",
  params: [
    codeCellDep.outPoint.txHash,
    Number(codeCellDep.outPoint.index),
    // args.index,
    [
      transferPathHex,
      `0x${heldTxEncodedHex}`,
      `0x${toLockArrayEncodedHex}`,
      `0x${toAmountArrayEncodedHex}`,
    ],
    // NOTE: field names are wrong when using udtTypeScript.toBytes()
    {
      code_hash: udtTypeScript.codeHash,
      hash_type: udtTypeScript.hashType,
      args: udtTypeScript.args,
    },
  ],
};

// Icon
const payload = {
  id: 2,
  jsonrpc: "2.0",
  method: "run_script_level_code",
  params: [
    matchingCellDep.outPoint.txHash,
    Number(matchingCellDep.outPoint.index),
    [iconPathHex],
  ],
};

// Decimals
const payload = {
  id: 2,
  jsonrpc: "2.0",
  method: "run_script_level_code",
  params: [
    matchingCellDep.outPoint.txHash,
    Number(matchingCellDep.outPoint.index),
    [decimalPathHex],
  ],
};

// Name
const payload = {
  id: 2,
  jsonrpc: "2.0",
  method: "run_script_level_code",
  params: [
    matchingCellDep.outPoint.txHash,
    Number(matchingCellDep.outPoint.index),
    [namePathHex],
  ],
};

// Symbol
const payload = {
  id: 2,
  jsonrpc: "2.0",
  method: "run_script_level_code",
  params: [
    matchingCellDep.outPoint.txHash,
    Number(matchingCellDep.outPoint.index),
    [symbolPathHex],
  ],
};

// Create (Contract Initialization)
const payload = {
  id: 2,
  jsonrpc: "2.0",
  method: "run_script_level_code",
  params: [
    codeCellDep.outPoint.txHash,
    Number(codeCellDep.outPoint.index),
    [
      createPathHex,
      `0x${heldTxEncodedHex}`,
      `0x${ownerLockEncodedHex}`,
      `0x${ssriMetadataEncodedHex}`,
    ],
  ],
};

// Get transaction and send it
// Send POST request
try {
  const response = await axios.post(process.env.SSRI_SERVER_URL!, payload, {
    headers: { "Content-Type": "application/json" },
  });
  const mintTx = blockchain.Transaction.unpack(response.data.result);
  const cccMintTx = ccc.Transaction.from(mintTx);
  await cccMintTx.completeInputsByCapacity(signer);
  await cccMintTx.completeFeeBy(signer);
  const mintTxHash = await signer.sendTransaction(cccMintTx);
  this.log(
    `Mint ${args.toAmount} ${args.symbol} to ${args.toAddress}. Tx hash: ${mintTxHash}`
  );
} catch (error) {
  console.error("Request failed", error);
}
```

## Testing

- Due to the limitations of `ckb_testtools`, it is recommended to test the same SSRI-Compliant Contract on two level:
  - On-chain Verification: Test with `ckb_testtools`
  - Off-chain Query/Integration, Transaction Generations/Completions: Test with `ckb_ssri_cli` against the latest deployment.

## Deployment and Migration

- Deploy and upgrade with [ckb-cinnabar](https://github.com/ashuralyk/ckb-cinnabar?tab=readme-ov-file#deployment-module) for easier deployment and migration with Type ID.

```bash
ckb-cinnabar deploy --contract-name ssri-udt --tag transaction.v241112 --payer-address ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqtxe0gs9yvwrsc40znvdc6sg4fehd2mttsngg4t4 --type-id

ckb-cinnabar migrate --contract-name ssri-udt --from-tag v0.2.8 --to-tag v0.2.9
```

## Roadmaps and Goal

- [x] Equivalent functionalities of sUDT in pure Rust;
- [x] Validations of UDT transactions in fallback function (transfer/mint detection);
- [x] First integration with dApps for the purpose of demonstration with CCC;
- [x] Fully supported SSRI protocol;
- [x] Streamlined implementation focusing on core UDT functionality
