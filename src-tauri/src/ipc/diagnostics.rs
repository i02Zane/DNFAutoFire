use crate::app::state::AppState;
use crate::domain::classes::{class_id_by_detection_index, class_name_by_id};
use serde::Serialize;
use tauri::State;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct RuntimeDiagnostics {
    assistant: crate::runtime::AssistantRuntimeSnapshot,
    foreground: ForegroundDiagnostics,
    active_config: ActiveConfigDiagnostics,
    autofire: crate::engines::autofire::AutoFireSnapshot,
    combo: crate::engines::combo::ComboSnapshot,
    auto_run: AutoRunDiagnostics,
    detection: DetectionDiagnostics,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct ForegroundDiagnostics {
    target_active: bool,
    class_name: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct ActiveConfigDiagnostics {
    active_class_id: Option<String>,
    detection_enabled: bool,
    #[ts(type = "number")]
    detection_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct AutoRunDiagnostics {
    enabled: bool,
    running: bool,
    left_vk: u16,
    right_vk: u16,
    #[ts(type = "number")]
    pulse_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct DetectionDiagnostics {
    running: bool,
    #[ts(type = "number")]
    interval_ms: u64,
    last_result: Option<DetectionResultDiagnostics>,
    town_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct DetectionResultDiagnostics {
    class_index: Option<u16>,
    class_id: Option<String>,
    class_name: Option<String>,
    confidence: f32,
    reason: String,
}

#[tauri::command]
pub(crate) fn load_runtime_diagnostics(state: State<AppState>) -> RuntimeDiagnostics {
    let settings = state.config_store.settings();
    let profiles = state.config_store.profiles();
    let assistant_snapshots = state.assistant_runtime.engine_snapshots();
    RuntimeDiagnostics {
        assistant: assistant_snapshots.assistant,
        foreground: foreground_diagnostics(),
        active_config: ActiveConfigDiagnostics {
            active_class_id: profiles.active_class_id,
            detection_enabled: settings.detection.enabled,
            detection_interval_ms: settings.detection.interval_ms,
        },
        autofire: assistant_snapshots.autofire,
        combo: assistant_snapshots.combo,
        auto_run: AutoRunDiagnostics {
            enabled: profiles.auto_run.enabled,
            running: assistant_snapshots.auto_run.running,
            left_vk: assistant_snapshots.auto_run.left_vk,
            right_vk: assistant_snapshots.auto_run.right_vk,
            pulse_delay_ms: assistant_snapshots.auto_run.pulse_delay_ms,
        },
        detection: detection_diagnostics(state.detection_runtime.lock().snapshot()),
    }
}

fn detection_diagnostics(
    snapshot: crate::vision::detection::DetectionSnapshot,
) -> DetectionDiagnostics {
    DetectionDiagnostics {
        running: snapshot.running,
        interval_ms: snapshot.interval_ms,
        last_result: snapshot.last_result.map(detection_result_diagnostics),
        town_active: snapshot.town_active,
    }
}

fn detection_result_diagnostics(
    result: crate::vision::detection::ClassDetectionResultEvent,
) -> DetectionResultDiagnostics {
    let class_id = result
        .class_index
        .and_then(class_id_by_detection_index)
        .map(str::to_string);
    let class_name = class_id
        .as_deref()
        .and_then(class_name_by_id)
        .map(str::to_string);
    DetectionResultDiagnostics {
        class_index: result.class_index,
        class_id,
        class_name,
        confidence: result.confidence,
        reason: result.reason,
    }
}

fn foreground_diagnostics() -> ForegroundDiagnostics {
    #[cfg(windows)]
    {
        ForegroundDiagnostics {
            target_active: crate::platform::window::is_foreground_target_window_active(),
            class_name: crate::platform::window::get_foreground_window_class_name(),
        }
    }

    #[cfg(not(windows))]
    {
        ForegroundDiagnostics {
            target_active: false,
            class_name: String::new(),
        }
    }
}
