
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0);
    let command = args.join(" ");
    let conn = dbus::blocking::Connection::new_session()?;
    let proxy = conn.with_proxy("com.ChristianWSmith.dsl", "/swaycommand", std::time::Duration::from_millis(5000));
    let (reply,): (String,) = proxy.method_call("com.ChristianWSmith.dsl", "SwayCommand", (command,))?;
    println!("{}", reply);
    Ok(())
}
