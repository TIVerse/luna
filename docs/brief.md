# 🌙 LUNA – Practical Project Brief
## Your Personal Voice-Controlled Assistant

---

## 📋 What LUNA Actually Does

**LUNA** is a voice-activated assistant that listens for your commands and helps with everyday tasks. Say "Hey LUNA" or "Okay LUNA", then tell it what to do - it understands natural language and executes complex multi-step actions automatically.

**Example Interactions:**
- "Luna, open VS Code and the Luna project" → Opens VS Code, finds your project folder, opens it
- "Luna, play my workout playlist on Spotify" → Launches Spotify, finds and plays the playlist
- "Luna, close all Chrome tabs except YouTube" → Closes specific browser tabs
- "Luna, search my documents for the budget report" → Searches files, shows results
- "Luna, remind me about the meeting in 30 minutes" → Sets a reminder with notification
- "Luna, what's my CPU usage?" → Shows system stats and speaks them back

---

## 🎯 Core Philosophy

**Simple but Smart:**
- Always listening (low power mode)
- Understands natural commands
- Executes complex multi-step tasks
- No cloud required (privacy-first)
- Works offline completely
- Fast responses (under 1 second)

---

## 🏗️ System Architecture (Simplified)

```
┌─────────────────────────────────────────────────────────┐
│                     LUNA ASSISTANT                       │
└─────────────────────────────────────────────────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
   ┌────────┐         ┌────────┐        ┌────────┐
   │ Listen │         │ Think  │        │ Action │
   │        │────────▶│        │───────▶│        │
   │ (Ears) │         │(Brain) │        │(Hands) │
   └────────┘         └────────┘        └────────┘
        │                  │                  │
        ▼                  ▼                  ▼
   • Wake word        • Understand       • Launch apps
   • Speech to text   • Find things      • Control system
   • Voice input      • Plan steps       • Execute tasks
                      • Smart context    • Speak results
```

---

## 🔧 Technology Stack

### Core Components

**1. Audio Input (Always Listening)**
```toml
cpal = "0.15"              # Microphone access
whisper-rs = "0.11"        # Speech recognition (Whisper AI)
porcupine = "2.2"          # Wake word detection ("Hey Luna")
```

**2. Understanding (Brain)**
```toml
regex = "1.10"             # Pattern matching for commands
rust-bert = "0.21"         # Advanced language understanding
mistral-rs = "0.1"         # Local AI model (for complex queries)
```

**3. Action Execution (Hands)**
```toml
# Windows
windows = "0.52"           # Windows APIs
winapi = "0.3"             # System control

# Linux  
nix = "0.27"               # Linux system APIs
dbus = "0.9"               # Desktop integration

# macOS
core-foundation = "0.9"    # macOS frameworks
```

**4. Voice Output (Mouth)**
```toml
tts = "0.26"               # Text-to-speech (speaks back to you)
```

**5. Async Runtime**
```toml
tokio = { version = "1.35", features = ["full"] }
```

---

## 📦 Project Structure

```
luna/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs                    # Entry point
    ├── audio/
    │   ├── mod.rs
    │   ├── capture.rs             # Microphone input
    │   ├── wake_word.rs           # "Hey Luna" detection
    │   └── speech_to_text.rs      # Whisper integration
    │
    ├── brain/
    │   ├── mod.rs
    │   ├── command_parser.rs      # Understand what you want
    │   ├── intent_classifier.rs   # Categorize commands
    │   └── task_planner.rs        # Break down complex tasks
    │
    ├── actions/
    │   ├── mod.rs
    │   ├── app_launcher.rs        # Open applications
    │   ├── file_search.rs         # Find files/folders
    │   ├── window_control.rs      # Manage windows
    │   ├── system_control.rs      # Volume, power, etc.
    │   └── executor.rs            # Execute multi-step tasks
    │
    ├── os/
    │   ├── mod.rs
    │   ├── windows.rs             # Windows-specific code
    │   ├── linux.rs               # Linux-specific code
    │   └── discovery.rs           # Find installed apps
    │
    ├── tts/
    │   ├── mod.rs
    │   └── synthesizer.rs         # Voice responses
    │
    ├── context/
    │   ├── mod.rs
    │   └── memory.rs              # Remember conversation
    │
    └── config.rs                  # Settings
```

---

## 🚀 How It Works (Step by Step)

### Example: "Luna, open VS Code and the Luna project"

**Step 1: Wake Word Detection**
```
[Microphone] ──┐
               │
               ├──▶ [Always listening buffer]
               │
               └──▶ [Porcupine] ──▶ Detects "Luna" ✓
                                     └──▶ Start recording command
```

