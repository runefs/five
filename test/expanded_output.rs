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
        fn deposit(&mut self, message: String, amount: i32);
        fn withdraw(&mut self, message: String, amount: i32);
        fn balance(&self) -> i32;
        fn get_account_no(&self) -> i64;
    }
    pub trait LedgerContract {
        fn push(&mut self, entry: LedgerEntry);
        fn as_vec(&self) -> Vec<LedgerEntry>;
    }
    impl<TLedger: LedgerContract> Context<TLedger> {
        pub fn ledger_add(&mut self, entry: LedgerEntry) {
            self.ledger.push(entry.clone());
            self.ledger_log(entry.message());
        }
        pub fn ledger_log(&self, msg: String) {
            {
                ::std::io::_print(format_args!("{0}\n", msg));
            };
        }
    }
    impl<TLedger: LedgerContract> Account for Context<TLedger> {
        fn deposit(&mut self, message: String, amount: i32) {
            self.ledger_add(LedgerEntry::Deposit(message, amount))
        }
        fn withdraw(&mut self, message: String, amount: i32) {
            self.ledger_add(LedgerEntry::Withdrawal(message, amount))
        }
        fn balance(&self) -> i32 {
            self.ledger
                .as_vec()
                .iter()
                .map(|entry| match entry {
                    LedgerEntry::Deposit(_, amount) => *amount,
                    LedgerEntry::Withdrawal(_, amount) => -*amount,
                })
                .sum()
        }
        fn get_account_no(&self) -> i64 {
            self.account_no
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
    pub enum LedgerEntry {
        Deposit(String, i32),
        Withdrawal(String, i32),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for LedgerEntry {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                LedgerEntry::Deposit(__self_0, __self_1) => {
                    ::core::fmt::Formatter::debug_tuple_field2_finish(
                        f,
                        "Deposit",
                        __self_0,
                        &__self_1,
                    )
                }
                LedgerEntry::Withdrawal(__self_0, __self_1) => {
                    ::core::fmt::Formatter::debug_tuple_field2_finish(
                        f,
                        "Withdrawal",
                        __self_0,
                        &__self_1,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for LedgerEntry {
        #[inline]
        fn clone(&self) -> LedgerEntry {
            match self {
                LedgerEntry::Deposit(__self_0, __self_1) => {
                    LedgerEntry::Deposit(
                        ::core::clone::Clone::clone(__self_0),
                        ::core::clone::Clone::clone(__self_1),
                    )
                }
                LedgerEntry::Withdrawal(__self_0, __self_1) => {
                    LedgerEntry::Withdrawal(
                        ::core::clone::Clone::clone(__self_0),
                        ::core::clone::Clone::clone(__self_1),
                    )
                }
            }
        }
    }
    impl LedgerEntry {
        fn message(&self) -> String {
            match self {
                LedgerEntry::Deposit(msg, _) => msg.to_string(),
                LedgerEntry::Withdrawal(msg, _) => msg.to_string(),
            }
        }
    }
}
use account::LedgerContract;
fn main() {
    struct Aa {
        vec: Vec<account::LedgerEntry>,
    }
    impl Aa {
        fn new() -> Self {
            Self { vec: Vec::new() }
        }
    }
    impl LedgerContract for Aa {
        fn push(&mut self, entry: account::LedgerEntry) {
            self.vec.push(entry);
        }
        fn as_vec(&self) -> Vec<account::LedgerEntry> {
            self.vec.clone()
        }
    }
    let ledger = Aa::new();
    use account::Account;
    let mut account = account::bind(ledger, 67676555);
    account.deposit(String::from("Deposit 1"), 100);
    account.withdraw(String::from("Withdrawal 1"), 50);
    account.deposit(String::from("Deposit 2"), 200);
    account.withdraw(String::from("Withdrawal 2"), 100);
    {
        ::std::io::_print(format_args!("Balance: {0}\n", account.balance()));
    };
    match (&account.balance(), &150) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
