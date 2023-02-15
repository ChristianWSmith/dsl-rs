#[macro_use]
extern crate lazy_static;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    lazy_static! {
        static ref SWAY: std::sync::Mutex<swayipc::Connection> = std::sync::Mutex::new(swayipc::Connection::new().unwrap());
    }
    let c = dbus::blocking::Connection::new_session()?;
    c.request_name("com.ChristianWSmith.dsl", false, true, false)?;
    let mut cr = dbus_crossroads::Crossroads::new();
    let token = cr.register("com.ChristianWSmith.dsl", |b| {
        b.method("SwayCommand", ("command",), ("reply",), |_, _, (command,): (String,)| {
            SWAY.lock().unwrap().run_command(&command);
            Ok((format!("{}", &command),))
        });
    });
    cr.insert("/swaycommand", &[token], ());
    cr.serve(&c)?;
    Ok(())
}
