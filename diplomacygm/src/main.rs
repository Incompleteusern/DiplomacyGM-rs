use std::env;

use bot::bot::run_bot;

mod bot;
mod diplomacy;


#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    // TODO thread communicated just for emojis?

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    run_bot(token).await;
}