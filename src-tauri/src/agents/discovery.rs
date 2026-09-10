use std::{
    env,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use super::{
    integrations::integration,
    model::{AgentClient, AgentClientStatus, AgentClientVersion, REGISTERED_AGENT_CLIENTS},
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{ERROR_NO_MORE_ITEMS, WIN32_ERROR},
        System::{
            Registry::{
                RegCloseKey, RegEnumKeyW, RegGetValueW, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER,
                HKEY_LOCAL_MACHINE, KEY_READ, RRF_RT_REG_SZ,
            },
            Threading::CREATE_NO_WINDOW,
        },
    },
};

const VERSION_COMMAND_TIMEOUT: Duration = Duration::from_secs(5);

#[cfg(windows)]
const CHATGPT_PACKAGE_FAMILY_PREFIX: &str = "OpenAI.Codex_";

#[cfg(windows)]
const CHATGPT_UNINSTALL_NAME_PREFIX: &str = "ChatGPT";

#[cfg(windows)]
const CHATGPT_UNINSTALL_PUBLISHER: &str = "OpenAI";

#[cfg(windows)]
const PACKAGE_FAMILIES_KEY: &str = "Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\AppModel\\Repository\\Families";

#[cfg(windows)]
const UNINSTALL_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall";

pub fn agent_client_versions(clients: Vec<AgentClient>) -> Vec<AgentClientVersion> {
    clients
        .into_iter()
        .map(|client| {
            thread::spawn(move || AgentClientVersion {
                client,
                version: integration(client).version(),
            })
        })
        .filter_map(|task| task.join().ok())
        .collect()
}

pub fn agent_client_statuses() -> Vec<AgentClientStatus> {
    agent_client_statuses_with(agent_client_is_installed, agent_client_versions)
}

pub(crate) fn agent_client_statuses_with(
    is_installed: impl Fn(AgentClient) -> bool,
    load_versions: impl Fn(Vec<AgentClient>) -> Vec<AgentClientVersion>,
) -> Vec<AgentClientStatus> {
    let installed_clients = REGISTERED_AGENT_CLIENTS
        .into_iter()
        .filter(|client| is_installed(*client))
        .collect::<Vec<_>>();
    let versions = load_versions(installed_clients.clone());

    REGISTERED_AGENT_CLIENTS
        .into_iter()
        .map(|client| AgentClientStatus {
            client,
            installed: installed_clients.contains(&client),
            version: versions
                .iter()
                .find(|version| version.client == client)
                .and_then(|version| version.version.clone()),
        })
        .collect()
}

pub(crate) fn agent_client_is_installed(client: AgentClient) -> bool {
    integration(client).is_installed()
}

pub(crate) fn command_path(command: &str) -> Option<PathBuf> {
    let paths = env::var_os("PATH")
        .map(|value| env::split_paths(&value).collect::<Vec<_>>())
        .unwrap_or_default();
    let extensions = command_extensions();
    command_path_in(command, &paths, &extensions)
}

/// ChatGPT 桌面端有应用商店包和离线安装包两种安装形态，分别注册在当前用户的包仓库和卸载注册表。
#[cfg(windows)]
pub(crate) fn chatgpt_desktop_version() -> Option<String> {
    newest_version(
        chatgpt_package_versions()
            .into_iter()
            .chain(chatgpt_offline_install_versions()),
    )
}

#[cfg(not(windows))]
pub(crate) fn chatgpt_desktop_version() -> Option<String> {
    None
}

#[cfg(windows)]
fn chatgpt_package_versions() -> Vec<String> {
    subkey_names(HKEY_CURRENT_USER, PACKAGE_FAMILIES_KEY)
        .into_iter()
        .filter(|family| family.starts_with(CHATGPT_PACKAGE_FAMILY_PREFIX))
        .flat_map(|family| {
            subkey_names(
                HKEY_CURRENT_USER,
                &format!("{PACKAGE_FAMILIES_KEY}\\{family}"),
            )
        })
        .filter_map(|package| chatgpt_package_version(&package))
        .collect()
}

#[cfg(windows)]
fn chatgpt_offline_install_versions() -> Vec<String> {
    let mut versions = Vec::new();
    for hive in [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
        for entry in subkey_names(hive, UNINSTALL_KEY) {
            let entry_key = format!("{UNINSTALL_KEY}\\{entry}");
            let (Some(display_name), Some(display_version)) = (
                registry_string(hive, &entry_key, "DisplayName"),
                registry_string(hive, &entry_key, "DisplayVersion"),
            ) else {
                continue;
            };
            let publisher = registry_string(hive, &entry_key, "Publisher").unwrap_or_default();
            if let Some(version) =
                chatgpt_offline_install_version(&display_name, &publisher, &display_version)
            {
                versions.push(version);
            }
        }
    }
    versions
}

#[cfg(windows)]
pub(crate) fn chatgpt_package_version(package_full_name: &str) -> Option<String> {
    let version = package_full_name
        .strip_prefix(CHATGPT_PACKAGE_FAMILY_PREFIX)?
        .split('_')
        .next()?;
    let components = version
        .split('.')
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    (components.len() == 4).then(|| version.to_string())
}

