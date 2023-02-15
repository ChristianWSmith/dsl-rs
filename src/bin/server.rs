#[macro_use]
extern crate lazy_static;

enum CommandType {
    MOVE,
    MOVE_TO_WORKSPACE,
    KILL
}

enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT
}

struct Command {
    command_type: CommandType,
    direction: Direction,
    workspace_name: String
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    lazy_static! {
        static ref SWAY: std::sync::Mutex<swayipc::Connection> = std::sync::Mutex::new(swayipc::Connection::new().unwrap());
        static ref CHANNEL: (async_priority_channel::Sender<Command, usize>, async_priority_channel::Receiver<Command, usize>) = async_priority_channel::unbounded();
    }

    let c = dbus::blocking::Connection::new_session()?;
    c.request_name("com.ChristianWSmith.dsl", false, true, false)?;
    let mut cr = dbus_crossroads::Crossroads::new();
    let token = cr.register("com.ChristianWSmith.dsl", |b| {
        b.method("SwayCommand", ("command",), ("reply",), |_, _, (command,): (String,)| {
            let dsl_command: Command = Command {
                command_type: CommandType::MOVE,
                direction: Direction::UP,
                workspace_name: "".to_string()
            };
            CHANNEL.0.send(dsl_command, 0);
            SWAY.lock().unwrap().run_command(&command);
            Ok((format!("{}", &command),))
        });
    });
    cr.insert("/swaycommand", &[token], ());
    cr.serve(&c)?;
    Ok(())
}
