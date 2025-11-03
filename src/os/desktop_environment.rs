//! Desktop Environment Intelligence (GOD-LEVEL)

use crate::error::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DEType {
    Gnome(String),
    KDE(String),
    XFCE,
    MATE,
    Cinnamon,
    Windows11,
    Windows10,
    MacOS(String),
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DEFeatures {
    pub has_system_tray: bool,
    pub has_global_menu: bool,
    pub supports_transparency: bool,
    pub supports_blur: bool,
    pub notification_daemon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Compositor {
    Mutter,
    KWin,
    Xfwm,
    Compiz,
    Compton,
    Picom,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAvailability {
    pub wmctrl: bool,
    pub xdotool: bool,
    pub qdbus: bool,
    pub gnome_shell: bool,
}

pub struct DesktopEnvironment {
    pub de_type: DEType,
    pub compositor: Option<Compositor>,
    pub features: DEFeatures,
    pub tools: ToolAvailability,
}

impl DesktopEnvironment {
    pub async fn detect() -> Result<Self> {
        info!("Detecting desktop environment");

        #[cfg(target_os = "linux")]
        let de_type = Self::detect_linux_de().await;

        #[cfg(target_os = "windows")]
        let de_type = Self::detect_windows_version();

        #[cfg(target_os = "macos")]
        let de_type = Self::detect_macos_version();

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        let de_type = DEType::Other("Unknown".to_string());

        let tools = Self::detect_tools().await;
        let features = Self::detect_features(&de_type);

        Ok(Self {
            de_type,
            compositor: None,
            features,
            tools,
        })
    }

    #[cfg(target_os = "linux")]
    async fn detect_linux_de() -> DEType {
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            if desktop.contains("GNOME") {
                return DEType::Gnome("Unknown".to_string());
            } else if desktop.contains("KDE") {
                return DEType::KDE("Unknown".to_string());
            } else if desktop.contains("XFCE") {
                return DEType::XFCE;
            } else if desktop.contains("MATE") {
                return DEType::MATE;
            }
        }
        DEType::Other("Unknown".to_string())
    }

    #[cfg(target_os = "windows")]
    fn detect_windows_version() -> DEType {
        DEType::Windows11
    }

    #[cfg(target_os = "macos")]
    fn detect_macos_version() -> DEType {
        DEType::MacOS("Unknown".to_string())
    }

    async fn detect_tools() -> ToolAvailability {
        ToolAvailability {
            wmctrl: Self::check_command("wmctrl").await,
            xdotool: Self::check_command("xdotool").await,
            qdbus: Self::check_command("qdbus").await,
            gnome_shell: Self::check_command("gnome-shell").await,
        }
    }

    async fn check_command(cmd: &str) -> bool {
        std::process::Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn detect_features(de_type: &DEType) -> DEFeatures {
        match de_type {
            DEType::Gnome(_) => DEFeatures {
                has_system_tray: true,
                has_global_menu: false,
                supports_transparency: true,
                supports_blur: true,
                notification_daemon: Some("gnome-shell".to_string()),
            },
            DEType::KDE(_) => DEFeatures {
                has_system_tray: true,
                has_global_menu: true,
                supports_transparency: true,
                supports_blur: true,
                notification_daemon: Some("plasma".to_string()),
            },
            _ => DEFeatures {
                has_system_tray: false,
                has_global_menu: false,
                supports_transparency: false,
                supports_blur: false,
                notification_daemon: None,
            },
        }
    }
}
