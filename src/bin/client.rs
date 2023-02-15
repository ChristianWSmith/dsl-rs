fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0);
    let dbus_message = args.join(" ");
    let conn = dbus::blocking::Connection::new_session()?;
    let proxy = conn.with_proxy(
        dsl::constants::DBUS_DEST,
        dsl::constants::DBUS_PATH,
        std::time::Duration::from_millis(5000),
    );
    let (reply,): (String,) =
        proxy.method_call(dsl::constants::DBUS_DEST, dsl::constants::DBUS_METHOD, (dbus_message,))?;
    Ok(())
}
