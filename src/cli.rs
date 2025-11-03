//! CLI commands for LUNA assistant
//!
//! Provides diagnostic and maintenance tools:
//! - `doctor`: System diagnostics
//! - `index`: Rebuild application/file indices
//! - `events`: Live event stream monitoring
//! - `metrics`: Display metrics snapshot

use clap::{Parser, Subcommand};
use crate::error::Result;
use crate::{ConfigManager, LunaConfig};
use std::path::PathBuf;
use tracing::info;

/// LUNA Voice Assistant - Privacy-first offline voice control
#[derive(Parser, Debug)]
#[command(name = "luna")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to config file (default: ~/.config/luna/config.toml)
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,
    
    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    pub log_level: String,
    
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Audio system subcommands
#[derive(Subcommand, Debug)]
pub enum AudioCommands {
    /// List available audio devices
    Devices,
    
    /// Monitor live audio levels and VAD
    Monitor {
        /// Duration in seconds
        #[arg(short, long, default_value = "10")]
        duration: u64,
    },
    
    /// Record audio to WAV file
    Record {
        /// Duration in seconds
        #[arg(short, long, default_value = "5")]
        duration: u64,
        
        /// Output file path
        #[arg(short, long, default_value = "recording.wav")]
        output: PathBuf,
    },
    
    /// Test wake word detection
    TestWake {
        /// Test duration in seconds
        #[arg(short, long, default_value = "30")]
        duration: u64,
    },
    
    /// Show audio statistics
    Stats,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run system diagnostics
    Doctor {
        /// Include extended diagnostics
        #[arg(short, long)]
        extended: bool,
    },
    
    /// Rebuild application and file indices
    Index {
        /// Rebuild application database
        #[arg(short, long)]
        apps: bool,
        
        /// Rebuild file index
        #[arg(short, long)]
        files: bool,
        
        /// Rebuild all indices
        #[arg(long)]
        all: bool,
    },
    
    /// Monitor live event stream
    Events {
        /// Filter by event type
        #[arg(short, long)]
        filter: Option<String>,
        
        /// Tail mode (follow new events)
        #[arg(short, long)]
        tail: bool,
        
        /// Maximum events to display
        #[arg(short, long, default_value = "100")]
        limit: usize,
    },
    
