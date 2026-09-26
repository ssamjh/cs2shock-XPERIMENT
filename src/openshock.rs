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

    let body = build_request(
        &config.shocker_ids,
        control_type,
        intensity.clamp(0, 100),
        duration_ms,
    )?;

    let url = format!(
        "{}/2/shockers/control",
        config.api_server.trim_end_matches('/')
    );

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

fn build_request(
    shocker_ids: &[String],
    control_type: &str,
    intensity: i32,
    duration: i32,
) -> Result<OpenShockRequest, String> {
    let mut seen = std::collections::HashSet::new();
    let shocks: Vec<_> = shocker_ids
        .iter()
        .map(|id| id.trim())
        .filter(|id| !id.is_empty() && seen.insert(*id))
        .map(|id| Control {
            id: id.to_owned(),
            control_type: control_type.to_owned(),
            intensity,
            duration,
        })
        .collect();

    if shocks.is_empty() {
        return Err("No shockers are selected".to_string());
    }

    Ok(OpenShockRequest { shocks })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredShocker {
    pub id: String,
    pub name: String,
}

/// Fetches shockers owned by the user and shockers shared with the user.
pub async fn discover_shockers(
    api_server: &str,
    api_token: &str,
) -> Result<Vec<DiscoveredShocker>, String> {
    let client = reqwest::Client::new();
    let mut discovered = Vec::new();

    for endpoint in ["/1/shockers/own", "/1/shockers/shared"] {
        let url = format!("{}{}", api_server.trim_end_matches('/'), endpoint);
        let response = client
            .get(url)
            .header("Open-Shock-Token", api_token)
            .header("User-Agent", "CS2Shock-XPERIMENT/1.1.1")
            .send()
            .await
            .map_err(|error| error.to_string())?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let details = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!(
                "Failed to discover shockers from {}: {} - {}",
                endpoint, status, details
            ));
        }

        let response: serde_json::Value = response
            .json()
            .await
            .map_err(|error| format!("Invalid response from {}: {}", endpoint, error))?;

        let data = response.get("data").unwrap_or(&response);
        collect_shockers(data, &mut discovered);
    }

    let mut seen = std::collections::HashSet::new();
    discovered.retain(|shocker| seen.insert(shocker.id.clone()));
    Ok(discovered)
}

fn collect_shockers(value: &serde_json::Value, output: &mut Vec<DiscoveredShocker>) {
    match value {
        serde_json::Value::Object(object) => {
            if let Some(serde_json::Value::Array(shockers)) = object.get("shockers") {
                for shocker in shockers {
                    let Some(id) = shocker.get("id").and_then(serde_json::Value::as_str) else {
                        continue;
                    };
                    if id.trim().is_empty() {
                        continue;
                    }
                    let name = shocker
                        .get("name")
                        .and_then(serde_json::Value::as_str)
                        .filter(|name| !name.trim().is_empty())
                        .unwrap_or(id);
                    output.push(DiscoveredShocker {
                        id: id.to_owned(),
                        name: name.to_owned(),
                    });
                }
            }
            for child in object.values() {
                collect_shockers(child, output);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_shockers(item, output);
            }
        }
        _ => {}
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

#[cfg(test)]
mod tests {
    use super::{build_request, collect_shockers, DiscoveredShocker};

    #[test]
    fn creates_a_control_for_each_selected_shocker() {
        let request =
            build_request(&["one".to_string(), "two".to_string()], "Shock", 40, 1000).unwrap();
        let json = serde_json::to_value(request).unwrap();
        assert_eq!(json["shocks"].as_array().unwrap().len(), 2);
        assert_eq!(json["shocks"][0]["id"], "one");
        assert_eq!(json["shocks"][1]["id"], "two");
    }

    #[test]
    fn rejects_empty_selection() {
        assert!(build_request(&[], "Shock", 40, 1000).is_err());
        assert!(build_request(&[" ".to_string()], "Shock", 40, 1000).is_err());
    }

    #[test]
    fn discovers_owned_and_shared_shockers_from_api_shapes() {
        let owned_response = serde_json::json!({
            "data": [{
                "id": "device-id",
                "name": "My Hub",
                "shockers": [{ "id": "own-id", "name": "My Collar" }]
            }]
        });
        let shared_response = serde_json::json!({
            "data": [[{
                "id": "owner-id",
                "name": "Owner",
                "devices": [{
                    "id": "shared-device-id",
                    "name": "Shared Hub",
                    "shockers": [{ "id": "shared-id", "name": "Shared Collar" }]
                }]
            }]]
        });
        let mut found = Vec::<DiscoveredShocker>::new();
        collect_shockers(owned_response.get("data").unwrap(), &mut found);
        collect_shockers(shared_response.get("data").unwrap(), &mut found);
        assert_eq!(found[0].id, "own-id");
        assert_eq!(found[0].name, "My Collar");
        assert_eq!(found[1].id, "shared-id");
        assert_eq!(found[1].name, "Shared Collar");
    }

    #[test]
    fn outgoing_controls_ignore_duplicates() {
        let request =
            build_request(&["one".to_string(), " one ".to_string()], "Shock", 40, 1000).unwrap();
        assert_eq!(request.shocks.len(), 1);
    }
}
