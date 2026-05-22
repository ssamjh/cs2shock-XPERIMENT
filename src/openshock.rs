use std::sync::Arc;

use log::{debug, error, info};
use serde::Serialize;
use tokio::sync::RwLock;

use crate::config::Config;

pub async fn shock(config: Arc<RwLock<Config>>, intensity: i32, duration: i32) {
    debug!(target: "OpenShock API", "Sending shock: intensity={}, duration={}s", intensity, duration);

    let res = post(
        config,
        OpenShockOp::Shock {
            intensity,
            duration,
        },
    )
    .await;

    match res {
        Ok(_) => {
            info!(target: "OpenShock API",
                "Successfully sent shock (intensity: {}, duration: {}s)", intensity, duration
            );
        }
        Err(e) => {
            error!(target: "OpenShock API", "Failed to send shock: {}", e);
        }
    }
}

#[allow(dead_code)]
pub async fn vibrate(config: Arc<RwLock<Config>>, intensity: i32, duration: i32) {
    debug!(target: "OpenShock API",
        "Sending vibrate: intensity={}, duration={}s", intensity, duration
    );

    let res = post(
        config,
        OpenShockOp::Vibrate {
            intensity,
            duration,
        },
    )
    .await;

    match res {
        Ok(_) => {
            info!(
                target: "OpenShock API",
                "Successfully sent vibrate (intensity: {}, duration: {}s)", intensity, duration
            );
        }
        Err(e) => {
            error!(target: "OpenShock API", "Failed to send vibrate: {}", e);
        }
    }
}

pub async fn beep(config: Arc<RwLock<Config>>, duration: i32) {
    debug!(target: "OpenShock API", "Sending beep: duration={}s", duration);

    let res = post(config, OpenShockOp::Beep { duration }).await;

    match res {
        Ok(_) => {
            info!(
                target: "OpenShock API",
                "Successfully sent beep (duration: {}s)", duration
            );
        }
        Err(e) => {
            error!(target: "OpenShock API", "Failed to send beep: {}", e);
        }
    }
}

pub async fn post(config: Arc<RwLock<Config>>, op: OpenShockOp) -> Result<i32, String> {
    let config = config.read().await;

    let control_type = match op {
        OpenShockOp::Beep { .. } => "Sound",
        OpenShockOp::Vibrate { .. } => "Vibrate",
        OpenShockOp::Shock { .. } => "Shock",
    };

    let (intensity, duration) = match op {
        OpenShockOp::Beep { duration } => (0, duration),
        OpenShockOp::Vibrate {
            intensity,
            duration,
        } => (intensity, duration),
        OpenShockOp::Shock {
            intensity,
            duration,
        } => (intensity, duration),
    };

    // Clamp duration to valid range (300-65535 milliseconds)
    let duration_ms = (duration * 1000).clamp(300, 65535);

    let body = OpenShockRequest {
        shocks: vec![Control {
            id: config.shocker_id.clone(),
            control_type: control_type.to_string(),
            intensity: intensity.clamp(0, 100),
            duration: duration_ms,
        }],
    };

    let url = format!("{}/2/shockers/control", config.api_server.trim_end_matches('/'));

    let res = reqwest::Client::new()
        .post(url)
        .header("Open-Shock-Token", &config.api_token)
        .header("User-Agent", "CS2Shock-XPERIMENT/1.1.1")
        .json(&body)
        .send()
        .await;

    match res {
        Ok(res) => {
            if res.status().is_success() {
                Ok(res.status().as_u16() as i32)
            } else {
                let status = res.status().as_u16();
                let error_text = res
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(format!(
                    "Failed to post to OpenShock: {} - {}",
                    status, error_text
                ))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct OpenShockRequest {
    shocks: Vec<Control>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Control {
    id: String,
    #[serde(rename = "type")]
    control_type: String,
    intensity: i32,
    duration: i32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum OpenShockOp {
    Beep { duration: i32 },
    Vibrate { intensity: i32, duration: i32 },
    Shock { intensity: i32, duration: i32 },
}
