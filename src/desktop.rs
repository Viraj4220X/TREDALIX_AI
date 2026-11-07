use serde::{Deserialize, Serialize};

pub const DESKTOP_API_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopStatus {
    pub status: String,
    pub app_version: String,
    pub api_version: u16,
    pub transport: String,
    pub build_mode: String,
    pub capabilities: Vec<String>,
}

pub fn desktop_status() -> DesktopStatus {
    DesktopStatus {
        status: "ready".into(),
        app_version: env!("CARGO_PKG_VERSION").into(),
        api_version: DESKTOP_API_VERSION,
        transport: "tauri_ipc".into(),
        build_mode: if cfg!(debug_assertions) {
            "development"
        } else {
            "release"
        }
        .into(),
        capabilities: vec![
            "local_csv_validation".into(),
            "local_chart_indicators".into(),
            "local_backtesting".into(),
            "offline_deterministic_analyst".into(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::{DESKTOP_API_VERSION, desktop_status};

    #[test]
    fn desktop_health_reports_local_readiness_and_only_supported_capabilities() {
        let status = desktop_status();
        assert_eq!(status.status, "ready");
        assert_eq!(status.api_version, DESKTOP_API_VERSION);
        assert_eq!(status.app_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(status.transport, "tauri_ipc");
        assert!(status.capabilities.contains(&"local_csv_validation".into()));
        assert!(
            status
                .capabilities
                .contains(&"offline_deterministic_analyst".into())
        );
        assert!(!status.capabilities.iter().any(|capability| {
            capability.contains("provider")
                || capability.contains("secret")
                || capability.contains("token")
        }));
        let serialized = serde_json::to_string(&status).unwrap();
        assert!(!serialized.contains("127.0.0.1"));
        assert!(!serialized.contains("secret"));
    }
}
