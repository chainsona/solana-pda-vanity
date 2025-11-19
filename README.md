# PDA Vanity Token Creator

A high-performance Solana program and utility suite for generating and creating SPL Tokens with vanity Program Derived Addresses (PDAs).

## Overview

This project allows you to create SPL Token Mints where the address ends with a specific suffix (e.g., `pump`). It consists of two main components:

1.  **Off-chain Search Tool**: A multi-threaded Rust utility that brute-forces seeds to find a PDA matching your desired suffix.
2.  **On-chain Program**: An Anchor program that initializes the token mint and enforces the vanity suffix requirement on-chain.

## Features

-   **High Performance Search**: Utilizes all available CPU cores to search millions of seeds per second.
-   **On-Chain Enforcement**: The smart contract verifies that the generated address actually matches the required pattern before allowing creation.
-   **Anchor Framework**: Built with modern Solana development standards.

## Prerequisites

-   Rust & Cargo
-   Solana CLI
-   Anchor CLI
-   Node.js & Yarn/PNPM

## Installation

1.  Clone the repository:
    ```bash
    git clone <repo-url>
    cd pda-vanity
    ```

2.  Install dependencies:
    ```bash
    pnpm install
    ```

## Usage

### 1. Find a Vanity Seed

Use the included Rust binary to search for a seed that generates a PDA with your desired suffix.

```bash
cd programs/pda-vanity
cargo run --release --bin search
```

*Note: You can modify the target suffix in `programs/pda-vanity/src/search.rs`.*

Output example:
```text
Searching for seed for suffix 'pump' with 14 threads...
Found seed: 5270498306774619999
PDA: HZTPCxeTBLEr5FfUkjzLixXduWCzhgzjhvoNrKVspump
Bump: 255
Time: 2.20s
```

### 2. Create the Token

Once you have the seed, you can call the program to create the token.

**Testing:**
Update the seed in `tests/pda-vanity.ts` and run:

```bash
anchor test
```

**Production:**
You can integrate the client-side logic into your dApp using the IDL.

```typescript
await program.methods
  .createVanityToken(new BN("5270498306774619999"), 6) // seed, decimals
  .accounts({
    payer: wallet.publicKey,
  })
  .rpc();
```

## Technical Details

### PDA Derivation
The program uses a standard PDA derivation scheme:
```rust
seeds = [vanity_seed.to_le_bytes().as_ref()]
```

### Optimization
The search tool is optimized by:
-   Using multi-threading (Rayon-style manual thread spawning).
-   Checking only the canonical bump (255) to avoid the expensive `find_program_address` loop.
-   Using `Pubkey::create_program_address` directly.

## Contributing

Contributions are welcome! Please check out the [CONTRIBUTING.md](CONTRIBUTING.md) file for guidelines on how to proceed.

We also have a [Code of Conduct](CODE_OF_CONDUCT.md) that we expect all contributors to adhere to.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