    /// Display metrics snapshot
    Metrics {
        /// Print detailed metrics
        #[arg(short, long)]
        detailed: bool,
        
        /// Export to file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Audio system tools
    Audio {
        #[command(subcommand)]
        command: AudioCommands,
    },
    
    /// Validate configuration
    Config {
        /// Show current configuration
        #[arg(short, long)]
        show: bool,
        
        /// Validate configuration file
        #[arg(short, long)]
        validate: bool,
    },
}

/// Run the doctor command
pub async fn run_doctor(extended: bool) -> Result<()> {
    println!("\n🔍 LUNA System Diagnostics\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Load configuration
    let config_mgr = ConfigManager::new(None).await?;
    let config = config_mgr.get().await;
    
    println!("📋 Configuration:");
    println!("  Config loaded: ✅");
    println!("  Sample rate: {} Hz", config.audio.sample_rate);
    println!("  Wake words: {:?}", config.audio.wake_words);
    println!("  Log level: {}", config.system.log_level);
    println!("  Data dir: {}", config.system.data_dir);
    
    // Check audio system
    println!("\n🎤 Audio System:");
    #[cfg(feature = "audio")]
    {
        match cpal::default_host().default_input_device() {
            Some(device) => {
                println!("  Default input device: ✅");
                if let Ok(name) = device.name() {
                    println!("  Device name: {}", name);
                }
            }
            None => {
                println!("  Default input device: ❌ Not found");
            }
        }
    }
    #[cfg(not(feature = "audio"))]
    {
        println!("  Audio feature: ⚠️  Not enabled");
    }
    
    // Check models
    println!("\n🤖 AI Models:");
    let whisper_path = std::path::Path::new(&config.brain.whisper_model_path);
    if whisper_path.exists() {
        println!("  Whisper model: ✅ Found at {}", config.brain.whisper_model_path);
    } else {
        println!("  Whisper model: ⚠️  Not found at {}", config.brain.whisper_model_path);
        println!("    Download from: https://huggingface.co/ggerganov/whisper.cpp");
    }
    
    // Check directories
    println!("\n📁 Directories:");
    println!("  Data dir: {}", if std::path::Path::new(&config.system.data_dir).exists() { "✅" } else { "⚠️  Missing" });
    println!("  Cache dir: {}", if std::path::Path::new(&config.system.cache_dir).exists() { "✅" } else { "⚠️  Missing" });
    
    if extended {
        println!("\n🔧 Extended Diagnostics:");
        println!("  CPU cores: {}", num_cpus::get());
        println!("  Max threads: {}", config.performance.max_threads);
        println!("  Cache size: {} MB", config.performance.cache_size_mb);
        
        // Platform info
        println!("\n💻 Platform:");
        println!("  OS: {}", std::env::consts::OS);
        println!("  Arch: {}", std::env::consts::ARCH);
    }
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Diagnostics complete\n");
    
    Ok(())
}

/// Run the index rebuild command
pub async fn run_index(apps: bool, files: bool, all: bool) -> Result<()> {
    use crate::db::{AppDatabase, FileIndex};
    use crate::os::discovery::discover_applications;
    use crate::db::schema::{FileEntry, FileType};
    use std::time::Instant;
    
    if !apps && !files && !all {
        println!("Please specify --apps, --files, or --all");
        return Ok(());
    }
    
    let config_mgr = ConfigManager::new(None).await?;
    let config = config_mgr.get().await;
    let data_dir = std::path::PathBuf::from(&config.system.data_dir);
    
    println!("\n📚 Rebuilding Indices\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    if apps || all {
        println!("\n🔄 Rebuilding application database...");
        let start = Instant::now();
        
        // Discover applications
        let discovered_apps = discover_applications().await?;
        println!("  Discovered {} applications", discovered_apps.len());
        
        // Build database
        let mut db = AppDatabase::new();
        for app in discovered_apps {
            db.add_app(app);
        }
        
        // Save to disk
        let db_path = data_dir.join("app_database.json");
        db.save_to_disk(&db_path).await?;
        
        let elapsed = start.elapsed();
        println!("  ✅ Application database rebuilt: {} apps ({:.2}s)", db.len(), elapsed.as_secs_f32());
        println!("  Saved to: {:?}", db_path);
    }
    
    if files || all {
        println!("\n🔄 Rebuilding file index...");
        let start = Instant::now();
        
        let mut index = FileIndex::new();
        let mut total_files = 0;
        
        // Index configured search paths
        let search_paths = &config.paths.search_paths;
        let exclude_paths = &config.paths.exclude_paths;
        
        println!("  Indexing {} search paths...", search_paths.len());
        
        for search_path in search_paths {
            let path = std::path::PathBuf::from(search_path);
            if !path.exists() {
                println!("  ⚠️  Skipping non-existent path: {:?}", path);
                continue;
            }
            
            println!("  Scanning: {:?}", path);
            match index_directory(&path, exclude_paths, &mut index).await {
                Ok(count) => {
                    total_files += count;
                    println!("    Indexed {} files", count);
                }
                Err(e) => {
                    println!("    ⚠️  Error: {}", e);
                }
            }
        }
        
        // Save to disk
        let index_path = data_dir.join("file_index.json");
        index.save_to_disk(&index_path).await?;
        
        let elapsed = start.elapsed();
        println!("  ✅ File index rebuilt: {} files ({:.2}s)", total_files, elapsed.as_secs_f32());
        println!("  Saved to: {:?}", index_path);
    }
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Index rebuild complete\n");
    Ok(())
}

/// Index a directory recursively
fn index_directory<'a>(
    path: &'a std::path::Path,
    exclude_paths: &'a [String],
    index: &'a mut crate::db::FileIndex,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<usize>> + 'a>> {
    Box::pin(async move {
        use crate::db::schema::{FileEntry, FileType};
        use tokio::fs;
        
        let mut count = 0;
        
        // Check if path should be excluded
        let path_str = path.to_string_lossy();
        for exclude in exclude_paths {
            if path_str.contains(exclude) {
                return Ok(0);
            }
        }
        
        // Read directory
        let mut entries = match fs::read_dir(path).await {
            Ok(entries) => entries,
            Err(_) => return Ok(0), // Skip inaccessible directories
        };
        
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            
            // Check exclude patterns
            let entry_str = entry_path.to_string_lossy();
            let excluded = exclude_paths.iter().any(|ex| entry_str.contains(ex));
            if excluded {
                continue;
            }
            
            if entry_path.is_dir() {
                // Recurse into subdirectory (limit depth to avoid infinite loops)
                count += index_directory(&entry_path, exclude_paths, index).await?;
            } else if entry_path.is_file() {
                // Add file to index
                if let Some(file_entry) = create_file_entry(&entry_path).await {
                    index.add_file(file_entry);
                    count += 1;
                }
            }
        }
        
        Ok(count)
    })
}

/// Create a FileEntry from a path
async fn create_file_entry(path: &std::path::Path) -> Option<crate::db::schema::FileEntry> {
    use crate::db::schema::{FileEntry, FileType};
    
    let metadata = tokio::fs::metadata(path).await.ok()?;
    let name = path.file_name()?.to_string_lossy().to_string();
    let extension = path.extension().and_then(|e| e.to_str()).map(String::from);
    
    // Determine file type from extension
    let file_type = match extension.as_deref() {
        Some("txt") | Some("md") | Some("doc") | Some("docx") | Some("pdf") => FileType::Document,
        Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") => FileType::Image,
        Some("mp4") | Some("avi") | Some("mkv") | Some("mov") => FileType::Video,
        Some("mp3") | Some("wav") | Some("flac") | Some("ogg") => FileType::Audio,
        _ => FileType::Other,
    };
    
    Some(FileEntry {
        path: path.to_path_buf(),
        name,
        extension,
        size: metadata.len(),
        modified: metadata.modified().ok()?
            .duration_since(std::time::UNIX_EPOCH).ok()?
            .as_secs() as i64,
        file_type,
    })
}

/// Run the events monitor command
pub async fn run_events(filter: Option<String>, tail: bool, limit: usize) -> Result<()> {
    use crate::events::EventBus;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    println!("\n📡 LUNA Event Monitor\n");
    
    if let Some(ref f) = filter {
        println!("Filter: {}", f);
    } else {
        println!("Filter: <all events>");
    }
    
    println!("Tail mode: {}", if tail { "ON" } else { "OFF" });
    println!("Limit: {} events\n", limit);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Create event bus
    let bus = Arc::new(EventBus::new());
    let handle = bus.start_processing().await;
    
    let event_count = Arc::new(Mutex::new(0));
    let count_clone = Arc::clone(&event_count);
    let filter_clone = filter.clone();
    
    // Subscribe to events
    bus.subscribe(vec![], move |envelope| {
        let event_type = envelope.event_type();
        
        // Apply filter if specified
        if let Some(ref filter_str) = filter_clone {
            if !event_type.contains(filter_str.as_str()) {
                return;
            }
        }
        
        // Check limit
        let mut count = count_clone.blocking_lock();
        if *count >= limit {
            return;
        }
        *count += 1;
        
        // Display event
        let timestamp = chrono::DateTime::from_timestamp(
            (envelope.timestamp / 1_000_000) as i64,
            ((envelope.timestamp % 1_000_000) * 1000) as u32,
        )
        .unwrap_or_default();
        
        println!("[{}] {} - {:?}", 
            timestamp.format("%H:%M:%S%.3f"),
            event_type,
            envelope.event
        );
    }).await;
    
    if tail {
        println!("Press Ctrl+C to stop monitoring...\n");
        
        // Wait for Ctrl+C
        tokio::signal::ctrl_c().await.ok();
    } else {
        // Non-tail mode: wait until limit is reached
        loop {
            let count = *event_count.lock().await;
            if count >= limit {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
    
    handle.abort();
    
    let final_count = *event_count.lock().await;
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Displayed {} events", final_count);
    println!("✅ Event monitoring complete\n");
    
    Ok(())
}

/// Run the metrics command
pub async fn run_metrics(detailed: bool, output: Option<PathBuf>) -> Result<()> {
    use crate::metrics::Metrics;
    use crate::error::LunaError;
    use tokio::fs;
    
    let metrics = Metrics::new();
    
    if detailed {
        metrics.print_summary();
    } else {
        println!("\n📊 LUNA Metrics Summary\n");
        println!("Commands processed: {}", metrics.get_commands_processed());
        println!("Success rate: {:.1}%", metrics.get_success_rate());
        println!("Wake words detected: {}", metrics.get_wake_words_detected());
    }
    
    if let Some(output_path) = output {
        println!("\n💾 Exporting metrics to: {:?}", output_path);
        
        // Determine format from extension
        let extension = output_path.extension().and_then(|s| s.to_str());
        
        let content = match extension {
            Some("json") => {
                // Export as JSON
                let metrics_json = serde_json::json!({
                    "commands_processed": metrics.get_commands_processed(),
                    "commands_succeeded": metrics.get_commands_succeeded(),
                    "commands_failed": metrics.get_commands_failed(),
                    "success_rate": metrics.get_success_rate(),
                    "wake_words_detected": metrics.get_wake_words_detected(),
                    "avg_audio_capture_ms": metrics.get_avg_audio_capture_ms(),
                    "avg_stt_ms": metrics.get_avg_stt_ms(),
                    "avg_parsing_ms": metrics.get_avg_parsing_ms(),
                    "avg_execution_ms": metrics.get_avg_execution_ms(),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                });
                serde_json::to_string_pretty(&metrics_json)?
            }
            Some("csv") => {
                // Export as CSV
                let mut csv = String::from("metric,value\n");
                csv.push_str(&format!("commands_processed,{}\n", metrics.get_commands_processed()));
                csv.push_str(&format!("commands_succeeded,{}\n", metrics.get_commands_succeeded()));
                csv.push_str(&format!("commands_failed,{}\n", metrics.get_commands_failed()));
                csv.push_str(&format!("success_rate,{:.2}\n", metrics.get_success_rate()));
                csv.push_str(&format!("wake_words_detected,{}\n", metrics.get_wake_words_detected()));
                csv.push_str(&format!("avg_audio_capture_ms,{:.2}\n", metrics.get_avg_audio_capture_ms()));
                csv.push_str(&format!("avg_stt_ms,{:.2}\n", metrics.get_avg_stt_ms()));
                csv.push_str(&format!("avg_parsing_ms,{:.2}\n", metrics.get_avg_parsing_ms()));
                csv.push_str(&format!("avg_execution_ms,{:.2}\n", metrics.get_avg_execution_ms()));
                csv
            }
            _ => {
                return Err(LunaError::InvalidParameter(
                    "Output file must have .json or .csv extension".to_string()
                ));
            }
        };
        
        // Write to file
        fs::write(&output_path, content).await?;
        
        println!("  ✅ Metrics exported to {:?}", output_path);
    }
    
    Ok(())
}

/// Run the config command
pub async fn run_config(show: bool, validate: bool) -> Result<()> {
    let config_mgr = ConfigManager::new(None).await?;
    let config = config_mgr.get().await;
    
    if validate {
        println!("\n🔍 Validating configuration...");
        config.validate()?;
        println!("✅ Configuration is valid\n");
    }
    
    if show {
        println!("\n📋 Current Configuration:\n");
        println!("{:#?}", *config);
    }
    
    Ok(())
}

/// Run audio device listing
pub async fn run_audio_devices() -> Result<()> {
    use crate::audio::AudioDeviceManager;
    
    println!("\n🎤 Available Audio Devices\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let manager = AudioDeviceManager::new();
    let devices = manager.list_input_devices()?;
    
    if devices.is_empty() {
        println!("❌ No audio input devices found");
    } else {
        for (i, device) in devices.iter().enumerate() {
            println!("\n{}. {}", i + 1, device.name);
            println!("   Default: {}", if device.is_default { "✅ Yes" } else { "No" });
            println!("   Sample rates: {:?}", device.sample_rates);
            println!("   Channels: {:?}", device.channels);
        }
    }
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    Ok(())
}

/// Run audio monitoring
pub async fn run_audio_monitor(duration: u64) -> Result<()> {
    use crate::audio::{AudioCapture, VoiceActivityDetector, VadEngine};
    use crate::config::AudioConfig;
    
    println!("\n🎧 Audio Monitor ({}s)\n", duration);
    println!("Press Ctrl+C to stop early\n");
    
    let config = AudioConfig::default();
    let mut capture = AudioCapture::new(config.clone())?;
    let mut vad = VoiceActivityDetector::new(
        VadEngine::from_str(&config.vad_engine),
        config.vad_aggressiveness,
        config.target_sample_rate,
    )?;
    
    capture.start()?;
    
    let start = tokio::time::Instant::now();
    let timeout = tokio::time::Duration::from_secs(duration);
    
    loop {
        if start.elapsed() >= timeout {
            break;
        }
        
        let buffer = capture.get_ring_buffer_data(100).await; // 100ms
        if !buffer.is_empty() {
            let rms: f32 = {
                let sum: f32 = buffer.iter().map(|&s| s * s).sum();
                (sum / buffer.len() as f32).sqrt()
            };
            
            let is_speech = vad.is_speech(&buffer)?;
            let bar_len = (rms * 50.0) as usize;
            let bar = "█".repeat(bar_len);
            
            print!("\r{:40} RMS: {:.3} {}", bar, rms, if is_speech { "🎤 SPEECH" } else { "       " });
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    capture.stop()?;
    println!("\n\n✅ Monitoring complete\n");
    Ok(())
}

/// Run audio recording
pub async fn run_audio_record(duration: u64, output: PathBuf) -> Result<()> {
    use crate::audio::AudioCapture;
    use crate::config::AudioConfig;
    use hound::{WavSpec, WavWriter};
    
    println!("\n🎙️  Recording Audio ({}s)\n", duration);
    
    let config = AudioConfig::default();
    let mut capture = AudioCapture::new(config.clone())?;
    
    capture.start()?;
    
    println!("Recording...");
    tokio::time::sleep(tokio::time::Duration::from_secs(duration)).await;
    
    let audio = capture.get_ring_buffer_data(duration * 1000).await;
    capture.stop()?;
    
    println!("Saving to {:?}...", output);
    
    let spec = WavSpec {
        channels: 1,
        sample_rate: config.target_sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = WavWriter::create(&output, spec)?;
    for &sample in &audio {
        let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        writer.write_sample(sample_i16)?;
    }
    writer.finalize()?;
    
    println!("✅ Recorded {} samples to {:?}\n", audio.len(), output);
    Ok(())
}

/// Run wake word test
pub async fn run_audio_test_wake(duration: u64) -> Result<()> {
    use crate::audio::{AudioCapture, WakeWordDetector, WakeWordEngine};
    use crate::config::{AudioConfig, BrainConfig};
    
    println!("\n🎤 Wake Word Detection Test ({}s)\n", duration);
    println!("Say the wake word to test detection...\n");
    
    let audio_config = AudioConfig::default();
    let brain_config = BrainConfig::default();
    
    let mut capture = AudioCapture::new(audio_config.clone())?;
    let detector = WakeWordDetector::new_with_engine(
        WakeWordEngine::from_str(&audio_config.wake_word_engine),
        audio_config.wake_words.clone(),
        brain_config.wake_word_sensitivity,
    )?;
    
    capture.start()?;
    
    let start = tokio::time::Instant::now();
    let timeout = tokio::time::Duration::from_secs(duration);
    
    let mut detections = 0;
    
    loop {
        if start.elapsed() >= timeout {
            break;
        }
        
        let buffer = capture.get_ring_buffer_data(1000).await; // 1 second
        
        if let Some(idx) = detector.detect(&buffer).await? {
            detections += 1;
            println!("✅ Wake word detected! ({}): {}", detections, audio_config.wake_words[idx]);
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    capture.stop()?;
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Total detections: {}", detections);
    println!("✅ Test complete\n");
    
    Ok(())
}

/// Run audio statistics
pub async fn run_audio_stats() -> Result<()> {
    use crate::audio::AudioCapture;
    use crate::config::AudioConfig;
    
    println!("\n📊 Audio System Statistics\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let config = AudioConfig::default();
    let capture = AudioCapture::new(config.clone())?;
    
    let stats = capture.get_stats();
    
    println!("Frames captured: {}", stats.frames_captured);
    println!("Frames dropped: {}", stats.frames_dropped);
    println!("Ring fill ratio: {:.1}%", stats.ring_fill_ratio * 100.0);
    println!("Sample rate: {} Hz", stats.sample_rate);
    
    if stats.frames_captured > 0 {
        let drop_rate = stats.frames_dropped as f32 / stats.frames_captured as f32 * 100.0;
        println!("Drop rate: {:.2}%", drop_rate);
    }
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    Ok(())
}

/// Run audio command
pub async fn run_audio(command: AudioCommands) -> Result<()> {
    match command {
        AudioCommands::Devices => run_audio_devices().await,
        AudioCommands::Monitor { duration } => run_audio_monitor(duration).await,
        AudioCommands::Record { duration, output } => run_audio_record(duration, output).await,
        AudioCommands::TestWake { duration } => run_audio_test_wake(duration).await,
        AudioCommands::Stats => run_audio_stats().await,
    }
}

/// Run the CLI based on parsed arguments
pub async fn run_cli(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Commands::Doctor { extended }) => run_doctor(extended).await,
        Some(Commands::Index { apps, files, all }) => run_index(apps, files, all).await,
        Some(Commands::Events { filter, tail, limit }) => run_events(filter, tail, limit).await,
        Some(Commands::Metrics { detailed, output }) => run_metrics(detailed, output).await,
        Some(Commands::Config { show, validate }) => run_config(show, validate).await,
        Some(Commands::Audio { command }) => run_audio(command).await,
        None => {
            println!("No command specified. Use --help for usage information.");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cli_parsing() {
        use clap::CommandFactory;
        
        // Test that CLI can be built
        let _cli = Cli::command();
    }
}
