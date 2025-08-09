mod ex1;
mod ex2;

fn main() {
    match ex1::main_ex1() {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    match ex2::main_ex2() {
        Ok(result) => return,
        Err(e) => eprintln!("Error: {}", e),
    } 
}