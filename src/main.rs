use std::error::Error;
mod custom_error;
use custom_error::CustomError;
mod utils;

fn main() -> Result<(), Box<dyn Error>>{
    println!("Hello, world!");
    Ok(())
}
