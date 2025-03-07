# RustUp

## Overview
RustUp is a simple Rust-based blockchain system which can be thought of as a linked list. This structure ensures cryptographic guarantees across the entire chain.

## Features
- **Blockchain Implementation:** A linked list-like structure where each block references the previous block.
- **Proof-of-Work Mechanism:** A computationally intensive mining process ensuring block validity.
- **Work Queue System:** Efficiently manages mining tasks to add new blocks to the chain.

## Installation
### Setup Steps
Make sure you install the proper version of Rust 
### 1. Clone the repository
```sh
git clone https://github.com/TigerTim/RustUp.git
```
### 2. Build the project
```sh
cargo build --release
```
### 3. Run the application
```sh
cargo run
```

## Usage
Once the project is running, you can:

- Create a new blockchain and initialize the genesis block.
- Mine new blocks using proof-of-work.
- Verify the integrity of the chain by checking hash references.
- Efficiently process mining tasks using the work queue.

## Contributing
Feel free to fork and contribute improvements. Submit a pull request with detailed explanations of your changes.

## License
This project is licensed under the MIT License.
