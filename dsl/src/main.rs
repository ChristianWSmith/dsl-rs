use swayipc;

fn main() -> swayipc::Fallible<()> {
    let mut sway_connection = swayipc::Connection::new()?;

    Ok(())
}