**Step 2: Speech Recognition**
```
[Recording: "open VS Code and the Luna project"]
               │
               ▼
         [Whisper AI]
               │
               ▼
Text: "open vs code and the luna project"
```

**Step 3: Understanding**
```
Text: "open vs code and the luna project"
               │
               ▼
      [Command Parser]
               │
               ├──▶ Intent: "OPEN_PROJECT"
               ├──▶ Application: "VS Code"
               ├──▶ Project: "luna"
               └──▶ Steps:
                    1. Find VS Code executable
                    2. Search for "luna" project folder
                    3. Launch VS Code with project path
```

**Step 4: Execution**
```
[Task Executor]
    │
    ├──▶ Step 1: Search installed applications
    │    └──▶ Found: C:\Users\...\Microsoft VS Code\Code.exe
    │
    ├──▶ Step 2: Search common project locations
    │    ├──▶ ~/Documents/ (not found)
    │    ├──▶ ~/Projects/ (not found)
    │    └──▶ ~/dev/luna/ (FOUND!)
    │
    └──▶ Step 3: Execute command
         └──▶ Process::Command()
              ├─ Program: "Code.exe"
              └─ Args: ["~/dev/luna/"]
         
         ✓ VS Code opens with Luna project
```

**Step 5: Response**
```
[TTS] ──▶ "Opening VS Code with the Luna project"
```

---

## 🎯 Core Features (What It Can Do)

### 1. Application Control
```
Commands:
- "open [app name]"
- "close [app name]"
- "launch [app] with [file]"
- "switch to [app]"
- "minimize/maximize all windows"

Example:
"Luna, open Chrome and go to GitHub"
→ Launches Chrome
→ Opens GitHub.com
```

### 2. File & Folder Management
```
Commands:
- "find [filename]"
- "open [folder name]"
- "search documents for [keyword]"
- "create folder [name]"
- "move [file] to [folder]"

Example:
"Luna, search my documents for the budget report"
→ Searches all documents
→ Finds matching files
→ Opens file explorer with results
```

### 3. System Control
```
Commands:
- "volume [up/down/mute]"
- "brightness [up/down]"
- "lock computer"
- "sleep/shutdown/restart"
- "what's my [CPU/memory/disk] usage?"

Example:
"Luna, what's my CPU usage?"
→ Reads system stats
→ Speaks: "CPU is at 45%, memory at 60%"
```

### 4. Window Management
```
Commands:
- "move [app] to [monitor]"
- "arrange windows side by side"
- "full screen [app]"
- "close all [app] windows"

Example:
"Luna, move Chrome to my second monitor"
→ Detects which window is Chrome
→ Moves it to monitor 2
```

### 5. Smart Reminders & Time
```
Commands:
- "remind me [about] in [time]"
- "set timer for [duration]"
- "what time is it?"
- "what's today's date?"

Example:
"Luna, remind me about the meeting in 30 minutes"
→ Sets timer
→ Shows notification after 30 min
→ Speaks reminder
```

### 6. Clipboard & Text
```
Commands:
- "copy this"
- "paste"
- "read my clipboard"
- "take a note: [text]"

Example:
"Luna, take a note: buy milk tomorrow"
→ Creates note file
→ Saves content
```

### 7. Media Control
```
Commands:
- "play/pause music"
- "next/previous track"
- "play [playlist/song]"
- "volume [level]"

Example:
"Luna, play my workout playlist"
→ Opens default music player
→ Finds playlist by name
→ Starts playback
```

### 8. Web & Search
```
Commands:
- "search for [query]"
- "open [website]"
- "search my files for [term]"

Example:
"Luna, search for Rust tutorials"
→ Opens browser
→ Searches Google/DuckDuckGo
```

---

## 🔍 Application Discovery System

### How LUNA Finds Your Applications

#### Windows Detection
```rust
// Search locations:
1. Start Menu
   - C:\ProgramData\Microsoft\Windows\Start Menu
   - C:\Users\{user}\AppData\Roaming\Microsoft\Windows\Start Menu

2. Registry
   - HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall
   - HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall
   
3. Common Paths
   - C:\Program Files
   - C:\Program Files (x86)
   - C:\Users\{user}\AppData\Local\Programs

4. UWP Apps
   - Windows Store apps (Get-AppxPackage)
```

**Example Database:**
```rust
struct Application {
    name: String,           // "Visual Studio Code"
    executable: PathBuf,    // "C:\...\Code.exe"
    aliases: Vec<String>,   // ["vscode", "code", "vs code"]
    icon: Option<PathBuf>,
    category: AppCategory,  // IDE, Browser, Media, etc.
}
```

