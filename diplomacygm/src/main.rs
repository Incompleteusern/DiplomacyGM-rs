use std::{collections::HashSet, env};

use bot::bot::run_bot;
use diplomacy::persistence::player::Player;

mod bot;
mod diplomacy;


#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    // TODO thread communicated just for emojis?

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let player = Player::new(String::from("e"), String::from("e"), 0, 0, HashSet::new(), HashSet::new());

    run_bot(token).await;
}