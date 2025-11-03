# LUNA Example Commands

## Overview
Comprehensive list of commands LUNA understands, organized by category.

## Wake Words
Say one of these to activate LUNA:
- "Luna"
- "Hey Luna"
- "OK Luna"

---

## Application Management

### Launch Applications
```
"Open Chrome"
"Launch Firefox"
"Start Visual Studio Code"
"Open Spotify"
"Launch terminal"
"Start Slack"
```

### Close Applications
```
"Close Chrome"
"Quit Firefox"
"Stop Spotify"
"Kill all Chrome windows"
```

### Switch Applications
```
"Switch to Chrome"
"Focus Firefox"
"Show me Spotify"
```

---

## System Control

### Volume
```
"Volume up"
"Volume down"
"Mute"
"Unmute"
"Set volume to 50"
"Increase volume"
"Decrease volume by 10"
```

### Brightness
```
"Brightness up"
"Brightness down"
"Set brightness to 80"
"Increase brightness"
"Dim the screen"
```

### Power Management
```
"Lock screen"
"Lock computer"
"Log out"
"Shut down"
"Restart"
"Sleep"
"Hibernate"
```

### Display
```
"Screenshot"
"Take a screenshot"
"Screen capture"
"Switch to dark mode"
"Enable night mode"
```

---

## File Operations

### Search Files
```
"Find document.pdf"
"Search for budget spreadsheet"
"Locate image.png"
"Find files named report"
"Search for .txt files"
"Find files modified today"
```

### File Actions
```
"Open budget.pdf"
"Delete old_file.txt"
"Copy document.pdf to Downloads"
"Move file to Documents"
"Rename file to new_name"
```

### Navigation
```
"Open Downloads folder"
"Show me Documents"
"Navigate to Desktop"
"List files in current directory"
```

---

## Media Control

### Music Playback
```
"Play"
"Pause"
"Stop"
"Next track"
"Previous song"
"Skip"
"Play music"
"Pause music"
```

### Video Control
```
"Play video"
"Pause video"
"Stop playback"
"Fast forward"
"Rewind"
"Next video"
```

### Playlist Management
```
"Play my favorites"
"Shuffle playlist"
"Repeat song"
"Add to playlist"
```

---

## Window Management

### Window Positioning
```
"Maximize window"
"Minimize window"
"Restore window"
"Full screen"
"Tile window left"
"Tile window right"
"Center window"
```

### Multi-monitor
```
"Move window to monitor 2"
"Show on primary display"
"Duplicate displays"
```

### Workspace
```
"Switch to workspace 2"
"Move to desktop 3"
"Show all workspaces"
```

---

## Clipboard Operations

### Copy/Paste
```
"Copy this"
"Paste"
"Copy to clipboard"
"Clear clipboard"
"Show clipboard history"
```

### Advanced
```
"Copy as plain text"
"Paste without formatting"
```

---

## Information Queries

### Time & Date
```
"What time is it?"
"What's the date?"
"What day is it?"
"Tell me the current time"
"What's today's date?"
```

### System Information
```
"How much battery?"
"What's my IP address?"
"System status"
"Disk space"
"Memory usage"
"CPU usage"
```

### Weather
```
"What's the weather?"
"Weather forecast"
"Temperature"
"Will it rain today?"
```

---

## Web & Search

### Web Search
```
"Search for Rust programming"
"Google artificial intelligence"
"Look up recipe for pasta"
"Wikipedia Linux"
```

### Open URLs
```
"Open GitHub"
"Go to Reddit"
"Navigate to google.com"
"Open YouTube"
```

---

## Reminders & Calendar

### Set Reminders
```
"Remind me to call mom at 5pm"
"Set reminder for meeting tomorrow"
"Create reminder for grocery shopping"
"Alert me in 30 minutes"
```

### View Reminders
```
"Show my reminders"
"What are my reminders?"
"List today's reminders"
"Clear all reminders"
```

