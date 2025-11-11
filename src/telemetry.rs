use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelemetryConfig {
    pub enabled: bool,
    #[cfg(feature = "telemetry")]
    pub client_id: String,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default - user must opt in
            #[cfg(feature = "telemetry")]
            client_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TelemetryEvent {
    #[cfg(feature = "telemetry")]
    pub client_id: String,
    pub event_type: String,
    pub timestamp: String,
    pub metadata: serde_json::Value,
}

pub fn get_config_path() -> PathBuf {
    let home = directories::UserDirs::new()
        .expect("Failed to get user directories")
        .home_dir()
        .to_path_buf();
    home.join("localdoc").join("config.json")
}

pub fn load_config() -> TelemetryConfig {
    let config_path = get_config_path();
    
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
    }
    
    // Create default config if it doesn't exist
    let config = TelemetryConfig::default();
    let _ = save_config(&config);
    config
}

pub fn save_config(config: &TelemetryConfig) -> Result<(), std::io::Error> {
    let config_path = get_config_path();
    
    // Ensure directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let content = serde_json::to_string_pretty(config)?;
    fs::write(config_path, content)?;
    Ok(())
}

#[cfg(feature = "telemetry")]
pub fn send_telemetry_event(event_type: &str, metadata: serde_json::Value) {
    let config = load_config();
    
    if !config.enabled {
        return;
    }
    
    let event = TelemetryEvent {
        client_id: config.client_id.clone(),
        event_type: event_type.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        metadata,
    };
    
    // Spawn a background thread so we don't block the CLI
    std::thread::spawn(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
            .ok()?;
        
        // Send to Doctown API
        let _result = client
            .post("https://api.doctown.dev/telemetry")
            .json(&event)
            .send();
        
        Some(())
    });
}

#[cfg(not(feature = "telemetry"))]
pub fn send_telemetry_event(_event_type: &str, _metadata: serde_json::Value) {
    // No-op when telemetry feature is disabled
}

pub fn telemetry_status() -> String {
    #[cfg(feature = "telemetry")]
    {
        let config = load_config();
        if config.enabled {
            format!(
                "✅ Telemetry: ENABLED\n\
                 📊 Anonymous usage data is collected\n\
                 🆔 Client ID: {}\n\
                 \n\
                 Disable with: localdoc config set telemetry false",
                config.client_id
            )
        } else {
            "❌ Telemetry: DISABLED\n\
             📊 No data is collected\n\
             \n\
             Enable with: localdoc config set telemetry true"
                .to_string()
        }
    }
    
    #[cfg(not(feature = "telemetry"))]
    {
        "🚫 Telemetry: NOT COMPILED\n\
         📊 This binary was built without telemetry support\n\
         \n\
         To rebuild with telemetry: cargo build --release\n\
         To rebuild without: cargo build --release --no-default-features"
            .to_string()
    }
}
