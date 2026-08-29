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
        Foundation::WIN32_ERROR,
        System::{
            Registry::{RegCloseKey, RegEnumKeyW, RegOpenKeyExW, HKEY_CURRENT_USER, KEY_READ},
            Threading::CREATE_NO_WINDOW,
        },
    },
};

const VERSION_COMMAND_TIMEOUT: Duration = Duration::from_secs(5);

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

#[cfg(windows)]
pub(crate) fn chatgpt_desktop_version() -> Option<String> {
    let key_path = wide("Software\\Classes\\ActivatableClasses\\Package");
    let mut package_key = Default::default();
    if unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR::from_raw(key_path.as_ptr()),
            None,
            KEY_READ,
            &mut package_key,
        )
    } != WIN32_ERROR(0)
    {
        return None;
    }
    let mut package_names = Vec::new();
    for index in 0.. {
        let mut name = vec![0u16; 256];
        let status = unsafe { RegEnumKeyW(package_key, index, Some(&mut name)) };
        if status == WIN32_ERROR(259) {
            break;
        }
        if status == WIN32_ERROR(0) {
            if let Some(name) = string_from_wide(&name) {
                package_names.push(name);
            }
        }
    }
    let _ = unsafe { RegCloseKey(package_key) };
    newest_chatgpt_desktop_version(package_names.iter().map(String::as_str))
}

#[cfg(not(windows))]
pub(crate) fn chatgpt_desktop_version() -> Option<String> {
    None
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
pub(crate) fn newest_chatgpt_desktop_version<'a>(
    package_names: impl IntoIterator<Item = &'a str>,
) -> Option<String> {
    package_names
        .into_iter()
        .filter_map(|package_name| {
            let version = package_name
                .strip_prefix("OpenAI.Codex_")?
                .split('_')
                .next()?;
            let components = version
                .split('.')
                .map(str::parse::<u32>)
                .collect::<Result<Vec<_>, _>>()
                .ok()?;
            (components.len() == 4).then_some((components, version))
        })
        .max_by(|left, right| left.0.cmp(&right.0))
        .map(|(_, version)| version.to_string())
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
