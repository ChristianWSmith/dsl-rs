#[macro_use]
extern crate lazy_static;


lazy_static! {
    static ref SWAY: std::sync::Mutex<swayipc::Connection> =
        std::sync::Mutex::new(swayipc::Connection::new().unwrap());
    static ref CHANNEL: (
        async_priority_channel::Sender<Command, usize>,
        async_priority_channel::Receiver<Command, usize>
    ) = async_priority_channel::unbounded();
}

enum CommandType {
    MOVE,
    MOVE_TO_WORKSPACE,
    KILL,
    LAYOUT,
}

enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    NONE,
}

struct Command {
    command_type: CommandType,
    direction: Direction,
    workspace_name: String,
}


fn parse_dbus_message(dbus_message: &str) -> Command {
    Command {
        command_type: CommandType::KILL,
        direction: Direction::NONE,
        workspace_name: "".to_string()
    }
}

fn sway_event_listener() -> swayipc::Fallible<()>  {
    let subs = [
        swayipc::EventType::Workspace,
        swayipc::EventType::Input,
        swayipc::EventType::Tick,
        swayipc::EventType::Shutdown,
        swayipc::EventType::Mode,
        swayipc::EventType::Window,
        swayipc::EventType::BarStateUpdate,
        swayipc::EventType::BarConfigUpdate,
        swayipc::EventType::Binding,
    ];
    for event in swayipc::Connection::new()?.subscribe(subs)? {
        println!("{:?}\n", event?)
    }
    Ok(())
}

fn dbus_listener() -> Result<(), Box<dyn std::error::Error>> {
    let dbus_connection: dbus::blocking::Connection = dbus::blocking::Connection::new_session()?;
    dbus_connection.request_name(dsl::constants::DBUS_DEST, false, true, false)?;
    let mut crossroads: dbus_crossroads::Crossroads = dbus_crossroads::Crossroads::new();
    let token = crossroads.register(dsl::constants::DBUS_DEST, |b| {
        b.method(
            dsl::constants::DBUS_METHOD,
            (dsl::constants::DBUS_ARG,),
            (dsl::constants::DBUS_REPLY,),
            |_, _, (dbus_message,): (String,)| {
                let command: Command = parse_dbus_message(&dbus_message);
                CHANNEL.0.send(command, 0);
                SWAY.lock().unwrap().run_command(&dbus_message);
                Ok((format!("{}", &dbus_message),))
            },
        );
    });
    crossroads.insert(dsl::constants::DBUS_PATH, &[token], ());
    crossroads.serve(&dbus_connection)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dbus_connection: dbus::blocking::Connection = dbus::blocking::Connection::new_session()?;
    dbus_connection.request_name(dsl::constants::DBUS_DEST, false, true, false)?;
    let mut crossroads: dbus_crossroads::Crossroads = dbus_crossroads::Crossroads::new();
    let token = crossroads.register(dsl::constants::DBUS_DEST, |b| {
        b.method(
            dsl::constants::DBUS_METHOD,
            (dsl::constants::DBUS_ARG,),
            (dsl::constants::DBUS_REPLY,),
            |_, _, (dbus_message,): (String,)| {
                let command: Command = parse_dbus_message(&dbus_message);
                CHANNEL.0.send(command, 0);
                SWAY.lock().unwrap().run_command(&dbus_message);
                Ok((format!("{}", &dbus_message),))
            },
        );
    });
    crossroads.insert(dsl::constants::DBUS_PATH, &[token], ());
    crossroads.serve(&dbus_connection)?;
    Ok(())
}
