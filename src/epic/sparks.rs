use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::constants::SPARKS_TRACKS_ENDPOINT;

type SparkTracksMap = HashMap<String, SparkTrack>;

#[derive(Serialize, Deserialize)]
pub struct SparkTrackIntensities {
  #[serde(skip)]
  _type: String,

  #[serde(rename = "pb")]
  pub plastic_bass: Option<u32>,

  #[serde(rename = "pd")]
  pub plastic_drums: Option<u32>,

  #[serde(rename = "pg")]
  pub plastic_guitar: Option<u32>,

  #[serde(rename = "vl")]
  pub vocals: Option<u32>,

  #[serde(rename = "gr")]
  pub guitar: Option<u32>,

  #[serde(rename = "ds")]
  pub drums: Option<u32>,

  #[serde(rename = "ba")]
  pub bass: Option<u32>
}

#[derive(Serialize, Deserialize)]
pub struct SparkTrack {
  #[serde(skip)]
  _type: String,

  #[serde(rename = "su")]
  pub uuid: String,

  #[serde(rename = "sn")]
  pub slug: String,

  #[serde(rename = "tt")]
  pub title: String, 

  #[serde(rename = "an")]
  pub artist: String, 

  #[serde(rename = "ab")]
  pub album: Option<String>,

  #[serde(rename = "ry")]
  pub release_year: usize,

  #[serde(rename = "mt")]
  pub tempo: usize,

  #[serde(rename = "mu")]
  pub midi_url: String,

  #[serde(rename = "dn")]
  pub duration: usize,

  #[serde(rename = "siv")]
  pub instrument_vocals: String,

  #[serde(rename = "sib")]
  pub instrument_bass: String,

  #[serde(rename = "sid")]
  pub instrument_drums: String,

  #[serde(rename = "sig")]
  pub instrument_guitar: String,

  #[serde(rename = "au")]
  pub art_url: String,

  #[serde(rename = "ti")]
  pub internal_id: String,

  #[serde(rename = "ld")]
  pub lipsync_url: Option<String>,
  
  #[serde(rename = "jc")]
  pub jamcode: Option<String>,

  #[serde(rename = "ge")]
  pub genres: Option<Vec<String>>,

  #[serde(rename = "mm")]
  pub mode: String,

  #[serde(rename = "mk")]
  pub key: String,

  #[serde(rename = "in")]
  pub intensities: SparkTrackIntensities,

  #[serde(rename = "qi")]
  pub quickplay: String,

  #[serde(rename = "gt")]
  pub tags: Option<Vec<String>>, // ?
}

// pub async fn get_spark_tracks() -> Result<HashMap<String, &str>, String> {
pub async fn get_spark_tracks() -> Result<SparkTracksMap, String> {
  let _resp = reqwest::get(SPARKS_TRACKS_ENDPOINT)
    .await;

  let resp_raw = match _resp {
    Ok(r) => r.text().await.expect("error serializing top-level response"),
    Err(err) => return Err(format!("error making request: {}", err))
  };

  let mut tracks: SparkTracksMap = HashMap::new();
  let resp: Value = serde_json::from_str(&resp_raw).expect("error deserializing json");
  
  for (k, v) in resp.as_object().unwrap().iter() {
    let slug = k.to_string();
    if slug.chars().nth(0).unwrap() == '_' || slug == "lastModified" {
      continue;
    }
    let serialized = serde_json::from_value::<SparkTrack>(v.get("track").unwrap().clone())
      .unwrap();

    tracks.insert(k.to_string(), serialized);
  }

  return Ok(tracks);
}