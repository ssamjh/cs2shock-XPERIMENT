use std::{fs::OpenOptions, io::Write};

use log::error;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ShockMode {
    Random,
    LastHitPercentage,
}

fn default_api_server() -> String {
    "https://api.openshock.app".to_string()
}

fn deserialize_shocker_ids<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ShockerIds {
        Many(Vec<String>),
        One(String),
    }

    Ok(match Option::<ShockerIds>::deserialize(deserializer)? {
        Some(ShockerIds::Many(ids)) => ids,
        Some(ShockerIds::One(id)) => vec![id],
        None => Vec::new(),
    })
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Config {
    pub shock_mode: ShockMode,
    pub min_duration: i32,
    pub max_duration: i32,
    pub min_intensity: i32,
    pub max_intensity: i32,
    pub beep_on_match_start: bool,
    pub beep_on_round_start: bool,
    #[serde(
        default,
        deserialize_with = "deserialize_shocker_ids",
        alias = "shocker_id"
    )]
    pub shocker_ids: Vec<String>,
    pub api_token: String,
    #[serde(default = "default_api_server")]
    pub api_server: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shock_mode: ShockMode::Random,
            min_duration: 1,
            max_duration: 1,
            min_intensity: 1,
            max_intensity: 1,
            beep_on_match_start: false,
            beep_on_round_start: false,
            shocker_ids: Vec::new(),
            api_token: String::new(),
            api_server: default_api_server(),
        }
    }
}

impl Config {
    pub fn validate(&self) -> bool {
        if self.min_duration < 1 || self.min_duration > 15 {
            error!(target: "Config", "min_duration must be between 1 and 15");
            return false;
        }

        if self.max_duration < 1 || self.max_duration > 15 {
            error!(target: "Config", "max_duration must be between 1 and 15");
            return false;
        }

        if self.min_duration > self.max_duration {
            error!(target: "Config", "min_duration must be less than or equal to max_duration");
            return false;
        }

        if self.min_intensity < 0 || self.min_intensity > 100 {
            error!(target: "Config", "min_intensity must be between 0 and 100");
            return false;
        }

        if self.max_intensity < 0 || self.max_intensity > 100 {
            error!(target: "Config", "max_intensity must be between 0 and 100");
            return false;
        }

        if self.min_intensity > self.max_intensity {
            error!(target: "Config", "min_intensity must be less than or equal to max_intensity");
            return false;
        }

        true
    }

    pub fn write_to_file(&self, path: &str) {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap_or_else(|_| panic!("Failed to open config file, {}", path));

        let json = serde_json::to_string_pretty(self).expect("Failed to serialize config");

        file.write_all(json.as_bytes())
            .expect("Failed to write config file");
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn migrates_legacy_single_shocker_id() {
        let config: Config = serde_json::from_str(
            r#"{
                "shock_mode":"Random","min_duration":1,"max_duration":1,
                "min_intensity":1,"max_intensity":1,"beep_on_match_start":false,
                "beep_on_round_start":false,"shocker_id":"legacy-id","api_token":"token"
            }"#,
        )
        .unwrap();
        assert_eq!(config.shocker_ids, ["legacy-id"]);
    }

    #[test]
    fn reads_multiple_shocker_ids() {
        let config: Config = serde_json::from_str(
            r#"{
                "shock_mode":"Random","min_duration":1,"max_duration":1,
                "min_intensity":1,"max_intensity":1,"beep_on_match_start":false,
                "beep_on_round_start":false,"shocker_ids":["one","two"],"api_token":"token"
            }"#,
        )
        .unwrap();
        assert_eq!(config.shocker_ids, ["one", "two"]);
    }
}
