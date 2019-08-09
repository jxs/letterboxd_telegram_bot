
mod letterboxd;

use std::time::Duration;

use futures::StreamExt;
use tokio::timer::Timeout;
use letterboxd::Api as LetterboxdApi;
use telegram_bot::{
    Api as TelegramBotApi, CanAnswerInlineQuery,
    CanReplySendMessage, InlineQueryResult, Update, UpdateKind,
};

#[tokio::main]
async fn main() {
    env_logger::init();
    let telegram_token =
        std::env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not found in environment");
    let letterboxd_key =
        std::env::var("LETTERBOXD_API_KEY").expect("LETTERBOXD_API_KEY not found in environment");
    let letterboxd_secret = std::env::var("LETTERBOXD_API_SECRET")
        .expect("LETTERBOXD_API_SECRET not found in environment");
    let telegram_api = TelegramBotApi::new(&telegram_token);

    let letterboxd_api = LetterboxdApi::new(&letterboxd_key, &letterboxd_secret);
    let mut stream = telegram_api.stream();

    log::info!("starting good reads telegram bot!");
    while let Some(update) = stream.next().await {
        match update {
            Ok(Update {
                kind: UpdateKind::InlineQuery(query),
                ..
            }) => match Timeout::new(letterboxd_api.search(&query.query), Duration::new(5, 0)).await {
                Ok(Ok(results)) => {
                    let reply = query.answer(results);
                    let res = telegram_api.send(reply).await;
                    if let Err(err) = res {
                        log::error!("telegram bot send error, {:?}", err);

                    }
                }
                Err(_elapsed) => {
                    log::error!("request to letterboxd Api timedout after 5 seconds");
                    let empty: Vec<InlineQueryResult> = vec![];
                    let reply = query.answer(empty);
                    let res = telegram_api.send(reply).await;
                    if let Err(err) = res {
                        log::error!("telegram bot send error, {:?}", err);
                    }
                }
                Ok(Err(err)) => {
                    log::error!("update error, {}", err);
                }
            },
            Ok(Update {
                kind: UpdateKind::Message(message),
                ..
            }) => {
                let res = telegram_api.send(message.text_reply(
                    "Hi, I am an inline Telegram Bot, I don't respond to commands, you can use me to search Good Reads books:
 start a message tagging me following the book you want to seach Letterboxd ex:
@lbdSBot Lord of the rings",
                )).await;
                if let Err(err) = res {
                    log::error!("telegram bot send error, {:?}", err);
                }
            }
            Err(err) => {
                log::error!("update error, {}", err);
            }
            _ => {}
        }
    }
}