#### Linux Detection
```rust
// Search locations:
1. Desktop Files
   - /usr/share/applications/*.desktop
   - /usr/local/share/applications/*.desktop
   - ~/.local/share/applications/*.desktop

2. Package Managers
   - dpkg -l (Debian/Ubuntu)
   - rpm -qa (Fedora/RHEL)
   - pacman -Q (Arch)

3. Snap/Flatpak
   - snap list
   - flatpak list

4. PATH binaries
   - Parse $PATH directories
```

#### macOS Detection
```rust
// Search locations:
1. Applications folder
   - /Applications
   - ~/Applications

2. LaunchServices database
   - system_profiler SPApplicationsDataType

3. Spotlight
   - mdfind "kMDItemKind == 'Application'"
```

### Smart Name Matching

```rust
// User says: "open vee ess code"
// LUNA matches:
1. Exact match: "vscode" → ✗
2. Fuzzy match: "vs code" → ✓
3. Alias match: "code" → ✓
4. Phonetic match: "vee ess code" → Visual Studio Code ✓

// Result: Launches Visual Studio Code
```

---

## 🧠 Command Understanding System

### Intent Recognition

**Simple Commands (Pattern Matching)**
```rust
// Fast path: regex patterns
"open {app}" → LAUNCH_APP
"close {app}" → CLOSE_APP  
"volume up" → VOLUME_UP
"what time is it" → GET_TIME
```

**Complex Commands (AI Understanding)**
```rust
// Uses local LLM for complex queries
"open vs code and the luna project"
→ AI breaks down:
   1. Open application (VS Code)
   2. Find project (luna)
   3. Load project in application

"close all chrome tabs except youtube"
→ AI breaks down:
   1. Get all Chrome windows
   2. Get all tabs
   3. Filter: keep only YouTube
   4. Close others
```

### Context Awareness

```rust
// Conversation memory
User: "open chrome"
Luna: [Opens Chrome]

User: "now go to github"  // "now" refers to Chrome
Luna: [Opens GitHub in Chrome]

User: "search for rust tutorials"  // assumes same context
Luna: [Searches in GitHub]
```

---

## 🎬 Real-World Usage Scenarios

### Scenario 1: Developer Workflow
```
You: "Hey Luna, start my dev environment"
Luna: "Starting development environment..."
      → Opens VS Code with your project
      → Starts Docker containers
      → Opens terminal with git status
      → Launches browser to localhost:3000
Luna: "Development environment ready"
```

### Scenario 2: Morning Routine
```
You: "Hey Luna, good morning"
Luna: "Good morning! It's 7:30 AM, Saturday."
      → Reads your calendar
Luna: "You have one meeting today at 2 PM - Team Sync"
      → Checks weather
Luna: "Weather is sunny, 72 degrees"
      → Opens news
```

### Scenario 3: Quick Actions
```
You: "Luna, take a screenshot"
Luna: → Captures screen
      → Saves to Pictures
Luna: "Screenshot saved"

You: "Luna, email that to john"
Luna: → Opens email client
      → Attaches screenshot
      → Fills recipient: john@example.com
Luna: "Email draft ready, should I send it?"
```

### Scenario 4: System Maintenance
```
You: "Luna, what's slowing down my computer?"
Luna: "Analyzing system..."
      → Checks CPU usage
      → Checks memory usage
      → Identifies heavy processes
Luna: "Chrome is using 60% of your memory with 40 tabs open.
       Would you like me to close some?"
```

### Scenario 5: File Organization
```
You: "Luna, organize my downloads folder"
Luna: "Organizing downloads..."
      → Scans files
      → Creates folders: Documents, Images, Videos, Archives
      → Moves files to appropriate folders
      → Removes duplicates
Luna: "Organized 150 files. Found and removed 12 duplicates."
```

---

## 📊 Expected Code Size

### Realistic Scope: **30,000-40,000 lines**

