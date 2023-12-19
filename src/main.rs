mod epic;
mod utils;
mod constants;

use std::fs;
use dotenv::dotenv;
use colored::Colorize;

use constants::{FORT_CLIENT, FORT_SECRET, EGL_CLIENT, EGL_SECRET};
use utils::ascii_bar;
use epic::sparks::SparkTrack;
use epic::auth::{AccountPublicService, Account};
use epic::calendar;
use megalodon::megalodon::PostStatusInputOptions;

#[tokio::main]
async fn main() {
  dotenv().ok();
  println!("{}", "### fortnite festival setlist ###".bold().bright_purple());
  
  let mut account: Option<Account> = None;
  let egl = AccountPublicService::new(EGL_CLIENT, EGL_SECRET);

  // try load saved login if it exists
  let save = fs::read_to_string("account.json");
  if save.is_ok() {
    println!("found saved account!");
    let acc = Account::from_save(&egl, save.unwrap()).await;
    match acc {
      Some(s) => {
        account = Some(s);
      } 
      None => {
        println!("couldnt resolve saved account!")
      }
    }
  } else if save.is_err() {
    println!("error reading saved account! continuing anyway..")
  }

  // else lets try and fetch a fresh response using an authorization code
  if account.is_none() {
    println!("no saved login found.. trying authorization code");
    let authoriz = std::env::var("AUTHORIZATION_CODE");

    // prompt for info on how to grab the authorization code
    if authoriz.as_ref().is_err() || authoriz.as_ref().unwrap() == "" {
      println!("couldnt get authorization code from .env!");
      println!(
        "please login to epic games in a browser and paste your authorization code from this url into the .env file: {}", 
        egl.get_redirect_url()
      );
      return;
    }

    // try to login
    println!("trying login..");
    let code = authoriz.unwrap();
    let _acct = egl.login_to_account(
      "authorization_code".to_string(), 
      Some(&code)
    ).await;

    account = match _acct {
      Ok(v) => Some(v),
      Err(e) => {
        println!("error logging in!: {}", e);
        return;
      }
    };

    // serialize account
    fs::write("./account.json", serde_json::to_string(account.as_ref().unwrap()).unwrap())
      .expect("error saving account to file!");
  }

  // we've def got an account now now so we can safely unwrap
  let a = account.unwrap();
  println!("logged in as {}", a.display_name);

  // yes, there are at least 3 fucking authentication requests required for this
  let exch = egl.request_exchange_code(&a).await.unwrap();
  let fort = AccountPublicService::new(FORT_CLIENT, FORT_SECRET);
  let fortacc = fort.login_to_account("exchange_code".to_string(), Some(&exch.code))
    .await
    .expect("error getting exchange code!");

  println!("got fort account");

  let sparks_tracks = epic::sparks::get_spark_tracks().await.unwrap();

  let pilgrim_songids = calendar::get_pilgrim_songs(&fortacc)
    .await
    .unwrap();

  let pilgrim_songs: Vec<&SparkTrack> = pilgrim_songids.iter()
    .map(|p| {
      sparks_tracks.get(p).unwrap()
    })
    .collect();

  println!("current pilgrim songs: {}", pilgrim_songids.join(", "));

  // fedi stuff starts here
  let fediverse = megalodon::generator(
    megalodon::SNS::Mastodon,
    std::env::var("FEDI_INSTANCE").unwrap(),
    Some(std::env::var("FEDI_ACCESS_TOKEN").unwrap()),
    None,
  );

  let res = fediverse.verify_account_credentials().await;
  match res {
    Ok(_) => {
      println!("logged into fedi")
    }
    Err(e) => {
      println!("error verifying fedi credentials! {}", e);
      return;
    }
  }

  // format tracks into strings to be appended to the post
  let formatted_songs = pilgrim_songs.iter()
    .map(|s| {
      format!("{} - {}\nGuitar {}\nDrums {}\nVocals {}\nBass {}", 
      s.artist, s.title, 
      ascii_bar(s.intensities.guitar.unwrap() + 1, 7),
      ascii_bar(s.intensities.drums.unwrap() + 1, 7),
      ascii_bar(s.intensities.vocals.unwrap() + 1, 7),
      ascii_bar(s.intensities.bass.unwrap() + 1, 7))
    })
    .collect::<Vec<String>>()
    .join("\n\n");
    

  let post = format!("{}", formatted_songs);

  // set cw + visibility
  let post_params = PostStatusInputOptions {
    visibility: Some(megalodon::entities::StatusVisibility::Unlisted),
    spoiler_text: Some(format!("Fortnite Festival Setlist for {}", chrono::Utc::now().format("%d/%m/%Y"))),
    ..Default::default()
  };

  // post the dang thing
  match fediverse.post_status(post, Some(&post_params)).await {
    Ok(_) => { println!("posted status!")}
    Err(e) => { println!("error posting status! {e}")}
  }

}
