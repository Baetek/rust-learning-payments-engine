use std::env;
use std::env::Args;
use std::error::Error;

use bank::bank::Bank;

/// Takes in a space separated list of csv file paths from stdin
/// Simultaneously processes all contained transactions to a central bank
/// and writes the final resulting state of all bank client accounts to stdout
///
/// In the event that one CSV file is malformed, processing continues on the rest.
/// Unless an unexpected crash occurs where the bank data is poisoned.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let bank = Bank::new();

    let processes = get_csv_paths().into_iter().map(
        |csv_path| spawn_tokio_process_for_csv(csv_path, &bank)
    );
    for process in processes {
        match process.await {
            _ => (),
        }
    }

    bank.write_accounts()?;
    Ok(())
}

/// Spawns and returns a process for the given csv
fn spawn_tokio_process_for_csv(csv_path: String, bank: &Bank) -> tokio::task::JoinHandle<()> {
    let tokio_bank = Bank::new_for_tokio(bank);
    tokio::spawn(async move {
        Bank::process_transactions_from_csv_path(
            &csv_path, tokio_bank
        ).await;
    })
}

/// Gets the csv paths from stdin
fn get_csv_paths() -> Args {
    let mut args = env::args().into_iter();
    args.next();
    args
}

