fn process_layout(sway: &swayipc::Connection) {
    println!("process_layout");
}

fn process_move(sway: &swayipc::Connection, tokens: Vec<&str>) {
    println!("process_move {:?}", tokens);
}

fn process_move_to_workspace(sway: &swayipc::Connection, tokens: Vec<&str>) {
    println!("process_move_to_workspace {:?}", tokens);
}

fn process_kill(sway: &swayipc::Connection) {
    println!("process_kill");
}

fn command_processor(command_receiver: async_priority_channel::Receiver<String, usize>) {
    let sway: swayipc::Connection = swayipc::Connection::new().unwrap();
    loop {
        let command: String = sync_recv(&command_receiver);
        let mut tokens: Vec<&str> = command.split_whitespace().collect();
        match tokens.remove(0) {
            dsl::constants::CMD_LAYOUT => process_layout(&sway),
            dsl::constants::CMD_MOVE => process_move(&sway, tokens),
            dsl::constants::CMD_MOVE_TO_WORKSPACE => process_move_to_workspace(&sway, tokens),
            dsl::constants::CMD_KILL => process_kill(&sway),
            _ => continue,
        }
    }
}

fn sway_event_listener(command_sender: async_priority_channel::Sender<String, usize>) {
    let subs: Vec<swayipc::EventType> = vec![swayipc::EventType::Window];
    let sway_window_changes: Vec<swayipc::WindowChange> = vec![
        swayipc::WindowChange::New,
        swayipc::WindowChange::Close,
        swayipc::WindowChange::Move,
    ];
    for event in swayipc::Connection::new().unwrap().subscribe(subs).unwrap() {
        let window_event: Box<swayipc::WindowEvent> = match event.unwrap() {
            swayipc::Event::Window(c) => c,
            _ => unreachable!(),
        };
        if sway_window_changes.contains(&window_event.change) {
            sync_send(&command_sender, dsl::constants::CMD_LAYOUT.to_string(), 1);
        }
    }
}

fn dbus_listener(command_sender: async_priority_channel::Sender<String, usize>) {
    let dbus_connection: dbus::blocking::Connection =
        dbus::blocking::Connection::new_session().unwrap();
    dbus_connection
        .request_name(dsl::constants::DBUS_DEST, false, true, false)
        .unwrap();
    let mut crossroads: dbus_crossroads::Crossroads = dbus_crossroads::Crossroads::new();
    let token: dbus_crossroads::IfaceToken<()> =
        crossroads.register(dsl::constants::DBUS_DEST, |builder| {
            builder.method(
                dsl::constants::DBUS_METHOD,
                (dsl::constants::DBUS_ARG,),
                (dsl::constants::DBUS_REPLY,),
                move |_, _, (dbus_message,): (String,)| {
                    sync_send(&command_sender, dbus_message, 0);
                    Ok(("",))
                },
            );
        });
    crossroads.insert(dsl::constants::DBUS_PATH, &[token], ());
    crossroads.serve(&dbus_connection).unwrap();
}

fn sync_send<I, P: std::cmp::Ord>(
    sender: &async_priority_channel::Sender<I, P>,
    item: I,
    priority: P,
) {
    futures::executor::block_on(async_send(sender, item, priority));
}

fn sync_recv<I, P: std::cmp::Ord>(receiver: &async_priority_channel::Receiver<I, P>) -> I {
    futures::executor::block_on(async_recv(receiver))
}

async fn async_send<I, P: std::cmp::Ord>(
    sender: &async_priority_channel::Sender<I, P>,
    item: I,
    priority: P,
) {
    sender.send(item, priority).await.unwrap();
}

async fn async_recv<I, P: std::cmp::Ord>(receiver: &async_priority_channel::Receiver<I, P>) -> I {
    receiver.recv().await.unwrap().0
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (command_sender, command_receiver): (
        async_priority_channel::Sender<String, usize>,
        async_priority_channel::Receiver<String, usize>,
    ) = async_priority_channel::unbounded();

    let command_sender_clone: async_priority_channel::Sender<String, usize> =
        command_sender.clone();

    let dbus_handle: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        dbus_listener(command_sender);
    });
    let sway_handle: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        sway_event_listener(command_sender_clone);
    });
    let processor_handle: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        command_processor(command_receiver);
    });

    dbus_handle.join().unwrap();
    sway_handle.join().unwrap();
    processor_handle.join().unwrap();
    Ok(())
}
