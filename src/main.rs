use dotenvy::dotenv;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::{Value, json};
use serenity::async_trait;
use serenity::builder::GetMessages;
use serenity::model::channel::Message;
use serenity::prelude::*;
use std::env;

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

                let api_key = env::var("GROK_API_KEY").unwrap();

                let payload = json!({
                    "messages": [
                        {
                            "role": "system",
                            "content": r#"
You are Grok, a highly intelligent, helpful AI assistant. You must verify that
what the user is saying is correct. Be short, concise, and insanely funny.
Do not be afraid to use bad words. If this seems like a casual context,
such as a friend making fun of another, just give a funny response.
If there is a factual answer though actually give the real answer"#
                        },
                        {
                            "role": "user",
                            "content": format!("{previous_message_content}")
                        }
                    ],
                    "model": "grok-4-fast-reasoning",
                    "stream": false
                });

                let client = reqwest::Client::new();
                let res = client
                    .post("https://api.x.ai/v1/chat/completions")
                    .header(CONTENT_TYPE, "application/json")
                    .header(AUTHORIZATION, format!("Bearer {}", api_key))
                    .json(&payload)
                    .send()
                    .await
                    .unwrap();

                let response: Value = res.json().await.unwrap();
                let response_str = &response["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or_default();

                if let Err(error) = msg
                    .channel_id
                    .say(&ctx.http, response_str.to_string())
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
