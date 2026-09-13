pub(crate) mod catalog;
pub(crate) mod install;
pub(crate) mod mcp;
pub(crate) mod mcp_manifest;
pub(crate) mod model;
pub(crate) mod rules;
pub(crate) mod skills;
pub(crate) mod support;

pub use catalog::{list_extensions, read_extension_readme, update_all_skill_extensions};
pub use install::{install_extension, read_mcp_install_manifest};
pub use model::{
    ExtensionCatalogSnapshot, ExtensionInstallAction, ExtensionInstallRequest,
    ExtensionInstallResult, ExtensionKind, ExtensionPackage, McpInstallPreview,
};

pub(crate) use model::SKILLS_PREFIX;
pub(crate) use support::{atomic_write, decode_extension_file, storage_error};

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
