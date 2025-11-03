//! OS Integration Demo
//!
//! Demonstrates LUNA's cross-platform OS integration capabilities.
//! Run with: cargo run --example os_integration_demo

use luna::os::{OsInterface, common, discovery};
use luna::db::schema::AppCategory;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== LUNA OS Integration Demo ===\n");
    
    // 1. System Information
    println!("📊 System Information:");
    println!("   OS: {}", common::get_os_info());
    
    if let Ok(username) = common::get_username() {
        println!("   User: {}", username);
    }
    
    if let Ok(hostname) = common::get_hostname() {
        println!("   Hostname: {}", hostname);
    }
    
    if let Ok(uptime) = common::get_uptime() {
        let hours = uptime / 3600;
        let minutes = (uptime % 3600) / 60;
        println!("   Uptime: {}h {}m", hours, minutes);
    }
    
    if common::is_elevated() {
        println!("   ⚠️  Running with elevated privileges");
    }
    
    println!();
    
    // 2. OS Interface
    println!("🖥️  OS Interface:");
    let os_interface = OsInterface::new()?;
    
    // Get volume
    match os_interface.get_volume() {
        Ok(volume) => println!("   Current volume: {}%", volume),
        Err(e) => println!("   Volume control unavailable: {}", e),
    }
    
    println!();
    
    // 3. Application Discovery
    println!("🔍 Discovering installed applications...");
    
    match discovery::discover_applications().await {
        Ok(apps) => {
            println!("   Found {} applications\n", apps.len());
            
            // Count by category
            let browsers = apps.iter().filter(|a| a.category == AppCategory::Browser).count();
            let ides = apps.iter().filter(|a| a.category == AppCategory::IDE).count();
            let editors = apps.iter().filter(|a| a.category == AppCategory::TextEditor).count();
            let terminals = apps.iter().filter(|a| a.category == AppCategory::Terminal).count();
            let media = apps.iter().filter(|a| a.category == AppCategory::Media).count();
            let comm = apps.iter().filter(|a| a.category == AppCategory::Communication).count();
            let office = apps.iter().filter(|a| a.category == AppCategory::Office).count();
            let games = apps.iter().filter(|a| a.category == AppCategory::Games).count();
            let system = apps.iter().filter(|a| a.category == AppCategory::System).count();
            let other = apps.iter().filter(|a| a.category == AppCategory::Other).count();
            
            println!("📊 Applications by Category:");
            if browsers > 0 { println!("   Browser: {}", browsers); }
            if ides > 0 { println!("   IDE: {}", ides); }
            if editors > 0 { println!("   TextEditor: {}", editors); }
            if terminals > 0 { println!("   Terminal: {}", terminals); }
            if media > 0 { println!("   Media: {}", media); }
            if comm > 0 { println!("   Communication: {}", comm); }
            if office > 0 { println!("   Office: {}", office); }
            if games > 0 { println!("   Games: {}", games); }
            if system > 0 { println!("   System: {}", system); }
            if other > 0 { println!("   Other: {}", other); }
            
            println!();
            
            // Show browsers
            println!("🌐 Browsers:");
            let browsers: Vec<_> = apps.iter()
                .filter(|a| a.category == AppCategory::Browser)
                .take(5)
                .collect();
            
            for browser in browsers {
                println!("   • {} ({})", browser.name, browser.executable.display());
            }
            
            println!();
            
            // Show IDEs
            println!("💻 Development Tools:");
            let ides: Vec<_> = apps.iter()
                .filter(|a| a.category == AppCategory::IDE || 
                           a.category == AppCategory::TextEditor)
                .take(5)
                .collect();
            
            for ide in ides {
                println!("   • {} ({})", ide.name, ide.executable.display());
            }
            
            println!();
            
            // Test app matching
            println!("🔎 Testing App Search:");
            let search_terms = vec!["firefox", "chrome", "code", "terminal"];
            
            for term in search_terms {
                let matches: Vec<_> = apps.iter()
                    .filter(|a| a.matches(term))
                    .take(3)
                    .collect();
                
                if !matches.is_empty() {
                    println!("   '{}' matches:", term);
                    for app in matches {
                        println!("     - {}", app.name);
                    }
                }
            }
        }
        Err(e) => {
            println!("   ❌ Discovery failed: {}", e);
        }
    }
    
    println!();
    
    // 4. Platform-specific features
    #[cfg(target_os = "linux")]
    {
        println!("🐧 Linux-Specific Features:");
        
        use luna::os::linux;
        
        if let Ok(snaps) = linux::get_snap_packages() {
            println!("   Snap packages: {}", snaps.len());
        }
        
        if let Ok(flatpaks) = linux::get_flatpak_packages() {
            println!("   Flatpak packages: {}", flatpaks.len());
        }
        
        println!();
    }
    
    #[cfg(target_os = "windows")]
    {
        println!("🪟 Windows-Specific Features:");
        
        use luna::os::windows;
        
        if let Ok(processes) = windows::get_running_processes() {
            println!("   Running processes: {}", processes.len());
            println!("   Sample processes:");
            for process in processes.iter().take(5) {
                println!("     - {}", process);
            }
        }
        
        println!();
    }
    
    #[cfg(target_os = "macos")]
    {
        println!("🍎 macOS-Specific Features:");
        
        use luna::os::macos;
        
        if let Ok(apps) = macos::get_running_apps() {
            println!("   Running applications: {}", apps.len());
            println!("   Sample apps:");
            for app in apps.iter().take(5) {
                println!("     - {}", app);
            }
        }
        
        // Spotlight search demo
        if let Ok(results) = macos::spotlight_search("kMDItemKind == 'Application'") {
            println!("   Spotlight found {} applications", results.len());
        }
        
        println!();
    }
    
    println!("✅ Demo complete!");
    
    Ok(())
}
