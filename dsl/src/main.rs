use std::io::{stdin, stdout, Write};
use swayipc::{Connection, Fallible};

fn main() -> Fallible<()> {
    let mut connection = Connection::new()?;
    Ok(())
}
