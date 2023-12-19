# 🎹 Fortnite Festival Mastodon Bot

This bot posts the Fortnite Festival daily song list each day to a Mastodon post.

## Prerequisites
- Rust compiler
- All the packages in the Cargo.toml file

## Running
1. Create a .env file and put your fedi instance URL + access token there. Don't worry about the Authorization token yet
2. `cargo run`, the program should prompt you to visit a URL to fetch an Epic Games authorization code. Put this code in the .env file in the blank field
3. `cargo run` once more and you should be good!

## Usage
Running the program will fetch songs, and then immediately make a post. It is designed to be triggered by a cronjob or similar scheduling system.

## Credits
🦊 Massive thanks to [InvoxiPlayGames' EricLauncher project](https://github.com/InvoxiPlayGames/EricLauncher) which served as an extremely well designed reference to the Epic Games authentication APIs.