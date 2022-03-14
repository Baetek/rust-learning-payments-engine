use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::shared_types::{ClientId, TxId, Amount, AmountValue, RawAmountValue};
use crate::bank::{Account, Bank};

/// A Transaction is represented here.
/// type, client, tx, and amount are to be supplied from a payment processor.
/// disputed is an internal variable to indicate whether the transaction has been disputed.
#[derive(Deserialize, Debug)]
pub(crate) struct Tx {
    #[serde(rename = "type")]
    pub(crate) type_: TxType,
    pub(crate) client: ClientId,
    pub(crate) tx: TxId,
    pub(crate) amount: Amount,
    #[serde(skip)]
    pub(crate) disputed: bool,
}

impl Tx {
    /// Processes this transaction
    /// Updates the bank transaction sheet and the client's account
    ///
    /// If the client's account is locked, the transaction is not processed.
    ///
    /// Transactions of type Dispute, Resolve and Chargeback are
    /// meta-transactions that are not stored on the transaction sheet directly
    /// but instead affect the state of the client's account.
    ///
    /// # Arguments
    ///
    /// `bank` - The bank to process this transaction with
    pub(crate) fn process(self, bank: &mut Bank) {
        let mut accounts = bank.accounts.lock().unwrap();
        let account = match accounts.get_mut(&self.client) {
            Some(acc) => {
                if acc.locked { return; }
                acc
            },
            None => {
                let account = Account::new(self.client);
                accounts.insert(self.client, account);
                accounts.get_mut(&self.client).unwrap()
            }
        };
        match self.type_ {
            TxType::Deposit => {
                account.available.value += self.amount.value;
            },
            TxType::Withdrawal => {
                if account.available.value >= self.amount.value {
                    account.available.value -= self.amount.value;
                }
            },
            TxType::Dispute => {
                match bank.transactions.lock().unwrap().get_mut(&self.tx) {
                    Some(disputed_tx) => {
                        account.available.value -= disputed_tx.amount.value;
                        account.held.value += disputed_tx.amount.value;
                        disputed_tx.disputed = true;
                    },
                    None => ()
                }
            },
            TxType::Resolve => {
                match bank.transactions.lock().unwrap().get_mut(&self.tx) {
                    Some(disputed_tx) => {
                        if disputed_tx.disputed {
                            account.available.value += disputed_tx.amount.value;
                            account.held.value -= disputed_tx.amount.value;
                            disputed_tx.disputed = false;
                        }
                    },
                    None => ()
                }
            },
            TxType::Chargeback => {
                match bank.transactions.lock().unwrap().get(&self.tx) {
                    Some(disputed_tx) => {
                        if disputed_tx.disputed {
                            account.locked = true;
                            account.held.value -= disputed_tx.amount.value;
                        }
                    },
                    None => ()
                }
            },
        }
        if matches!(self.type_, TxType::Deposit | TxType::Withdrawal) {
            bank.transactions.lock().unwrap().insert(self.tx, self);
        }
    }
}

/// The type of transaction
#[derive(Debug)]
pub enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback
}

/// Used by serde to parse the transaction type given by a payment processor into a TxType
impl<'de> Deserialize<'de> for TxType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
        {
            let s = String::deserialize(deserializer)?;
            Ok(match s.as_str() {
                "deposit" => TxType::Deposit,
                "withdrawal" => TxType::Withdrawal,
                "dispute" => TxType::Dispute,
                "resolve" => TxType::Resolve,
                "chargeback" => TxType::Chargeback,
                _ => panic!("Unrecognized transaction type: {:?}", s.as_str())
            })
        }
}

/// Converts the amount of a transaction into an integer
/// While the program is running on a lot of tx's, errors due to floating point representation
/// are possible, so internally we use integers to represent the amount.
impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
        {
            let value = match RawAmountValue::deserialize(deserializer) {
                Ok(amount) => {
                    (amount * 10000.0).round() as AmountValue
                },
                _ => 0
            };
            Ok(Amount {
                value
            })
        }
}

/// When serializing the amount of a transaction or any amounts on a client account
/// we divide by 10000 to turn it back into a float to get the desired output
impl <'de> Serialize for Amount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
        {
            let value = self.value as RawAmountValue / 10000.0;
            value.serialize(serializer)
        }
}

#[cfg(test)]
mod tests {
    use crate::bank::Bank;
    use crate::shared_types::Amount;
    use crate::transaction::{Tx, TxType};

    #[test]
    fn test_amount_stored_as_integer() {
        let mut rdr = csv::Reader::from_reader("deposit, 2, 2, 5.1234".as_bytes());
        for record in rdr.deserialize() {
            let tx: Tx = record.unwrap();
            assert_eq!(tx.amount.value, 51234);

            let serialized = format!("{:?}", tx);
            assert_eq!(serialized.contains("5.1234"), true)
        }
    }