```
luna/
├── Audio System           5,000 lines
│   ├── Wake word          1,500
│   ├── Speech-to-text     2,000
│   └── Audio processing   1,500
│
├── Understanding          8,000 lines
│   ├── Command parser     3,000
│   ├── Intent classifier  2,500
│   └── Task planner       2,500
│
├── Actions               12,000 lines
│   ├── App launcher       2,500
│   ├── File operations    3,000
│   ├── Window control     2,000
│   ├── System control     2,000
│   └── Task executor      2,500
│
├── OS Integration         8,000 lines
│   ├── Windows APIs       3,500
│   ├── Linux APIs         3,000
│   └── App discovery      1,500
│
├── Voice Output           2,000 lines
│   └── TTS system         2,000
│
├── Context & Memory       3,000 lines
│   └── Conversation state 3,000
│
└── Core & Utils           5,000 lines
    ├── Config             1,000
    ├── Logging            1,500
    └── Error handling     2,500

Tests                      8,000 lines
Documentation              2,000 lines

TOTAL                     ~43,000 lines
```

---

## 🚀 Development Roadmap

### Phase 1: Foundation (Week 1-2)
**Goal:** Basic voice input → command execution

✅ **Deliverables:**
- Microphone capture working
- Wake word detection ("Hey Luna")
- Basic speech-to-text (Whisper)
- Simple commands (5-10):
  - "open [app]"
  - "close [app]"
  - "volume up/down"
  - "what time is it"
- TTS responses

**Milestone:** Say "Hey Luna, open Chrome" → Chrome opens

---

### Phase 2: Intelligence (Week 3-4)
**Goal:** Understand natural language

✅ **Deliverables:**
- Intent classification system
- Entity extraction (app names, file names, etc.)
- Command variations ("launch" = "open" = "start")
- Context memory (remember last 5 commands)
- 20+ commands working

**Milestone:** Say "Luna, open VS Code with my project" → Finds and opens project

---

### Phase 3: Application Discovery (Week 5-6)
**Goal:** Automatically find all installed apps

✅ **Deliverables:**
- Windows app detection (Start Menu, Registry)
- Linux app detection (.desktop files, packages)
- Application database (name, path, aliases)
- Smart fuzzy matching
- Icon extraction

**Milestone:** LUNA knows every app installed, can open any by name

---

### Phase 4: File Operations (Week 7-8)
**Goal:** Find and manage files

✅ **Deliverables:**
- File search engine (by name, content)
- Folder operations (create, delete, move)
- Project detection (git repos, specific folders)
- Recent files tracking
- Smart suggestions

**Milestone:** Say "find my budget report" → Searches and opens file

---

### Phase 5: Multi-Step Actions (Week 9-10)
**Goal:** Complex task execution

✅ **Deliverables:**
- Task planning system (break down complex commands)
- Sequential execution
- Error recovery
- Confirmation prompts for destructive actions
- 50+ total commands

**Milestone:** Say "open Chrome and search for Rust tutorials" → Does both

---

### Phase 6: Polish & Testing (Week 11-12)
**Goal:** Production ready

✅ **Deliverables:**
- Performance optimization (< 1s response)
- Error handling everywhere
- Comprehensive testing
- User documentation
- Configuration UI (optional GUI)

**Milestone:** Runs smoothly for hours without issues

---

### Phase 7: Advanced Features (Week 13-16)
**Optional enhancements:**
- Window arrangement macros
- Scheduled tasks
- Email integration
- Calendar integration
- Smart home control
- Custom voice commands

---

## 🛠️ Quick Start Development

### Setup
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/yourusername/luna.git
cd luna

# Install system dependencies

# Windows (run as Admin)
# No additional dependencies needed

# Linux
sudo apt install libasound2-dev portaudio19-dev

# macOS  
brew install portaudio

# Download Whisper model (tiny = fast, base = balanced)
mkdir -p models
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin -O models/whisper-base.bin

# Build and run
cargo build --release
cargo run --release
```

### First Commands to Implement

**Priority 1 (Week 1):**
```rust
// In src/actions/app_launcher.rs
pub async fn launch_app(app_name: &str) -> Result<()> {
    // 1. Find app in database
    // 2. Execute with Process::Command
    // 3. Return success
}

Commands:
- "open chrome"
- "open notepad"
- "open calculator"
```

**Priority 2 (Week 2):**
```rust
// In src/actions/system_control.rs
pub async fn adjust_volume(action: VolumeAction) -> Result<()> {
    // Use OS APIs to control volume
}

Commands:
- "volume up"
- "volume down"
- "mute"
```

**Priority 3 (Week 3):**
```rust
// In src/actions/file_search.rs
pub async fn find_file(query: &str) -> Result<Vec<PathBuf>> {
    // Search common locations
    // Return matching files
}

