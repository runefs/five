mod transfer;
mod account;

use transfer::transfer::{SourceContract,SinkContract, TransferCtx, bind};

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

    let mut ctx = bind(
        source,
        sink,
        30,
    );

    ctx.transfer();
}