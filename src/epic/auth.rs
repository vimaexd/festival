use reqwest;
use serde::{Deserialize, Serialize};
use chrono::DateTime;
use std::{fs, io::Error};

use super::super::constants::{
  APS_ENDPOINT,
  EGL_URL_REDIRBASE
};

/*
  💜 🦊
  thanks infinitely to InvoxiPlayGames / Emma
  for making EricLauncher, which was a helpful
  reference in Epic Games authentication
  systems.
*/

#[derive(Serialize, Deserialize)]
pub struct Account {
  pub account_id: String,
  #[serde(rename = "displayName")]
  pub display_name: String,

  pub access_token: String,
  pub expires_at: String,

  pub refresh_token: String,
  pub refresh_expires_at: String,
}

impl Account {
  pub async fn from_save(egl: &AccountPublicService, save: String) -> Option<Account> {
    let acc = serde_json::from_str::<Account>(&save.as_str())
      .expect("error deserializing saved account!");

    let now = chrono::Utc::now();
    let token_expiry = DateTime::parse_from_rfc3339(&acc.expires_at.as_str()).unwrap();
    let refresh_expiry = DateTime::parse_from_rfc3339(&acc.refresh_expires_at.as_str()).unwrap();

    // if token expired and refresh not expired then refresh
    if token_expiry < now && refresh_expiry > now {
      print!("refreshing auth..");
      let refreshed_acct = egl.login_to_account(
        "refresh_token".to_string(), 
        Some(&acc.refresh_token)
      ).await;

      match refreshed_acct {
        Ok(v) => { 
          println!("refreshed!"); 
          
          v.save_to_disk("./account.json")
            .expect("couldnt save account to disk!");
          return Some(v); 
        },
        Err(e) => println!("couldn't refresh auth! {}", e)
      }
    }

    // if we didnt just refresh, then verify if the token still works
    if token_expiry > now {
      print!("verifying saved account... ");
      let verified = egl.verify(&acc).await;
      if verified.is_err() {
        println!("couldn't verify saved account!")
      } else {
        // set account and continue
        println!("verified!");
        return Some(acc);
      }
    }

    return None
  }

  pub fn save_to_disk(&self, path: &str) -> Result<bool, Error> {
    let serialized = serde_json::to_string(&self).unwrap();
    match fs::write(path, serialized) {
      Ok(_) => return Ok(true),
      Err(e) => return Err(e)
    }
  }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeCode {
  pub code: String,
  pub creating_client_id: String,
  pub expires_in_seconds: u32
}

#[derive(Clone, Copy)]
pub struct AccountPublicService {
  pub client_id: &'static str,
  pub client_secret: &'static str
}

impl AccountPublicService {
  pub fn new(client: &'static str, secret: &'static str) -> AccountPublicService {
    AccountPublicService {
      client_id: client,
      client_secret: secret
    }
  }

  pub fn get_redirect_url(&self) -> String {
    return EGL_URL_REDIRBASE.to_string() + self.client_id;
  }

  pub async fn login_to_account(&self, grant_type: String, code: Option<&String>) -> Result<Account, reqwest::Error> {
    let mut params: Vec<(String, String)> = vec!(
      ("grant_type".to_string(), grant_type.to_string()), 
      ("token_type".to_string(), "eg1".to_string())
    );

    if code.is_some() {
      let actual_code = code.unwrap();
      if grant_type == "authorization_code" {
        params.push(("code".to_string(), actual_code.to_owned()));
      } else {
        params.push((grant_type.to_string(), actual_code.to_owned()));
      }
    }
    
    let client = reqwest::Client::new();
    let _resp = client.post(APS_ENDPOINT.to_string() + "/account/api/oauth/token")
      .form(&params)
      .basic_auth(self.client_id, Some(self.client_secret))
      .send()
      .await?;
    
    let resp = match _resp.error_for_status() {
      Ok(v) => v,
      Err(e) => return Err(e),
    };

    let acc = resp
      .json::<Account>().await
      .expect("error parsing aps response!");
    return Ok(acc);
  }

  pub async fn verify(&self, acct: &Account) -> Result<bool, reqwest::Error> {
    let client = reqwest::Client::new();
    let _resp = client.post(APS_ENDPOINT.to_string() + "/account/api/oauth/verify")
      .bearer_auth(&acct.access_token)
      .send()
      .await;

    match _resp {
      Ok(_) => return Ok(true),
      Err(e) => return Err(e),
    };
  }

  pub async fn request_exchange_code(&self, acct: &Account) -> Result<ExchangeCode, reqwest::Error> {
    let client = reqwest::Client::new();
    let _resp = client.get(APS_ENDPOINT.to_string() + "/account/api/oauth/exchange")
      .bearer_auth(&acct.access_token)
      .send()
      .await;

    let resp = match _resp {
      Ok(v) => v,
      Err(e) => return Err(e),
    };
    
    let acc = resp.error_for_status()?.json::<ExchangeCode>().await.unwrap();
    return Ok(acc);
  }
}