Role LedgerRole
Function: add
Res
Res
Res
Parsing token stream fn ledgerrole_add < TContract : LedgerRoleContract >
(mut this : & mut TContract, entry : & LedgerEntry) { { this.push(entry); } }
Parsed token stream
Adding ledgerrole_add
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
mod account {
    use five::context;
    pub enum LedgerEntry<'a> {
        Deposit(&'a str, i32),
        Withdrawal(&'a str, i32),
    }
    pub mod account {
        use super::LedgerEntry;
        impl Context {
            fn deposit(&self, message: &str, amount: i32) {
                self.ledger.add(LedgerEntry::Deposit(message, amount))
            }
            fn withdraw(&self, message: &str, amount: i32) {
                self.ledger.add(LedgerEntry::Withdrawal(message, amount))
            }
        }
        fn ledgerrole_add<TContract: LedgerRoleContract>(
            mut this: &mut TContract,
            entry: &LedgerEntry,
        ) {
            {
                this.push(entry);
            }
        }
    }
}
fn main() {
    let source = Account { balance: 100 };
}
