mod discovery;
pub(crate) mod integrations;
pub mod items;
mod model;
pub mod settings;

#[cfg(test)]
mod tests;

pub use discovery::{agent_client_statuses, agent_client_versions};
pub use items::{
    agent_rule_targets, agent_skill_target_roots, scan_agent_items, scan_user_items,
    uninstall_user_item,
};
pub use model::{
    AgentClient, AgentClientItems, AgentClientStatus, AgentClientVersion, AgentItem, AgentItemKind,
    AgentItemStatus, AgentItemsSnapshot, REGISTERED_AGENT_CLIENTS,
};

pub(crate) use discovery::{chatgpt_desktop_version, command_client_version, command_path};
pub(crate) use items::{
    deduplicate, error_item, opencode_configuration_path, remove_codex_config_item,
    remove_codex_plugin, remove_skill_directory, scan_codex, scan_skills, write_json,
};

#[cfg(test)]
pub(crate) use discovery::{
    agent_client_statuses_with, command_path_in, command_version_from_output,
    newest_chatgpt_desktop_version,
};
#[cfg(test)]
pub(crate) use items::{scan_user_items_with_installation, uninstall_user_item_with_installation};
