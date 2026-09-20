use std::{path::PathBuf, time::Duration};

use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::NativeState;

use super::{list_extensions, ExtensionCatalogSnapshot, ExtensionKind};

const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(30 * 60);
const CHECKED_KINDS: [ExtensionKind; 3] = [
    ExtensionKind::Rule,
    ExtensionKind::Skill,
    ExtensionKind::Mcp,
];
pub const CATALOG_EVENT: &str = "extensions:catalog";

/// 某一类扩展的目录快照推送负载，用 `kind` 让前端把快照落到对应分类。
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CatalogEvent {
    kind: ExtensionKind,
    snapshot: ExtensionCatalogSnapshot,
}

/// 扩展更新检查跟随进程，而不跟随页面：启动后立即检查一次，之后按固定间隔复查。
/// 规则、Skill、MCP 分别比对结果，只有发生变化的那一类才推送目录快照；
/// 未配置服务地址或尚未注册身份时跳过。
pub fn spawn_update_checks<R: Runtime>(app: AppHandle<R>) {
    let Some(home) = user_home() else {
        return;
    };
    tauri::async_runtime::spawn(async move {
        let mut last_pushed: [Option<String>; CHECKED_KINDS.len()] = Default::default();
        let mut interval = tokio::time::interval(UPDATE_CHECK_INTERVAL);
        loop {
            interval.tick().await;
            let state = app.state::<NativeState>();
            for (index, kind) in CHECKED_KINDS.into_iter().enumerate() {
                let Ok(snapshot) = list_extensions(&home, &state, kind).await else {
                    continue;
                };
                let Ok(payload) = serde_json::to_string(&snapshot) else {
                    continue;
                };
                if last_pushed[index].as_deref() == Some(payload.as_str()) {
                    continue;
                }
                last_pushed[index] = Some(payload);
                let _ = app.emit(CATALOG_EVENT, CatalogEvent { kind, snapshot });
            }
        }
    });
}

fn user_home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE").map(PathBuf::from)
}
