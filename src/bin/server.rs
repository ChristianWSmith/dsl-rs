#[macro_use]
extern crate lazy_static;

#[derive(Debug)]
enum CommandType {
    MOVE,
    MOVE_TO_WORKSPACE,
    KILL,
    LAYOUT,
}

#[derive(Debug)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    NONE,
}

#[derive(Debug)]
struct Command {
    command_type: CommandType,
    direction: Direction,
    workspace_name: String,
}

fn parse_dbus_message(dbus_message: &str) -> Command {
    Command {
        command_type: CommandType::KILL,
        direction: Direction::NONE,
        workspace_name: "".to_string(),
    }
}

fn command_processor(command_receiver: async_priority_channel::Receiver<Command, usize>) {
    let sway = swayipc::Connection::new().unwrap();
    loop {
        let command = sync_recv(&command_receiver);
        println!("{:?}", command);
    }
}

fn sway_event_listener(command_sender: async_priority_channel::Sender<Command, usize>) {
    let subs = [swayipc::EventType::Window];
    for event in swayipc::Connection::new().unwrap().subscribe(subs).unwrap() {
        let command: Command = Command {
            command_type: CommandType::LAYOUT,
            direction: Direction::NONE,
            workspace_name: "".to_string(),
        };
        sync_send(&command_sender, command, 1);
    }
}

fn dbus_listener(command_sender: async_priority_channel::Sender<Command, usize>) {
    let dbus_connection: dbus::blocking::Connection =
        dbus::blocking::Connection::new_session().unwrap();
    dbus_connection
        .request_name(dsl::constants::DBUS_DEST, false, true, false)
        .unwrap();
    let mut crossroads: dbus_crossroads::Crossroads = dbus_crossroads::Crossroads::new();
    let token = crossroads.register(dsl::constants::DBUS_DEST, |b| {
        b.method(
            dsl::constants::DBUS_METHOD,
            (dsl::constants::DBUS_ARG,),
            (dsl::constants::DBUS_REPLY,),
            move |_, _, (dbus_message,): (String,)| {
                let command: Command = parse_dbus_message(&dbus_message);
                sync_send(&command_sender, command, 0);
                Ok((format!("{}", &dbus_message),))
            },
        );
    });
    crossroads.insert(dsl::constants::DBUS_PATH, &[token], ());
    crossroads.serve(&dbus_connection);
}

fn sync_send(
    sender: &async_priority_channel::Sender<Command, usize>,
    command: Command,
    priority: usize,
) {
    futures::executor::block_on(async_send(sender, command, priority));
}

fn sync_recv(receiver: &async_priority_channel::Receiver<Command, usize>) -> Command {
    futures::executor::block_on(async_recv(receiver))
}

async fn async_send(
    sender: &async_priority_channel::Sender<Command, usize>,
    command: Command,
    priority: usize,
) {
    sender.send(command, priority).await;
}

async fn async_recv(receiver: &async_priority_channel::Receiver<Command, usize>) -> Command {
    receiver.recv().await.unwrap().0
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (command_sender, command_receiver): (
        async_priority_channel::Sender<Command, usize>,
        async_priority_channel::Receiver<Command, usize>,
    ) = async_priority_channel::unbounded();

    let dbus_listener_command_sender = command_sender.clone();
    let sway_listener_command_sender = command_sender.clone();

    let dbus_handle = std::thread::spawn(move || {
        dbus_listener(dbus_listener_command_sender);
    });
    let sway_handle = std::thread::spawn(move || {
        sway_event_listener(sway_listener_command_sender);
    });
    let processor_handle = std::thread::spawn(move || {
        command_processor(command_receiver);
    });

    dbus_handle.join();
    sway_handle.join();
    processor_handle.join();
    Ok(())
}
