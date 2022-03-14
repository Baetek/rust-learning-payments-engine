# rust-learning-payments-engine

This program simulates a bank. Given a transaction file it processes all the contained transactions to generate a state of the bank and all it's clients accounts.

- Supports async execution from multiple csv's at the same time for a considerable speed up, it is able to share state between the threads so different input data streams can reference the same customer accounts and transactions.
- Processes records from csv record by record , however it uses the csv crate's built in buffer reader to speed up IO
- Stores currency amounts internally as integers to avoid compunding floating point errors during long runs with millions of transactions.


# Getting Started

- You will need Rust installed with the toolchain

`cargo run -- transactions.csv` - Processes transactions in transactions.csv and outputs a final state of the bank's client accounts onto stdout

`cargo run -- transactions-provided-100k.csv transactions-provided-100k.csv transactions-provided-100k.csv` - Same as above but using tokio async to process multiple files at the same time

`cargo test` - Runs unit tests

`cargo doc --open` - Generates documentation for the project and opens in a webbrowser

`cargo clippy` - Outputs extra linting hints for the project

# Expected input format

Space separated list of one or more CSV file paths, with the files being of the following format: 

```
type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0
```

where 

`type` is the type of transaction, supported types are `deposit`, `withdrawal`, `dispute`, `resolve` or `chargeback`

`client` is a globally unique integer id of a client, 

`tx` is a globally unique integer id of the transaction, 

`amount` is a floating point amount of the transaction. This can be empty for transactions that aren't deposit or withdrawl - the empty value can be proceeded by a comma or not. It supports up to 4 places after the decimal point. 

# Expected Output format

For the example input above the program should output csv format text on stdout

```
client,available,held,total,locked
1,1.5,0,1.5,false
2,2,0,2,false
```

where

`client` is a globally unique id of a client, (integer)

`available` is the available balance in the clients account, (floating point)

`held` is the held / currently disputed balance in the clients account, (floating point)

`total` is the total funds in the clients account, (floating point)

`locked` is whether or not the account is locked due to a dispute ending with a chargeback (boolean)


# Project File Structure

```
.
├── Cargo.lock
├── Cargo.toml
├── README.md
├── src
│   ├── bin
│   │   └── main.rs             # The main file executed using `cargo run` (Reads csv paths from stdin and spins off tokio tasks of Bank::process_transactions_from_csv_path)
│   └── lib
│       ├── bank.rs             # Contains the Bank and Account structs and related functions
│       ├── lib.rs
│       ├── shared_types.rs     # Contains types used across the project to minimize duplication such as the type of ClientID of a transaction / bank 
│       └── transaction.rs      # Contains the Tx (Transction) struct and related functions + serde methods to serialize and deserialize th CSV
..
```

# Benchmarks

```
Single 1 million transaction CSV - 11 seconds
10 x 100k transaction CSV with async - 2 seconds
```

# To-Do
 
- Switch to zero copy serialization / deserialization if possible for this data set.
- CI/CD integration to run test on commits 
- More unit tests to cover more cases and integration tests of the main library entry points. 
- More documentation and fix to work better with cargo doc 
- Refactor to move getting client account of transaction::Tx:process to Bank::get_account
- Sanity checks on data - e.g held amount should probably never be negative
- doc examples (especially for how to use the library) and doc tests
- Logging
- More testing of error cases, errors seem to bubble up to where needed and appropriately handled but maybe missed something