Commands:
- "find [filename]"
- "search for [query]"
```

---

## 🎯 Core Implementation: Example Code

### Main Loop
```rust
// src/main.rs
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize components
    let audio_input = AudioCapture::new()?;
    let wake_word = WakeWordDetector::new("models/porcupine.ppn")?;
    let stt = WhisperModel::load("models/whisper-base.bin")?;
    let command_handler = CommandHandler::new()?;
    let tts = TextToSpeech::new()?;
    
    println!("🌙 LUNA is listening...");
    
    loop {
        // 1. Wait for wake word
        if wake_word.detect(&audio_input).await? {
            println!("👂 Wake word detected!");
            
            // 2. Record command (3 seconds or silence)
            let audio = audio_input.record_command().await?;
            
            // 3. Convert speech to text
            let text = stt.transcribe(&audio).await?;
            println!("💬 You said: {}", text);
            
            // 4. Execute command
            match command_handler.execute(&text).await {
                Ok(response) => {
                    println!("✅ {}", response);
                    tts.speak(&response).await?;
                }
                Err(e) => {
                    println!("❌ Error: {}", e);
                    tts.speak("Sorry, I couldn't do that").await?;
                }
            }
        }
    }
}
```

### Command Handler
```rust
// src/brain/command_parser.rs
pub struct CommandHandler {
    app_db: ApplicationDatabase,
    file_index: FileIndex,
}

impl CommandHandler {
    pub async fn execute(&self, command: &str) -> Result<String> {
        // Parse intent
        let intent = self.parse_intent(command)?;
        
        match intent {
            Intent::LaunchApp { name, project } => {
                // Find application
                let app = self.app_db.find(&name)?;
                
                if let Some(proj) = project {
                    // Find project folder
                    let path = self.file_index.find_project(&proj)?;
                    // Launch with project
                    app.launch_with_args(&[path.to_str().unwrap()])?;
                    Ok(format!("Opening {} with {}", name, proj))
                } else {
                    app.launch()?;
                    Ok(format!("Opening {}", name))
                }
            }
            
            Intent::CloseApp { name } => {
                // Find and close application
                self.app_db.find(&name)?.close()?;
                Ok(format!("Closed {}", name))
            }
            
            Intent::FindFile { query } => {
                let files = self.file_index.search(&query)?;
                if files.is_empty() {
                    Ok("No files found".to_string())
                } else {
                    // Open file explorer with results
                    open::that(files[0].parent().unwrap())?;
                    Ok(format!("Found {} files", files.len()))
                }
            }
            
            Intent::SystemControl { action } => {
                // Handle volume, brightness, etc.
                system::execute(action)?;
                Ok("Done".to_string())
            }
            
            _ => Err(anyhow!("Command not understood"))
        }
    }
}
```

---

## 🔒 Privacy & Performance

**Privacy:**
- ✅ Zero cloud dependencies
- ✅ All processing local
- ✅ No data leaves your computer
- ✅ Open source (auditable)

**Performance Targets:**
- Wake word detection: < 100ms
- Speech recognition: < 500ms  
- Command execution: < 300ms
- **Total response time: < 1 second**

**Resource Usage:**
- RAM (idle): ~200MB
- RAM (active): ~500MB
- CPU (idle): < 5%
- Disk: ~2GB (with models)

---

## 🎉 Success Metrics

**Technical:**
- ✅ Recognizes 95%+ of commands correctly
- ✅ Responds in under 1 second
- ✅ Runs stable for 24+ hours
- ✅ Works offline completely

**User Experience:**
- ✅ Feels natural to use
- ✅ Understands variations ("open" = "launch" = "start")
- ✅ Helpful error messages
- ✅ Predictable behavior

**Showcase Value:**
- ✅ Impressive live demos
- ✅ Practical real-world use
- ✅ Unique differentiator (local-first)
- ✅ Technical depth (Rust + ML)

---

## 📝 Next Steps

1. **Start with basics:**
   - Get microphone input working
   - Detect wake word
   - Recognize simple speech

2. **Build core loop:**
   - Wake word → Listen → Understand → Execute → Respond

3. **Add commands incrementally:**
   - Start with 5-10 commands
   - Test thoroughly
   - Add more gradually

4. **Polish and demo:**
   - Make it reliable
   - Record demos
   - Write documentation

---

## 💡 Key Insight

**Start Simple, Iterate Fast:**
- Week 1: Get "Luna, open Chrome" working perfectly
- Week 2: Add 10 more simple commands
- Week 3: Add natural language understanding
- Week 4: Add file/project finding
- Week 8: Add complex multi-step tasks

Build a solid foundation, then expand. A working assistant with 20 commands is better than a broken one with 200.

---

**This is LUNA: Your practical, voice-controlled assistant that actually works.**

No cloud. No complexity. Just natural voice commands that execute instantly.