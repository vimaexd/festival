use serde::{Serialize, Deserialize};
use serde_json::Value;

use super::auth::Account;
use crate::constants::CALENDAR_ENDPOINT;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
  pub active_since: String,
  pub active_until: String,
  pub event_type: String
}

pub async fn get_calendar(a: &Account) -> Result<Vec<CalendarEvent>, String> {
  let cl = reqwest::Client::new();
  let _resp = cl.get(CALENDAR_ENDPOINT)
    .bearer_auth(&a.access_token)
    .send()
    .await;

  let resp_raw = match _resp {
    Ok(r) => r.error_for_status().unwrap().text().await.expect("error serializing top-level response"),
    Err(err) => return Err(format!("error making request: {}", err))
  };

  let resp: Value = serde_json::from_str(&resp_raw).expect("error deserializing json");
  
  let active_events = resp
    .get("channels").unwrap()
    .get("client-events").unwrap()
    .get("states").unwrap()
    .get(0).unwrap()
    .get("activeEvents").unwrap();

  let serialized = serde_json::from_value::<Vec<CalendarEvent>>(active_events.to_owned()).unwrap();
  return Ok(serialized);
}

pub async fn get_pilgrim_songs(a: &Account) -> Result<Vec<String>, String> {
  let calendar = get_calendar(&a).await;

  let events = calendar.unwrap();

  let song_ids: Vec<String> = events.iter()
  .filter(|e| {
    e.event_type.starts_with("PilgrimSong")
  })
  .map(|e| {
    return e.event_type.split("PilgrimSong.").nth(1).unwrap().to_string();
  }).collect::<Vec<String>>();

  return Ok(song_ids);
}