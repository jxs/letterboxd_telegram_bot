mod letterboxd;

use std::time::Duration;

use futures::StreamExt;
use telegram_bot::{
    Api as TelegramBotApi, CanAnswerInlineQuery, CanReplySendMessage, InlineQuery,
    InlineQueryResult, Update, UpdateKind,
};
use tokio::timer::Timeout;

#[tokio::main]
async fn main() {
    env_logger::init();
    let telegram_token =
        std::env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not found in environment");
    let telegram_api = TelegramBotApi::new(&telegram_token);

    let mut stream = telegram_api.stream();

    log::info!("starting good reads telegram bot!");
    while let Some(update) = stream.next().await {
        match update {
            Ok(Update {
                kind: UpdateKind::InlineQuery(query),
                ..
            }) => {
                tokio::spawn(get_query_send_response(telegram_api.clone(), query));
            }
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

async fn get_query_send_response(telegram_api: TelegramBotApi, query: InlineQuery) {
    match Timeout::new(letterboxd::search(&query.query), Duration::new(5, 0)).await {
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
            log::error!("search query error, {}", err);
        }
    }
}
