mod parser;

fn main() {
    match parser::parse_exp("foo bar") {
        Ok((_remain, _result)) => println!("OK!"),
        Err(e) => println!("Error: {}", e),
    };
    println!("Hello, world!");
}