#[cfg(windows)]
pub(crate) fn chatgpt_offline_install_version(
    display_name: &str,
    publisher: &str,
    display_version: &str,
) -> Option<String> {
    (display_name.starts_with(CHATGPT_UNINSTALL_NAME_PREFIX)
        && publisher.eq_ignore_ascii_case(CHATGPT_UNINSTALL_PUBLISHER))
    .then(|| display_version.to_string())
}

#[cfg(windows)]
pub(crate) fn newest_version(versions: impl IntoIterator<Item = String>) -> Option<String> {
    versions
        .into_iter()
        .max_by(|left, right| (version_rank(left), left).cmp(&(version_rank(right), right)))
}

#[cfg(windows)]
fn version_rank(version: &str) -> Vec<u32> {
    version
        .split('.')
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()
        .ok()
        .unwrap_or_default()
}

#[cfg(windows)]
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
fn string_from_wide(value: &[u16]) -> Option<String> {
    let end = value.iter().position(|character| *character == 0)?;
    String::from_utf16(&value[..end]).ok()
}

#[cfg(windows)]
fn subkey_names(hive: HKEY, path: &str) -> Vec<String> {
    let path = wide(path);
    let mut key = HKEY::default();
    if unsafe {
        RegOpenKeyExW(
            hive,
            PCWSTR::from_raw(path.as_ptr()),
            None,
            KEY_READ,
            &mut key,
        )
    } != WIN32_ERROR(0)
    {
        return Vec::new();
    }
    let mut names = Vec::new();
    for index in 0.. {
        let mut name = vec![0u16; 256];
        match unsafe { RegEnumKeyW(key, index, Some(&mut name)) } {
            ERROR_NO_MORE_ITEMS => break,
            WIN32_ERROR(0) => {
                if let Some(name) = string_from_wide(&name) {
                    names.push(name);
                }
            }
            _ => break,
        }
    }
    let _ = unsafe { RegCloseKey(key) };
    names
}

#[cfg(windows)]
fn registry_string(hive: HKEY, key_path: &str, value_name: &str) -> Option<String> {
    let key_path = wide(key_path);
    let value_name = wide(value_name);
    let mut byte_count = 0;
    let status = unsafe {
        RegGetValueW(
            hive,
            PCWSTR::from_raw(key_path.as_ptr()),
            PCWSTR::from_raw(value_name.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            None,
            Some(&mut byte_count),
        )
    };
    if status != WIN32_ERROR(0) {
        return None;
    }
    let mut value = vec![0u16; byte_count as usize / std::mem::size_of::<u16>()];
    let status = unsafe {
        RegGetValueW(
            hive,
            PCWSTR::from_raw(key_path.as_ptr()),
            PCWSTR::from_raw(value_name.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            Some(value.as_mut_ptr().cast()),
            Some(&mut byte_count),
        )
    };
    if status != WIN32_ERROR(0) {
        return None;
    }
    string_from_wide(&value)
}

#[cfg(windows)]
fn command_extensions() -> Vec<String> {
    let extensions = env::var("PATHEXT")
        .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string())
        .split(';')
        .map(str::trim)
        .filter(|extension| !extension.is_empty())
        .map(|extension| {
            if extension.starts_with('.') {
                extension.to_string()
            } else {
                format!(".{extension}")
            }
        })
        .collect::<Vec<_>>();
    if extensions.is_empty() {
        vec![
            ".COM".to_string(),
            ".EXE".to_string(),
            ".BAT".to_string(),
            ".CMD".to_string(),
        ]
    } else {
        extensions
    }
}

#[cfg(not(windows))]
fn command_extensions() -> Vec<String> {
    vec![String::new()]
}

pub(crate) fn command_path_in(
    command: &str,
    paths: &[PathBuf],
    extensions: &[String],
) -> Option<PathBuf> {
    paths.iter().find_map(|path| {
        extensions
            .iter()
            .map(|extension| path.join(format!("{command}{extension}")))
            .find(|path| path.is_file())
    })
}

pub(crate) fn command_client_version(command_path: &Path) -> Option<String> {
    let mut command = Command::new(command_path);
    command
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW.0);
    let mut child = command.spawn().ok()?;
    let deadline = Instant::now() + VERSION_COMMAND_TIMEOUT;
    let status = loop {
        match child.try_wait().ok()? {
            Some(status) => break status,
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            None => thread::sleep(Duration::from_millis(10)),
        }
    };
    if !status.success() {
        return None;
    }
    let mut output = String::new();
    child.stdout.take()?.read_to_string(&mut output).ok()?;
    command_version_from_output(&output)
}

pub(crate) fn command_version_from_output(output: &str) -> Option<String> {
    output.split_whitespace().find_map(|word| {
        let version =
            word.trim_matches(|character: char| !character.is_ascii_digit() && character != '.');
        let segments = version.split('.').collect::<Vec<_>>();
        (segments.len() == 3
            && segments.iter().all(|segment| {
                !segment.is_empty() && segment.chars().all(|character| character.is_ascii_digit())
            }))
        .then(|| version.to_string())
    })
}
