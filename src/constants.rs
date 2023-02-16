pub const DBUS_DEST: &'static str = "com.ChristianWSmith.dsl";
pub const DBUS_PATH: &'static str = "/dslcommand";
pub const DBUS_METHOD: &'static str = "DSLCommand";
pub const DBUS_ARG: &'static str = "dbus_message";
pub const DBUS_REPLY: &'static str = "reply";

pub const CMD_LAYOUT: &'static str = "layout";
pub const CMD_MOVE: &'static str = "move";
pub const CMD_MOVE_TO_WORKSPACE: &'static str = "move_to_workspace";
pub const CMD_KILL: &'static str = "kill";

pub const SWAY_MASTER_MARK: &'static str = "master-{:?}";
pub const SWAY_STACK_MARK: &'static str = "stack-{:?}";
pub const SWAY_TEMP_MASTER_MARK: &'static str = "_temp_master";
pub const SWAY_TEMP_SWAP_MARK: &'static str = "_swap";
