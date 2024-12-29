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
