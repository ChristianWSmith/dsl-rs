fn get_parent_map(node: &swayipc::Node) -> std::collections::HashMap<i64, &swayipc::Node> {
    let mut parent_map: std::collections::HashMap<i64, &swayipc::Node> =
        std::collections::HashMap::new();
    let mut layer = vec![node];
    let mut next_layer = vec![];
    while layer.len() > 0 {
        for node in layer {
            for child in &node.nodes {
                parent_map.insert(child.id, node);
            }
        }
        layer = next_layer;
        next_layer = vec![];
    }
    parent_map
}

fn get_workspaces(node: &swayipc::Node) -> Vec<&swayipc::Node> {
    let mut out: Vec<&swayipc::Node> = vec![];
    let mut layer = vec![node];
    let mut next_layer = vec![];
    while layer.len() > 0 {
        for node in layer {
            if node.node_type == swayipc::NodeType::Workspace {
                out.push(node)
            } else {
                next_layer.extend(&node.nodes);
            }
        }
        layer = next_layer;
        next_layer = vec![];
    }
    out
}

fn get_leaf_containers(node: &swayipc::Node) -> Vec<&swayipc::Node> {
    let mut out: Vec<&swayipc::Node> = vec![];
    if node.nodes.len() == 0 {
        if node.node_type == swayipc::NodeType::Con {
            out.push(node);
        }
    } else {
        for child in &node.nodes {
            out.extend(get_leaf_containers(&child));
        }
    }
    out
}

fn find_focus_id(node: &swayipc::Node) -> i64 {
    let mut layer = vec![node];
    let mut next_layer = vec![];
    while layer.len() > 0 {
        for node in layer {
            if node.focused {
                return node.id;
            } else {
                next_layer.extend(&node.nodes);
            }
        }
        layer = next_layer;
        next_layer = vec![];
    }
    -1
}

fn enforce_splitting_workspace(workspace: &swayipc::Node) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    let leaves = get_leaf_containers(&workspace);
    if leaves.len() == 1 {
        let window = *leaves.get(0).unwrap();
        if !workspace.nodes.contains(window) {
            out.push(format!("[con_id={}] move up; ", window.id));
        }
        out.push(format!("[con_id={}] splith; ", window.id));
    } else if leaves.len() == 2 {
        let left = *leaves.get(0).unwrap();
        let right = *leaves.get(1).unwrap();
        out.push(format!("[con_id={}] splitv; ", left.id));
        out.push(format!("[con_id={}] splitv; ", right.id));
    }
    out
}

fn enforce_splitting(workspaces: Vec<&swayipc::Node>) -> String {
    let mut out: Vec<String> = vec![];
    for workspace in workspaces {
        out.extend(enforce_splitting_workspace(workspace));
    }
    out.concat()
}

fn enforce_marking_workspace(workspace: &swayipc::Node) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    let master_mark = format!("master-{:}", workspace.name.clone().unwrap());
    let stack_mark = format!("stack-{:}", workspace.name.clone().unwrap());
    if workspace.nodes.len() == 2 {
        let left = workspace.nodes.get(0).unwrap();
        let right = workspace.nodes.get(1).unwrap();
        if !left.marks.contains(&master_mark) {
            out.push(format!(
                "[con_id={}] mark --add {:}; ",
                left.id, master_mark
            ));
        }
        if !right.marks.contains(&stack_mark) {
            out.push(format!(
                "[con_id={}] mark --add {:}; ",
                right.id, stack_mark
            ));
        }
    }
    out
}

fn enforce_marking(workspaces: Vec<&swayipc::Node>) -> String {
    let mut out: Vec<String> = vec![];
    for workspace in workspaces {
        out.extend(enforce_marking_workspace(workspace));
    }
    out.concat()
}

fn enforce_eviction_workspace(workspace: &swayipc::Node) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    let master_mark = format!("master-{:}", workspace.name.clone().unwrap());
    let stack_mark = format!("stack-{:}", workspace.name.clone().unwrap());
    if workspace.nodes.len() > 1 {
        let parent = workspace.nodes.get(0).unwrap();
        if parent.marks.contains(&master_mark) {
            let mut master_found = false;
            for child in &parent.nodes {
                let is_temp_master = child
                    .marks
                    .contains(&dsl::constants::SWAY_TEMP_MASTER_MARK.to_string());
                if master_found {
                    if !is_temp_master {
                        out.push(format!(
                            "[con_id={}] move container to mark {:}; ",
                            child.id, stack_mark
                        ));
                    }
                } else if !is_temp_master {
                    master_found = true
                }
            }
        }
    }
    out
}

fn enforce_eviction(workspaces: Vec<&swayipc::Node>) -> String {
    let mut out: Vec<String> = vec![];
    for workspace in workspaces {
        out.extend(enforce_eviction_workspace(workspace));
    }
    out.concat()
}

fn process_layout(sway: &mut swayipc::Connection) {
    let mut tree = sway.get_tree().unwrap();
    let focus_id = find_focus_id(&tree);
    let refocus_command = format!("[con_id={}] focus; ", focus_id);

    let mut ran_command = false;

    let splitting_command = enforce_splitting(get_workspaces(&tree));
    if splitting_command != "" {
        sway.run_command(splitting_command).unwrap();
        ran_command = true;
    }

    tree = sway.get_tree().unwrap();
    let marking_command = enforce_marking(get_workspaces(&tree));
    if marking_command != "" {
        sway.run_command(marking_command).unwrap();
        ran_command = true;
    }

    tree = sway.get_tree().unwrap();
    let eviction_command = enforce_eviction(get_workspaces(&tree));
    if eviction_command != "" {
        sway.run_command(eviction_command).unwrap();
        ran_command = true;
    }

    if ran_command {
        sway.run_command(refocus_command).unwrap();
    }
}

fn process_move(sway: &mut swayipc::Connection, tokens: Vec<&str>) {
    println!("process_move {:?}", tokens);
    let tree = sway.get_tree().unwrap();
}

fn process_move_to_workspace(sway: &mut swayipc::Connection, tokens: Vec<&str>) {
    println!("process_move_to_workspace {:?}", tokens);
    let tree = sway.get_tree().unwrap();
}

fn process_kill(sway: &mut swayipc::Connection) {
    println!("process_kill");
    let tree = sway.get_tree().unwrap();
}

fn command_processor(command_receiver: async_priority_channel::Receiver<String, usize>) {
    let mut sway: swayipc::Connection = swayipc::Connection::new().unwrap();
    loop {
        let command: String = sync_recv(&command_receiver);
        let mut tokens: Vec<&str> = command.split_whitespace().collect();
        match tokens.remove(0) {
            dsl::constants::CMD_LAYOUT => process_layout(&mut sway),
            dsl::constants::CMD_MOVE => process_move(&mut sway, tokens),
            dsl::constants::CMD_MOVE_TO_WORKSPACE => process_move_to_workspace(&mut sway, tokens),
            dsl::constants::CMD_KILL => process_kill(&mut sway),
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
