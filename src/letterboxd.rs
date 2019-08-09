use std::time::{SystemTime, UNIX_EPOCH};

use failure::{format_err, Error, ResultExt};
use futures::TryStreamExt;
use hmac::{Hmac, Mac};
use hyper::Client;
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use sha2::Sha256;
use telegram_bot::{InlineQueryResult, InlineQueryResultArticle, InputTextMessageContent};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct LbResults {
    items: Vec<FilmSearch>,
}

#[derive(Deserialize, Debug, Default)]
pub struct FilmSearch {
    film: Option<Film>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Film {
    id: String,
    name: String,
    poster: Option<Posters>,
    links: Vec<Link>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Posters {
    sizes: Vec<Link>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Link {
    url: String,
}

pub struct Api {
    key: String,
    secret: String,
}

impl Api {
    pub fn new(key: &str, secret: &str) -> Api {
        Api {
            key: key.to_string(),
            secret: secret.to_string(),
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<InlineQueryResult>, Error> {
        letterboxd_search(self.key.clone(), self.secret.clone(), query.to_string()).await
    }
}

pub async fn letterboxd_search(
    key: String,
    secret: String,
    query: String,
) -> Result<Vec<InlineQueryResult>, Error> {
    let https = HttpsConnector::new(1)?;
    let client = Client::builder().build::<_, hyper::Body>(https);

    let nonce = Uuid::new_v4().to_simple();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context(format!("System Time is invalid"))?
        .as_secs();

    let url = format!(
        "https://api.letterboxd.com/api/v0/search?apikey={}&nonce={}&timestamp={}&input={}&perPage=5",
        key,
        nonce,
        timestamp,
        urlencoding::encode(&query)
    );
    let salted = format!("GET\u{0000}{}\u{0000}", url);
    let mut mac = Hmac::<Sha256>::new_varkey(secret.as_bytes())
        .map_err(|err| format_err!("invalid sha256 key, {}", err))?;
    mac.input(salted.as_bytes());
    let result = hex::encode(&mac.result().code().to_vec()).to_lowercase();

    let url = format!("{}&signature={}", url, result)
        .parse::<hyper::Uri>()
        .unwrap();

    let res = client.get(url)
        .await?;

    let body = res.into_body().try_concat().await?;

    let response: LbResults = serde_json::from_slice(body.as_ref())?;

    let results = response
        .items
        .into_iter()
        .filter_map(|film_search| film_search.film)
        .map(film_to_article)
        .map(From::from)
        .collect();

    Ok(results)
}

pub fn film_to_article(mut film: Film) -> InlineQueryResultArticle {
    let mut message = InputTextMessageContent {
        message_text: "".into(),
        parse_mode: Some(telegram_bot::ParseMode::Html),
        disable_web_page_preview: false,
    };

    if film.links.len() > 0 {
        message.message_text = film.links.remove(0).url;
    }

    let mut article = InlineQueryResultArticle::new(film.id, film.name, message);
    if film.poster.is_some() {
        article.thumb_url(film.poster.unwrap().sizes.remove(0).url);
    }
    article
}
