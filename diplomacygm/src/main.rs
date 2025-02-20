// TODO use std::env;

use bot::bot::run_bot;

mod bot;



// struct Handler;

// #[async_trait]
// impl EventHandler for Handler {
//     async fn message(&self, ctx: Context, msg: Message) {
//         if msg.content == "!ping" {
//             if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
//                 println!("Error sending message: {why:?}");
//             }
//         }
//     }
// }


#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    // Login with a bot token from the environment
    let token = ""; // env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    run_bot(token).await;
}