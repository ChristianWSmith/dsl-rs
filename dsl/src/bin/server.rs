fn main() -> Result<(), Box<dyn std::error::Error>> {
    let c = dbus::blocking::Connection::new_session()?;
    c.request_name("com.ChristianWSmith.dsl", false, true, false)?;
    let mut cr = dbus_crossroads::Crossroads::new();
    let token = cr.register("com.ChristianWSmith.dsl", |b| {
        b.method("SwayCommand", ("command",), ("reply",), |_, _, (command,): (String,)| {
            // TODO: how can we connect once and reuse this connection, rather than reconnecting each time?
            let mut sway_connection = swayipc::Connection::new().unwrap();
            sway_connection.run_command(&command);
            Ok((format!("{}", &command),))
        });
    });
    cr.insert("/swaycommand", &[token], ());
    cr.serve(&c)?;
    Ok(())
}