    #[test]
    fn test_process_tx_deposit() {
        let mut bank = Bank::new();

        Tx {
            type_: TxType::Deposit,
            client: 1,
            tx: 1,
            amount: Amount { value: 5},
            disputed: false
        }.process(&mut bank);

        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().client, &1);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().available.value, &5);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().held.value, &0);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().locked, &false);
        assert_eq!(bank.transactions.lock().unwrap().len() as i32, 1);

    }

    #[test]
    fn test_process_tx_deposit_locked() {
        let mut bank = Bank::new();

        Tx {
            type_: TxType::Deposit,
            client: 1,
            tx: 1,
            amount: Amount { value: 5},
            disputed: false
        }.process(&mut bank);
        Tx {
            type_: TxType::Dispute,
            client: 1,
            tx: 1,
            amount: Amount { value: 0},
            disputed: false
        }.process(&mut bank);
        Tx {
            type_: TxType::Chargeback,
            client: 1,
            tx: 1,
            amount: Amount { value: 0},
            disputed: false
        }.process(&mut bank);
        Tx {
            type_: TxType::Deposit,
            client: 1,
            tx: 2,
            amount: Amount { value: 1},
            disputed: false
        }.process(&mut bank);

        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().client, &1);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().available.value, &0);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().held.value, &0);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().locked, &true);
        assert_eq!(&bank.transactions.lock().unwrap().get(&1).unwrap().disputed, &true);
        assert_eq!(bank.transactions.lock().unwrap().len() as i32, 1);
    }

    #[test]
    fn test_process_tx_withdrawal() {
        let mut bank = Bank::new();

        Tx {
            type_: TxType::Deposit,
            client: 1,
            tx: 1,
            amount: Amount { value: 5},
            disputed: false
        }.process(&mut bank);
        Tx {
            type_: TxType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Amount { value: 5},
            disputed: false
        }.process(&mut bank);

        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().client, &1);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().available.value, &0);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().held.value, &0);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().locked, &false);
        assert_eq!(bank.transactions.lock().unwrap().len() as i32, 2);
    }

    #[test]
    fn test_process_tx_withdrawal_insufficient_funds() {
        let mut bank = Bank::new();

        Tx {
            type_: TxType::Deposit,
            client: 1,
            tx: 1,
            amount: Amount { value: 3},
            disputed: false
        }.process(&mut bank);
        Tx {
            type_: TxType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Amount { value: 5},
            disputed: false
        }.process(&mut bank);

        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().client, &1);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().available.value, &3);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().held.value, &0);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().locked, &false);
        assert_eq!(bank.transactions.lock().unwrap().len() as i32, 2);
    }

    #[test]
    fn test_process_tx_withdrawal_mid_dispute() {
        let mut bank = Bank::new();
        Tx {
            type_: TxType::Deposit,
            client: 1,
            tx: 1,
            amount: Amount { value: 3},
            disputed: false
        }.process(&mut bank);
        Tx {
            type_: TxType::Dispute,
            client: 1,
            tx: 1,
            amount: Amount { value: 0},
            disputed: false
        }.process(&mut bank);
        Tx {
            type_: TxType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Amount { value: 3},
            disputed: false
        }.process(&mut bank);

        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().client, &1);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().available.value, &0);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().held.value, &3);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().locked, &false);
        assert_eq!(&bank.transactions.lock().unwrap().get(&1).unwrap().disputed, &true);
        assert_eq!(bank.transactions.lock().unwrap().len() as i32, 2);
    }

    #[test]
    fn test_process_tx_dispute_resolved() {
        let mut bank = Bank::new();
        Tx {
            type_: TxType::Deposit,
            client: 1,
            tx: 1,
            amount: Amount { value: 3},
            disputed: false
        }.process(&mut bank);
        Tx {
            type_: TxType::Dispute,
            client: 1,
            tx: 1,
            amount: Amount { value: 0},
            disputed: false
        }.process(&mut bank);
        Tx {
            type_: TxType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Amount { value: 3},
            disputed: false
        }.process(&mut bank);
        Tx {
            type_: TxType::Resolve,
            client: 1,
            tx: 1,
            amount: Amount { value: 0},
            disputed: false
        }.process(&mut bank);
        Tx {
            type_: TxType::Withdrawal,
            client: 1,
            tx: 3,
            amount: Amount { value: 3},
            disputed: false
        }.process(&mut bank);

        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().client, &1);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().available.value, &0);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().held.value, &0);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().locked, &false);
        assert_eq!(&bank.transactions.lock().unwrap().get(&1).unwrap().disputed, &false);
        assert_eq!(bank.transactions.lock().unwrap().len() as i32, 3);
    }

    #[test]
    fn test_process_tx_resolve_wrong_tx_id() {
        let mut bank = Bank::new();
        Tx {
            type_: TxType::Deposit,
            client: 1,
            tx: 1,
            amount: Amount { value: 3},
            disputed: false
        }.process(&mut bank);
        Tx {
            type_: TxType::Dispute,
            client: 1,
            tx: 1,
            amount: Amount { value: 0},
            disputed: false
        }.process(&mut bank);
        Tx {
            type_: TxType::Resolve,
            client: 1,
            tx: 34,
            amount: Amount { value: 0},
            disputed: false
        }.process(&mut bank);

        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().client, &1);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().available.value, &0);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().held.value, &3);
        assert_eq!(&bank.accounts.lock().unwrap().get(&1).unwrap().locked, &false);
        assert_eq!(&bank.transactions.lock().unwrap().get(&1).unwrap().disputed, &true);
        assert_eq!(bank.transactions.lock().unwrap().len() as i32, 1);
    }
}