### Calendar
```
"What's on my calendar?"
"Schedule meeting for tomorrow"
"Add event to calendar"
"Show today's events"
```

---

## Productivity

### Notes
```
"Take a note"
"Create note about meeting"
"Open my notes"
"Delete last note"
```

### Timer
```
"Set timer for 5 minutes"
"Start countdown"
"How much time left?"
"Cancel timer"
```

### Focus Mode
```
"Enable focus mode"
"Do not disturb"
"Disable notifications"
"Enable notifications"
```

---

## Advanced Commands

### Multi-step Commands
```
"Open Chrome and go to GitHub"
"Close all browsers and lock screen"
"Find document.pdf and open it"
"Search for budget and send to printer"
```

### Context-aware
```
# After finding a file:
"Open it"
"Delete it"
"Copy it to Desktop"

# After opening an app:
"Close it"
"Maximize it"
"Move it to monitor 2"
```

### Conditional Commands
```
"If battery is low, enable power saving"
"When timer ends, notify me"
```

---

## Configuration

### Settings
```
"Change wake word to Hey Luna"
"Set voice to female"
"Increase speech rate"
"Lower TTS volume"
"Enable continuous listening"
"Disable continuous listening"
```

### Preferences
```
"Remember this preference"
"Set default browser to Firefox"
"Change notification sound"
```

---

## Help & Status

### Getting Help
```
"Help"
"What can you do?"
"Show commands"
"Give me examples"
"How do I open apps?"
```

### Status Checks
```
"Are you listening?"
"Status check"
"System diagnostics"
"Show statistics"
"Clear cache"
```

---

## Command Patterns

### Natural Language Variations

LUNA understands multiple ways to say the same thing:

**Opening apps:**
- "Open Chrome" ✓
- "Launch Chrome" ✓
- "Start Chrome" ✓
- "Run Chrome" ✓
- "Fire up Chrome" ✓

**Volume control:**
- "Volume up" ✓
- "Increase volume" ✓
- "Make it louder" ✓
- "Turn up the volume" ✓
- "Louder" ✓

**Time queries:**
- "What time is it?" ✓
- "Current time?" ✓
- "Tell me the time" ✓
- "What's the time?" ✓
- "Time please" ✓

### Parameterized Commands

**With numbers:**
```
"Set volume to [0-100]"
"Set brightness to [0-100]"
"Set timer for [X] minutes"
"Remind me in [X] hours"
```

**With names:**
```
"Open [app_name]"
"Find [filename]"
"Search for [query]"
"Play [song/artist]"
```

**With locations:**
```
"Copy [file] to [destination]"
"Move window to [monitor/position]"
"Navigate to [folder]"
```

---

## Command Chaining

### Sequential Execution
```
"Open Chrome, then maximize it, then go to GitHub"
"Find document.pdf, open it, and print it"
"Close all apps, then lock screen"
```

### Parallel Execution
```
"Volume up and brightness up"
"Open Chrome and Firefox"
"Mute and minimize window"
```

---

## Error Recovery

### Clarification Requests

If LUNA doesn't understand, try:

**Be more specific:**
- ❌ "Open it" (without context)
- ✅ "Open Chrome"

**Use wake word:**
- ❌ "Volume up" (without wake word)
- ✅ "Luna, volume up"

**Speak clearly:**
- ❌ Mumbled speech
- ✅ Clear enunciation

**Rephrase:**
- ❌ "Do the thing with the volume"
- ✅ "Increase volume"

### Common Mistakes

**Missing parameters:**
```
❌ "Set timer" (no duration)
✅ "Set timer for 5 minutes"

❌ "Open file" (no filename)
✅ "Open document.pdf"

❌ "Go to" (no destination)
✅ "Go to Desktop"
```

**Ambiguous commands:**
```
❌ "Close" (which app?)
✅ "Close Chrome"

❌ "Open" (what?)
✅ "Open Downloads folder"
```

---

## Tips for Better Recognition

