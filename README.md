# rust-learning-payments-engine

This program simulates a bank. Given a transaction file it processes all the contained transactions to generate a state of the bank and all it's clients accounts.
It supports async execution from multiple csv's at the same time for a considerable speed up, it is able to share state between the threads so different files can reference the same customer accounts and transactions.


# Getting Started

`cargo run -- transactions.csv' - Processes transactions in transactions.csv and outputs a final state of the bank's client accounts onto stdout
`cargo run -- transactions-provided-100k.csv transactions-provided-100k.csv transactions-provided-100k.csv' - Same as above but using tokio async to process multiple files at the same time
`cargo test` - Runs unit tests
`cargo doc --open` - Generates documentation for the project and opens in a webbrowser

# Benchmarks

```
1 1 million transaction CSV - 11 seconds
10 100k transaction CSVs - 2 seconds
```

# To-Do
 
- Switch to zero copy serialization / deserialization if possible for this data set.
- CI/CD integration to run test on commits 
- More unit tests to cover more cases
- More documentation and fix to work better with cargo doc 
- Refactor to move getting client account of transaction::Tx:process to Bank::get_account
- Sanity checks on data - e.g held amount should probably never be negative
- doc examples (especially for how to use the library) and doc tests
- Logging
- More testing of error cases, errors seem to bubble up to where needed and appropriately handled but maybe missed something
