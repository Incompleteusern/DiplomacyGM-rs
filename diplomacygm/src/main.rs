use std::env;

use bot::bot::run_bot;
use diplomacy::map_parser::vector::vector::Parser;

mod bot;
mod diplomacy;


#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    // TODO thread communicated just for emojis?

    Parser::new("impdip1.1.json".to_string()).parse();

    // Login with a bot token from the environment
    // let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // run_bot(token).await;
}