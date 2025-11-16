use std::env;

use dotenvy::dotenv;
use serenity::async_trait;
use serenity::builder::GetMessages;
use serenity::model::channel::Message;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let bot_id = ctx.http.get_current_user().await.unwrap().id;
        if msg.mentions_user_id(bot_id) {
            let builder = GetMessages::new().before(msg.id).limit(1);
            if let Ok(messages) = msg.channel_id.messages(&ctx.http, builder).await {
                let previous_message = &messages[0];
                let previous_message_content = previous_message.content.as_str();
                if let Err(error) = msg
                    .channel_id
                    .say(&ctx.http, previous_message_content)
                    .await
                {
                    println!("Failed: {:?}", error);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
