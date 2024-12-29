#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
mod transfer {
    use five::context;
}
mod account {
    use five::context;
    pub enum LedgerEntry<'a> {
        Deposit(&'a str, i32),
        Withdrawal(&'a str, i32),
    }
    pub mod account {
        use super::LedgerEntry;
        trait LedgerContract {
            fn push(&mut self, entry: LedgerEntry);
            fn as_vec(&self) -> Vec<LedgerEntry>;
        }
        struct Context<TLedgerRole>
        where
            TLedgerRole: LedgerContract,
        {
            ledger: TLedgerRole,
            accout_no: i64,
        }
        impl AccountCtx for Context<TLedgerRole> {
            fn deposit(&self, message: &str, amount: i32) {
                ledger_add(&mut self.ledger, LedgerEntry::Deposit(message, amount))
            }
            fn withdraw(&self, message: &str, amount: i32) {
                ledger_add(&mut self.ledger, LedgerEntry::Withdrawal(message, amount))
            }
        }
        pub trait AccountCtx {
            fn deposit(&self, message: &str, amount: i32);
            fn withdraw(&self, message: &str, amount: i32);
        }
        pub fn bind<TLedgerRole>(
            ledger: TLedgerRole,
            accout_no: i64,
        ) -> impl AccountCtx<TLedgerRole>
        where
            TLedgerRole: LedgerContract,
        {
            Context { ledger, accout_no }
        }
        fn ledger_add<T>(this: &mut T, entry: LedgerEntry)
        where
            T: LedgerContract,
        {
            this.push(entry);
        }
    }
}
use transfer::transfer::{SourceContract, SinkContract, TransferCtx, bind};
impl SourceContract for Account {
    fn get_balance(&self) -> i32 {
        self.balance
    }
}
impl SinkContract for Account {
    fn get_balance(&self) -> i32 {
        self.balance
    }
}
fn main() {
    let source = Account { balance: 100 };
    let sink = Account { balance: 50 };
    let mut ctx = bind(source, sink, 30);
    ctx.transfer();
}
