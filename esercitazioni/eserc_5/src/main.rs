mod ex1;
mod ex2;
mod ex3;

fn main() {
    match ex1::main_ex1() {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    // match ex2::main_ex2() {
    //     Ok(result) => println!("{}", result),
    //     Err(e) => eprintln!("Error: {}", e),
    // }

    match ex3::main_ex3() {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}
