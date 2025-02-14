# Automated Market Makers Program

The **AMMs Program** is a collection of Automated Market Makers (AMMs) implementations on the Solana blockchain built using the **Anchor Framework**. Currently, it supports the **Constant Product Market Maker (CPMM)** model, with future plans to implement **Concentrated Liquidity Market Maker (CLMM)**.

## Features

1. **Constant Product Market Maker**: A decentralized exchange model used in automated market makers (AMMs) like **Uniswap**, **Orca** and **Raydium**. It operates on the `x * y = k` formula, where `x` and `y` represent the reserves of two assets in a liquidity pool, and `k` is a constant that remains unchanged during trades.

## Requirements

- **Rust 1.79.0**. Install it by following [this guide](https://www.rust-lang.org/tools/install).
- **Solana CLI ≥2.0.0**: Install it by following [this guide](https://docs.solana.com/cli/install-solana-cli-tools).
- **Anchor CLI 0.30.1**: Install it by following [this guide](https://www.anchor-lang.com/docs/installation).
- **Node.js ≥23.1.0**: Install it by following [this guide](https://nodejs.org/en/download).

## Setup

1. **Clone the repository**:

   ```bash
   git clone https://github.com/Vondert/amms-program.git
   cd amms-program
   ```
   
2. **Generate keypair**:

   ```bash
   solana-keygen new --outfile owner.json
   ```
   
3. **Navigate to the project directory and install dependencies**:

   ```bash
   cd amms/
   npm install
   ```
   
4. **Synchronize keys and build the program**:

   ```bash
   anchor keys sync
   anchor build
   ```
   
5. **Generate client**:

   ```bash
   anchor run generate-clients
   ```
   
6. **Run tests**:

   ```bash
   anchor test
   ```
   
7. **Deploy**:

   ```bash
   anchor deploy
   ```
   
Before deploying, make sure your **Solana** configuration is correct and that you're using the intended network. If needed, update the keypair path in `Anchor.toml` to set the correct **Program Upgrade Authority**.

## License

This project is licensed under the MIT License - see the LICENSE file for details.