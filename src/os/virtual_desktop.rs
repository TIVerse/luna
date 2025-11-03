//! Virtual Desktop Management (GOD-LEVEL)

use crate::error::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VirtualDesktop {
    pub id: usize,
    pub name: String,
    pub windows: Vec<u32>,
}

pub struct VirtualDesktopManager {
    desktops: Vec<VirtualDesktop>,
    app_rules: HashMap<String, usize>,
    current_desktop: usize,
}

impl VirtualDesktopManager {
    pub fn new() -> Self {
        Self {
            desktops: vec![VirtualDesktop {
                id: 0,
                name: "Desktop 1".to_string(),
                windows: Vec::new(),
            }],
            app_rules: HashMap::new(),
            current_desktop: 0,
        }
    }

    pub async fn create_desktop(&mut self, name: String) -> Result<usize> {
        let id = self.desktops.len();
        self.desktops.push(VirtualDesktop {
            id,
            name,
            windows: Vec::new(),
        });
        Ok(id)
    }

    pub async fn switch_to(&mut self, desktop_id: usize) -> Result<()> {
        if desktop_id < self.desktops.len() {
            self.current_desktop = desktop_id;
            Ok(())
        } else {
            Err(crate::error::LunaError::SystemOperation(
                "Invalid desktop ID".to_string(),
            ))
        }
    }

    pub async fn move_window_to(&mut self, window_id: u32, desktop_id: usize) -> Result<()> {
        if desktop_id < self.desktops.len() {
            self.desktops[desktop_id].windows.push(window_id);
            Ok(())
        } else {
            Err(crate::error::LunaError::SystemOperation(
                "Invalid desktop ID".to_string(),
            ))
        }
    }

    pub fn set_app_rule(&mut self, app_name: String, desktop_id: usize) {
        self.app_rules.insert(app_name, desktop_id);
    }

    pub fn list_desktops(&self) -> &[VirtualDesktop] {
        &self.desktops
    }
}

impl Default for VirtualDesktopManager {
    fn default() -> Self {
        Self::new()
    }
}
