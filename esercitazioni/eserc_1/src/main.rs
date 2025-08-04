use clap::Parser;

mod ex1;
mod ex2;

fn main() {
    let args = ex1::Args::parse();

    match ex1::main_ex1(&args.slug_in) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    match ex2::main_ex2() {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}
