# LUNA Project - Part 4: Action Execution System

## Overview
Implement the action execution layer that controls applications, files, system settings, and more.

## Dependencies
```toml
open = "5.0"          # Open files/URLs
notify-rust = "4.10"  # System notifications
clipboard = "0.5"     # Clipboard operations
```

## Modules

### 1. App Launcher (`src/actions/app_launcher.rs`)
Launch and manage applications.

```rust
pub struct AppLauncher {
    app_db: Arc<AppDatabase>,
}

impl AppLauncher {
    pub async fn launch(&self, app_name: &str) -> Result<()>;
    pub async fn launch_with_args(&self, app_name: &str, args: &[String]) -> Result<()>;
    pub async fn close(&self, app_name: &str) -> Result<()>;
}
```

### 2. File Search (`src/actions/file_search.rs`)
Search for files across the system.

```rust
pub struct FileSearch {
    index: Arc<FileIndex>,
}

impl FileSearch {
    pub async fn search_by_name(&self, query: &str) -> Result<Vec<PathBuf>>;
    pub async fn search_by_content(&self, query: &str) -> Result<Vec<PathBuf>>;
    pub async fn find_recent(&self, limit: usize) -> Result<Vec<PathBuf>>;
}
```

### 3. System Control (`src/actions/system_control.rs`)
Control system settings and power.

```rust
pub struct SystemControl;

impl SystemControl {
    pub async fn set_volume(&self, level: u8) -> Result<()>;
    pub async fn adjust_volume(&self, delta: i8) -> Result<()>;
    pub async fn set_brightness(&self, level: u8) -> Result<()>;
    pub async fn lock_computer(&self) -> Result<()>;
    pub async fn shutdown(&self) -> Result<()>;
}
```

### 4. Window Control (`src/actions/window_control.rs`)
Manage window positions and states.

```rust
pub struct WindowControl;

impl WindowControl {
    pub async fn move_to_monitor(&self, app: &str, monitor: usize) -> Result<()>;
    pub async fn maximize(&self, app: &str) -> Result<()>;
    pub async fn minimize(&self, app: &str) -> Result<()>;
}
```

### 5. Media Control (`src/actions/media_control.rs`)
Control media playback.

```rust
pub struct MediaControl;

impl MediaControl {
    pub async fn play_pause(&self) -> Result<()>;
    pub async fn next_track(&self) -> Result<()>;
    pub async fn previous_track(&self) -> Result<()>;
}
```

### 6. Task Executor (`src/actions/executor.rs`)
Execute multi-step task plans.

```rust
pub struct TaskExecutor {
    launcher: AppLauncher,
    file_search: FileSearch,
    system: SystemControl,
}

impl TaskExecutor {
    pub async fn execute_plan(&self, plan: TaskPlan) -> Result<String>;
}
```

## Success Criteria
- ✅ All action types implemented
- ✅ Error recovery
- ✅ Async execution
- ✅ User feedback
