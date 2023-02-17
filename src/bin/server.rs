fn get_parent_map(node: &swayipc::Node) -> std::collections::HashMap<i64, &swayipc::Node> {
    let mut parent_map: std::collections::HashMap<i64, &swayipc::Node> =
        std::collections::HashMap::new();
    let mut layer = vec![node];
    parent_map.insert(node.id, node);
    let mut next_layer = vec![];
    while layer.len() > 0 {
        for node in layer {
            for child in &node.nodes {
                parent_map.insert(child.id, node);
                next_layer.push(child);
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

fn find_focused(node: &swayipc::Node) -> Option<&swayipc::Node> {
    let mut layer = vec![node];
    let mut next_layer = vec![];
    while layer.len() > 0 {
        for node in layer {
            if node.focused {
                return Some(node);
            } else {
                next_layer.extend(&node.nodes);
            }
        }
        layer = next_layer;
        next_layer = vec![];
    }
    None
}

fn find_workspace<'a>(
    node: &'a swayipc::Node,
    parent_map: std::collections::HashMap<i64, &'a swayipc::Node>,
) -> Option<&'a swayipc::Node> {
    let mut temp_node = node;
    while temp_node != parent_map[&temp_node.id] {
        if temp_node.node_type == swayipc::NodeType::Workspace {
            return Some(temp_node);
        } else {
            temp_node = parent_map[&temp_node.id];
        }
    }
    None
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
    let master_mark = format!("master-{:}", workspace.name.as_ref().unwrap());
    let stack_mark = format!("stack-{:}", workspace.name.as_ref().unwrap());
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
    let master_mark = format!("master-{:}", workspace.name.as_ref().unwrap());
    let stack_mark = format!("stack-{:}", workspace.name.as_ref().unwrap());
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

fn promote(workspace: &swayipc::Node) -> (String, String) {
    let master_mark = format!("master-{:}", workspace.name.as_ref().unwrap());
    let stack_top = workspace.nodes.get(1).unwrap().nodes.get(0).unwrap();
    let pre = format!(
        "[con_id={}] mark --add {}; [con_id={}] move container to mark {:}; ",
        stack_top.id,
        dsl::constants::SWAY_TEMP_MASTER_MARK,
        stack_top.id,
        master_mark
    );
    let post = format!(
        "[con_id={}] unmark {}; ",
        stack_top.id,
        dsl::constants::SWAY_TEMP_MASTER_MARK
    );
    (pre, post)
}

fn make_move_to_workspace_command(
    from_workspace: &swayipc::Node,
    to_workspace_name: String,
    to_workspace_nodes: &Vec<swayipc::Node>,
    focus: &swayipc::Node,
    focus_follow: bool,
) -> String {
    // TODO: this
    "".to_string()
}

fn process_layout(sway: &mut swayipc::Connection) {
    let mut tree = sway.get_tree().unwrap();
    let focused = find_focused(&tree).unwrap();
    let refocus_command = format!("[con_id={}] focus; ", focused.id);

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
    // TODO: finish this
    println!("process_move {:?}", tokens);

    if tokens.len() == 0 {
        return;
    }

    let tree = sway.get_tree().unwrap();

    let (forward, backward) = match *tokens.get(0).unwrap() {
        "up" => ("up", "down"),
        "down" => ("down", "up"),
        "left" => ("left", "right"),
        "right" => ("right", "left"),
        _ => return,
    };
}

fn process_move_to_workspace(sway: &mut swayipc::Connection, tokens: Vec<&str>) {
    if tokens.len() == 0 {
        return;
    }
    let to_workspace_name = *tokens.get(0).unwrap();
    let tree = sway.get_tree().unwrap();
    let parents = get_parent_map(&tree);
    let focused = find_focused(&tree).unwrap();
    let from_workspace = find_workspace(focused, parents).unwrap();
    let workspaces = get_workspaces(&tree);
    let mut to_workspace_nodes: &Vec<swayipc::Node> = &vec![];
    for workspace in workspaces {
        if workspace.name.as_ref().unwrap() == &to_workspace_name.to_string() {
            to_workspace_nodes = &workspace.nodes;
            break;
        }
    }
    let move_to_workspace_command = make_move_to_workspace_command(
        from_workspace,
        to_workspace_name.to_string(),
        to_workspace_nodes,
        focused,
        false,
    );
    sway.run_command(move_to_workspace_command).unwrap();
}

fn process_kill(sway: &mut swayipc::Connection) {
    let tree = sway.get_tree().unwrap();
    let parents = get_parent_map(&tree);
    let focused = find_focused(&tree).unwrap();
    let parent = *parents.get(&focused.id).unwrap();
    let grandparent = *parents.get(&parent.id).unwrap();
    if grandparent.node_type == swayipc::NodeType::Workspace {
        let master_mark = format!("master-{:}", grandparent.name.as_ref().unwrap());
        if parent.marks.contains(&master_mark) {
            let mut kill_command: Vec<String> = vec![];
            let (pre, post) = promote(grandparent);
            kill_command.push(pre);
            kill_command.push(format!("[con_id={}] kill; ", focused.id));
            kill_command.push(post);
            sway.run_command(kill_command.concat()).unwrap();
        } else {
            sway.run_command("kill").unwrap();
        }
    } else {
        sway.run_command("kill").unwrap();
    }
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
