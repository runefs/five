mod account;


fn main() {   
    struct Account {
        balance: i32
    }
    
    let source = Account { balance: 100 };
    println!("Balance: {}", source.balance);
}