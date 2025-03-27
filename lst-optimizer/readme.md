# LST Optimizer Client

## Overview

LST Optimizer Client is a Rust-based application that automatically optimizes staking yield by rebalancing across top Liquid Staking Tokens (LSTs) on Solana. The repository includes backtesting tools and a modular architecture to facilitate integration and performance evaluation.

## Repository Structure

```
LST Optimizer Client
│── backtesting/                 # Backtesting scripts (Python)
│── lst-optimizer/               # Rust workspace
│   │── integration-tests/       # Integration tests
│   │── packages/                # Rust packages
│   │   │── lst-optimizer-client/ # Main application
│   │   │── lst-optimizer-std/   # Standard package and interface
│   │   │── lst-optimizer-utils/ # Utility functions
```

## Installation

### Prerequisites

- Rust (latest stable version)
- Python (for backtesting)
- Solana Client
- Solana SDK

### Clone the Repository

```sh
git clone https://github.com/your-repo/lst-optimizer-client.git
cd lst-optimizer-client
```

### Build the Project

```sh
cd lst-optimizer/lst-optimizer-client
cargo build --release
```

## Usage

Run the optimizer with the following options:

```sh
./target/release/lst-optimizer-client --keypair <PATH_TO_KEYPAIR> --interval <REBALANCE_INTERVAL>
```

### Options:

- `--keypair <file>`: Path to the Solana keypair JSON file.
- `--interval <minutes>`: Rebalancing interval in minutes.

## Backtesting

To evaluate the performance of the optimizer using historical data, run the Python notebook script

## Contributing

We welcome contributions! Please open an issue or submit a pull request.

## License

This project is licensed under the MIT License.

## Contact

For inquiries, reach out via [@pawsengineer](https://github.com/pawsengineer).
