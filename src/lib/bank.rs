use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io;
use csv;
use csv::{Reader, ReaderBuilder};
use serde::Serialize;
use std::sync::{Arc, Mutex};

use crate::shared_types::{ClientId, TxId, Amount};
use crate::transaction::Tx;

#[derive(Debug)]
pub struct Bank {
    pub(crate) transactions: Arc<Mutex<HashMap<TxId, Tx>>>,
    pub(crate) accounts: Arc<Mutex<HashMap<ClientId, Account>>>
}

impl Bank {
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(Mutex::new(HashMap::new())),
            accounts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn new_for_tokio(bank: &Bank) -> Self {
        Self {
            transactions: bank.transactions.clone(),
            accounts: bank.accounts.clone()
        }
    }

    pub async fn process_transactions_from_csv_path(csv_path: &str, mut bank: Bank) {
        let mut file_reader = Bank::get_csv_reader(&csv_path)
            .expect(&format!("Failed to open csv {}", csv_path));
        for record in file_reader.deserialize() {
            let record: Tx = record.expect("Invalid raw data for transaction");
            record.process(&mut bank);
        }
    }

    fn get_csv_reader(csv_path: &str) -> Result<Reader<File>, Box<dyn Error>>  {
        Ok(ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .flexible(true)
            .from_path(csv_path)?
        )
    }

    /// Outputs the bank's accounts to stdout in csv format
    pub fn write_accounts(&self) -> Result<(), Box<dyn Error>> {
        let mut wtr = csv::Writer::from_writer(io::stdout());
        for account in self.accounts.lock().unwrap().values_mut() {
            account.calculate_total();
            wtr.serialize(account).unwrap();
        }
        wtr.flush()?;
        Ok(())
    }

}

/// The account state of a client
///
/// The client id is only used for writing to stdout
/// The total balance is only used for writing to stdout
/// So both can be optimized away, but this is more readable for now.
#[derive(Serialize, Debug)]
pub(crate) struct Account {
    pub(crate) client: ClientId,
    pub(crate) available: Amount,
    pub(crate) held: Amount,
    pub(crate) total: Amount,
    pub(crate) locked: bool,
}

impl Account {
    pub(crate) fn new(client: ClientId) -> Self {
        Self {
            client,
            available: Amount::new(),
            held: Amount::new(),
            total: Amount::new(),
            locked: false,
        }
    }

    /// Calculates the total balance of the account. Used for writing display output.
    pub(crate) fn calculate_total(&mut self) {
        self.total.value = self.available.value + self.held.value;
    }
}

#[cfg(test)]
mod tests {
    use crate::bank::{Account, Bank};
    use crate::shared_types::Amount;
    use crate::transaction::{Tx, TxType};

    #[test]
    fn test_new_for_tokio_bank_data_different_address() {
        // Make banks
        let bank = Bank::new();
        let tokio_bank = Bank::new_for_tokio(&bank);
        let tokio_bank_2 = Bank::new_for_tokio(&bank);

        assert_ne!(std::ptr::addr_of!(bank), std::ptr::addr_of!(tokio_bank));
        assert_ne!(std::ptr::addr_of!(bank.transactions), std::ptr::addr_of!(tokio_bank.transactions));
        assert_ne!(std::ptr::addr_of!(bank.accounts), std::ptr::addr_of!(tokio_bank.accounts));

        assert_ne!(std::ptr::addr_of!(tokio_bank), std::ptr::addr_of!(tokio_bank_2));
        assert_ne!(std::ptr::addr_of!(tokio_bank.transactions), std::ptr::addr_of!(tokio_bank_2.transactions));
        assert_ne!(std::ptr::addr_of!(tokio_bank.accounts), std::ptr::addr_of!(tokio_bank_2.accounts));
    }

    #[test]
    fn test_new_for_tokio_same_data() {
        // Make banks
        let bank = Bank::new();
        let tokio_bank = Bank::new_for_tokio(&bank);
        let tokio_bank_2 = Bank::new_for_tokio(&bank);
        // Make sample tx
        let tx = Tx {
            type_: TxType::Deposit,
            client: 0,
            tx: 0,
            amount: Amount { value: 500 },
            disputed: false
        };
        // Insert sample tx
        tokio_bank_2.transactions.lock().unwrap().insert(0, tx);

        // Get data
        let bank_amount = bank.transactions.lock().unwrap().get(&0).unwrap().amount.value.clone();
        let tokio_bank_amount = tokio_bank.transactions.lock().unwrap().get(&0).unwrap().amount.value.clone();
        let tokio_bank_2_amount = tokio_bank_2.transactions.lock().unwrap().get(&0).unwrap().amount.value.clone();

        // Compare data
        assert_eq!(tokio_bank_amount, tokio_bank_2_amount);
        assert_eq!(bank_amount, tokio_bank_2_amount);
    }

    #[test]
    fn test_calculate_total_avail_only() {
        let mut account = Account::new(1);
        account.available.value = 20;

        account.calculate_total();

        assert_eq!(account.total.value, account.available.value)
    }

    #[test]
    fn test_calculate_total_held_only() {
        let mut account = Account::new(1);
        account.held.value = 20;

        account.calculate_total();

        assert_eq!(account.total.value, account.held.value)
    }

    #[test]
    fn test_calculate_total_both() {
        let mut account = Account::new(1);
        account.available.value = 20;
        account.held.value = 10;

        account.calculate_total();

        assert_eq!(account.total.value, account.available.value + account.held.value)
    }
}