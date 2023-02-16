fn command_processor(command_receiver: async_priority_channel::Receiver<String, usize>) {
    let sway = swayipc::Connection::new().unwrap();
    loop {
        let command = sync_recv(&command_receiver);
        println!("{}", command);
    }
}

fn sway_event_listener(command_sender: async_priority_channel::Sender<String, usize>) {
    let subs = [swayipc::EventType::Window];
    for event in swayipc::Connection::new().unwrap().subscribe(subs).unwrap() {
        sync_send(&command_sender, "layout".to_string(), 1);
    }
}

fn dbus_listener(command_sender: async_priority_channel::Sender<String, usize>) {
    let dbus_connection: dbus::blocking::Connection =
        dbus::blocking::Connection::new_session().unwrap();
    dbus_connection
        .request_name(dsl::constants::DBUS_DEST, false, true, false)
        .unwrap();
    let mut crossroads: dbus_crossroads::Crossroads = dbus_crossroads::Crossroads::new();
    let token = crossroads.register(dsl::constants::DBUS_DEST, |builder| {
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
    crossroads.serve(&dbus_connection);
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
    sender.send(item, priority).await;
}

async fn async_recv<I, P: std::cmp::Ord>(receiver: &async_priority_channel::Receiver<I, P>) -> I {
    receiver.recv().await.unwrap().0
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (command_sender, command_receiver): (
        async_priority_channel::Sender<String, usize>,
        async_priority_channel::Receiver<String, usize>,
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
