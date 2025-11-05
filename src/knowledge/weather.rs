//! Weather information service
//!
//! Provides real-time weather data using Open-Meteo API (free, no API key needed).

use crate::error::{LunaError, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Weather condition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WeatherCondition {
    /// Clear sky
    Clear,
    /// Partly cloudy
    PartlyCloudy,
    /// Cloudy
    Cloudy,
    /// Rainy
    Rain,
    /// Heavy rain
    HeavyRain,
    /// Snowy
    Snow,
    /// Thunderstorm
    Thunderstorm,
    /// Foggy
    Fog,
    /// Unknown condition
    Unknown,
}

impl WeatherCondition {
    /// Get emoji representation
    pub fn emoji(&self) -> &str {
        match self {
            Self::Clear => "☀️",
            Self::PartlyCloudy => "⛅",
            Self::Cloudy => "☁️",
            Self::Rain => "🌧️",
            Self::HeavyRain => "⛈️",
            Self::Snow => "❄️",
            Self::Thunderstorm => "⚡",
            Self::Fog => "🌫️",
            Self::Unknown => "❓",
        }
    }

    /// Parse from weather code (WMO Weather interpretation codes)
    pub fn from_code(code: i32) -> Self {
        match code {
            0 => Self::Clear,
            1 | 2 => Self::PartlyCloudy,
            3 => Self::Cloudy,
            45 | 48 => Self::Fog,
            51 | 53 | 55 | 61 | 63 | 80 | 81 => Self::Rain,
            65 | 82 => Self::HeavyRain,
            71 | 73 | 75 | 77 | 85 | 86 => Self::Snow,
            95 | 96 | 99 => Self::Thunderstorm,
            _ => Self::Unknown,
        }
    }
}

/// Current weather data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentWeather {
    /// Temperature in Celsius
    pub temperature: f32,
    /// Temperature in Fahrenheit
    pub temperature_f: f32,
    /// Feels like temperature in Celsius
    pub feels_like: Option<f32>,
    /// Weather condition
    pub condition: WeatherCondition,
    /// Weather description
    pub description: String,
    /// Humidity percentage
    pub humidity: Option<f32>,
    /// Wind speed in km/h
    pub wind_speed: f32,
    /// Wind direction in degrees
    pub wind_direction: Option<f32>,
    /// Precipitation in mm
    pub precipitation: Option<f32>,
    /// Cloud coverage percentage
    pub cloud_cover: Option<f32>,
    /// Location name
    pub location: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl CurrentWeather {
    /// Format as a human-readable string
    pub fn to_string(&self) -> String {
        format!(
            "{} {}°C ({}°F) in {}. {}. Wind: {} km/h",
            self.condition.emoji(),
            self.temperature,
            self.temperature_f,
            self.location,
            self.description,
            self.wind_speed
        )
    }
}

/// Weather forecast for a day
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyForecast {
    /// Date
    pub date: chrono::NaiveDate,
    /// Maximum temperature
    pub temp_max: f32,
    /// Minimum temperature
    pub temp_min: f32,
    /// Weather condition
    pub condition: WeatherCondition,
    /// Precipitation probability (0-100)
    pub precipitation_probability: Option<f32>,
    /// Precipitation amount in mm
    pub precipitation: Option<f32>,
}

/// Geographic coordinates
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coordinates {
    /// Latitude
    pub latitude: f64,
    /// Longitude
    pub longitude: f64,
}

/// Weather service client
pub struct WeatherService {
    /// HTTP client
    client: reqwest::Client,
    /// Geocoding cache
    geocoding_cache: std::sync::Arc<dashmap::DashMap<String, Coordinates>>,
}

impl WeatherService {
    /// Create a new weather service
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Luna/0.1.0")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            geocoding_cache: std::sync::Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Get current weather for a location
    pub async fn get_current_weather(&self, location: &str) -> Result<CurrentWeather> {
        info!("🌤️ Fetching weather for: {}", location);

        // Get coordinates for location
        let coords = self.geocode(location).await?;

        // Fetch weather from Open-Meteo API (free, no API key)
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m,wind_direction_10m,cloud_cover",
            coords.latitude, coords.longitude
        );

        debug!("Weather API URL: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LunaError::Network(format!("Weather request failed: {}", e)))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LunaError::Network(format!("Failed to parse weather response: {}", e)))?;

        // Parse current weather
        let current = &json["current"];
        let temp_c = current["temperature_2m"]
            .as_f64()
            .ok_or_else(|| LunaError::Parse("Missing temperature data".to_string()))? as f32;
        
        let weather_code = current["weather_code"].as_i64().unwrap_or(0) as i32;
        let condition = WeatherCondition::from_code(weather_code);

        let weather = CurrentWeather {
            temperature: temp_c,
            temperature_f: temp_c * 9.0 / 5.0 + 32.0,
            feels_like: None, // Open-Meteo free tier doesn't include this
            condition: condition.clone(),
            description: self.get_condition_description(&condition),
            humidity: current["relative_humidity_2m"].as_f64().map(|v| v as f32),
            wind_speed: current["wind_speed_10m"].as_f64().unwrap_or(0.0) as f32,
            wind_direction: current["wind_direction_10m"].as_f64().map(|v| v as f32),
            precipitation: None,
            cloud_cover: current["cloud_cover"].as_f64().map(|v| v as f32),
            location: location.to_string(),
            timestamp: chrono::Utc::now(),
        };

        info!("✅ Weather retrieved: {}", weather.to_string());
        Ok(weather)
    }

    /// Get weather forecast for the next days
    pub async fn get_forecast(&self, location: &str, days: usize) -> Result<Vec<DailyForecast>> {
        info!("📅 Fetching {}-day forecast for: {}", days, location);

        let coords = self.geocode(location).await?;

        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&daily=temperature_2m_max,temperature_2m_min,weather_code,precipitation_probability_max,precipitation_sum&timezone=auto&forecast_days={}",
            coords.latitude, coords.longitude, days.min(16)
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LunaError::Network(format!("Forecast request failed: {}", e)))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LunaError::Network(format!("Failed to parse forecast response: {}", e)))?;

        let daily = &json["daily"];
        let dates = daily["time"].as_array()
            .ok_or_else(|| LunaError::Parse("Missing forecast dates".to_string()))?;
        let temps_max = daily["temperature_2m_max"].as_array()
            .ok_or_else(|| LunaError::Parse("Missing max temperatures".to_string()))?;
        let temps_min = daily["temperature_2m_min"].as_array()
            .ok_or_else(|| LunaError::Parse("Missing min temperatures".to_string()))?;
        let codes = daily["weather_code"].as_array()
            .ok_or_else(|| LunaError::Parse("Missing weather codes".to_string()))?;

        let mut forecasts = Vec::new();
        for i in 0..dates.len().min(days) {
            if let (Some(date_str), Some(tmax), Some(tmin), Some(code)) = (
                dates[i].as_str(),
                temps_max[i].as_f64(),
                temps_min[i].as_f64(),
                codes[i].as_i64(),
            ) {
                let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                    .map_err(|e| LunaError::Parse(format!("Invalid date: {}", e)))?;

                forecasts.push(DailyForecast {
                    date,
                    temp_max: tmax as f32,
                    temp_min: tmin as f32,
                    condition: WeatherCondition::from_code(code as i32),
                    precipitation_probability: daily["precipitation_probability_max"]
                        .as_array()
                        .and_then(|arr| arr.get(i))
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32),
                    precipitation: daily["precipitation_sum"]
                        .as_array()
                        .and_then(|arr| arr.get(i))
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32),
                });
            }
        }

        info!("✅ Retrieved {} day forecast", forecasts.len());
        Ok(forecasts)
    }

    /// Geocode location name to coordinates using Open-Meteo Geocoding API
    async fn geocode(&self, location: &str) -> Result<Coordinates> {
        // Check cache first
        if let Some(coords) = self.geocoding_cache.get(location) {
            debug!("Using cached coordinates for: {}", location);
            return Ok(*coords.value());
        }

        info!("🗺️ Geocoding location: {}", location);

        let url = format!(
            "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
            urlencoding::encode(location)
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LunaError::Network(format!("Geocoding request failed: {}", e)))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LunaError::Network(format!("Failed to parse geocoding response: {}", e)))?;

        if let Some(results) = json["results"].as_array() {
            if let Some(first) = results.first() {
                let coords = Coordinates {
                    latitude: first["latitude"]
                        .as_f64()
                        .ok_or_else(|| LunaError::Parse("Missing latitude".to_string()))?,
                    longitude: first["longitude"]
                        .as_f64()
                        .ok_or_else(|| LunaError::Parse("Missing longitude".to_string()))?,
                };

                // Cache the result
                self.geocoding_cache.insert(location.to_string(), coords);

                return Ok(coords);
            }
        }

        Err(LunaError::NotFound(format!("Location not found: {}", location)))
    }

    /// Get human-readable description for weather condition
    fn get_condition_description(&self, condition: &WeatherCondition) -> String {
        match condition {
            WeatherCondition::Clear => "Clear sky",
            WeatherCondition::PartlyCloudy => "Partly cloudy",
            WeatherCondition::Cloudy => "Overcast",
            WeatherCondition::Rain => "Light rain",
            WeatherCondition::HeavyRain => "Heavy rain",
            WeatherCondition::Snow => "Snowing",
            WeatherCondition::Thunderstorm => "Thunderstorm",
            WeatherCondition::Fog => "Foggy",
            WeatherCondition::Unknown => "Unknown conditions",
        }
        .to_string()
    }
}

impl Default for WeatherService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_condition_emoji() {
        assert_eq!(WeatherCondition::Clear.emoji(), "☀️");
        assert_eq!(WeatherCondition::Rain.emoji(), "🌧️");
    }

    #[test]
    fn test_weather_condition_from_code() {
        assert_eq!(WeatherCondition::from_code(0), WeatherCondition::Clear);
        assert_eq!(WeatherCondition::from_code(61), WeatherCondition::Rain);
        assert_eq!(WeatherCondition::from_code(95), WeatherCondition::Thunderstorm);
    }

    #[tokio::test]
    async fn test_weather_service_creation() {
        let service = WeatherService::new();
        assert!(std::mem::size_of_val(&service) > 0);
    }

    // Integration test - may fail without internet
    #[tokio::test]
    #[ignore]
    async fn test_get_current_weather() {
        let service = WeatherService::new();
        let result = service.get_current_weather("London").await;
        
        if let Ok(weather) = result {
            assert!(weather.temperature > -50.0 && weather.temperature < 60.0);
            assert_eq!(weather.location, "London");
        }
    }
}
