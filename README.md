# rust-learning-payments-engine

##To-Do
 
- Switch to zero copy serialization / deserialization if possible for this data set.
- CI/CD integration to run test on commits 
- More unit tests to cover more cases
- More documentation and refactor to move getting client account of transaction::Tx:process to Bank::get_account
- Sanity checks on data - e.g held amount should probably never be negative
- doc examples (especially for how to use the library) and doc tests
- Logging
- More testing of error cases, errors seem to bubble up to where needed and appropriately handled but maybe missed something