Creating bind function...
Created bind_fn_name
Creating bind_fn_body with ty_generics: < TLedger >
field_names: ledger, account_no
Created context_type Context < TLedger >
Created bind_fn_body
Creating return type with trait_name: Account
Created return_type
Creating params...
Field names and types:
  name: Some(Ident { ident: "ledger", span: #0 bytes(1286..1292) }), type: TLedger
  name: Some(Ident { ident: "account_no", span: #0 bytes(1315..1325) }), type: i64
Creating param for name: Some(Ident { ident: "ledger", span: #0 bytes(1286..1292) })
With type: TLedger
Created parameter
Creating param for name: Some(Ident { ident: "account_no", span: #0 bytes(1315..1325) })
With type: i64
Created parameter
Created all params: 2 parameters
Creating FunctionDescription...
Compiling bind_fn...
Emitting bind_fn...
Creating function signature
Creating final tokens...
Creating function signature
Creating function signature
Creating function signature
Creating function signature
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
mod account {
    struct Context<TLedger: LedgerContract> {
        ledger: TLedger,
        account_no: i64,
    }
    pub trait Account {
        fn deposit(&mut self, message: &str, amount: i32);
        fn withdraw(&mut self, message: &str, amount: i32);
    }
    pub trait LedgerContract {
        fn push(&mut self, entry: LedgerEntry);
        fn as_vec(&self) -> Vec<LedgerEntry>;
    }
    impl<TLedger: LedgerContract> Context<TLedger> {
        pub fn ledger_add(entry: LedgerEntry) {
            self.ledger.push(entry.clone());
            self.ledger_log(entry.message());
        }
        pub fn ledger_log(msg: &str) {
            {
                ::std::io::_print(format_args!("{0}\n", msg));
            };
        }
    }
    impl<TLedger: LedgerContract> Account for Context<TLedger> {
        fn deposit(message: &str, amount: i32) {
            self.ledger_add(LedgerEntry::Deposit(message, amount))
        }
        fn withdraw(message: &str, amount: i32) {
            self.ledger_add(LedgerEntry::Withdrawal(message, amount))
        }
    }
    pub fn bind<TLedger: LedgerContract>(
        ledger: TLedger,
        account_no: i64,
    ) -> impl Account {
        Context::<TLedger> {
            ledger: ledger,
            account_no: account_no,
        }
    }
    pub enum LedgerEntry<'a> {
        Deposit(&'a str, i32),
        Withdrawal(&'a str, i32),
    }
    impl<'a> Clone for LedgerEntry<'a> {
        fn clone(&self) -> Self {
            match self {
                LedgerEntry::Deposit(msg, amount) => LedgerEntry::Deposit(msg, *amount),
                LedgerEntry::Withdrawal(msg, amount) => {
                    LedgerEntry::Withdrawal(msg, *amount)
                }
            }
        }
    }
    impl<'a> LedgerEntry<'a> {
        fn message(&self) -> &str {
            match self {
                LedgerEntry::Deposit(msg, _) => msg,
                LedgerEntry::Withdrawal(msg, _) => msg,
            }
        }
    }
}
fn main() {
    struct Account {
        balance: i32,
    }
    let source = Account { balance: 100 };
    {
        ::std::io::_print(format_args!("Balance: {0}\n", source.balance));
    };
}