### Speaking Tips
1. **Speak naturally** - Use your normal voice
2. **Clear pronunciation** - Enunciate clearly
3. **Moderate pace** - Not too fast, not too slow
4. **Quiet environment** - Minimize background noise
5. **Wait for response** - Let LUNA finish before next command

### Command Construction
1. **Start with wake word** - "Luna, [command]"
2. **Use action verbs** - Open, close, find, set
3. **Be specific** - Include app names, filenames
4. **Keep it simple** - Short commands work best
5. **Use context** - "Open it" after finding something

### Troubleshooting
1. **Not responding?** - Check wake word detection
2. **Wrong action?** - Rephrase more clearly
3. **Error message?** - Try simpler command
4. **Low confidence?** - Speak more clearly
5. **Repeated failures?** - Check configuration

---

## Custom Commands

### Creating Custom Commands

Edit `~/.config/luna/custom_commands.yaml`:

```yaml
custom_commands:
  - trigger: "good morning"
    actions:
      - open_app: "chrome"
      - set_brightness: 80
      - play_music: "Morning Playlist"
      - say: "Good morning! Starting your day."
  
  - trigger: "work mode"
    actions:
      - close_app: "spotify"
      - open_app: "vscode"
      - open_app: "slack"
      - enable_focus_mode: true
      - say: "Work mode activated."
  
  - trigger: "end of day"
    actions:
      - close_all_apps: true
      - backup_files: true
      - lock_screen: true
```

### Macro Commands

```yaml
macros:
  - name: "setup_dev"
    description: "Open development environment"
    steps:
      - open: "vscode"
      - open: "terminal"
      - run: "cd ~/projects"
      - tile: "left"
      - focus: "vscode"
      - tile: "right"
```

---

## Language Support

### Currently Supported
- English (US)
- English (UK)

### Coming Soon
- Spanish
- French
- German
- Hindi

---

## Feedback

### Improving Recognition

**Low confidence commands are logged for improvement:**
```bash
# View recognition logs
luna logs --low-confidence

# Submit feedback
luna feedback "add command: [your command]"
```

### Report Issues

```bash
# Report command not working
luna report-issue "command 'open chrome' not working"

# Request new feature
luna feature-request "add support for [feature]"
```

---

## Quick Reference Card

### Most Common Commands
```
Wake:           "Luna"
Open App:       "Open [app]"
Close App:      "Close [app]"
Volume:         "Volume up/down"
Brightness:     "Brightness up/down"
Find File:      "Find [filename]"
Time:           "What time is it?"
Lock:           "Lock screen"
Screenshot:     "Screenshot"
Help:           "What can you do?"
```

### Emergency Commands
```
"Stop"          - Cancel current action
"Mute Luna"     - Disable TTS
"Reset"         - Clear context
"Help"          - Show help
"Quit"          - Exit LUNA
```

---

## Advanced Usage

### Voice Macros
Record and replay command sequences:
```
"Luna, record macro 'morning routine'"
[speak commands]
"Luna, stop recording"

# Later:
"Luna, run macro 'morning routine'"
```

### Automation
Trigger commands based on events:
```yaml
automations:
  - event: "battery_low"
    action: "enable_power_saving"
  
  - event: "time"
    schedule: "09:00"
    action: "say 'Time for standup meeting'"
  
  - event: "wifi_disconnected"
    action: "notify 'WiFi connection lost'"
```

---

## Best Practices

1. **Use consistent naming** for files and apps
2. **Keep commands short** and direct
3. **Leverage context** with follow-up commands
4. **Create macros** for repeated sequences
5. **Review logs** to improve recognition
6. **Train with corrections** when misunderstood
7. **Organize files** for easier search
8. **Use wake word** every time
9. **Wait for feedback** before next command
10. **Check status** if unsure

---

For more information, see:
- [Master Prompt](MASTER-PROMPT.md)
- [Architecture](architecture-diagram.md)
- [Deployment Guide](DEPLOYMENT_GUIDE.md)
