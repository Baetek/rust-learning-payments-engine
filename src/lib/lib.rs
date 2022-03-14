//!
//! This library is designed to be used with the [crate::main::main] runner,
//! however you can use it standalone.
//!
//! # Examples
//!
//! ```
//! use bank::bank::Bank;
//!
//! let bank = Bank::new();
//! Bank::process_transactions_from_csv_path("transactions.csv", bank);
//!
//! bank.write_accounts();
//! ```

pub mod bank;
pub mod shared_types;
pub mod transaction;
