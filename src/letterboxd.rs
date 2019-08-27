use failure::{format_err, Error};
use futures::TryStreamExt;
use hyper::Client;
use hyper_tls::HttpsConnector;
use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name, Predicate};
use telegram_bot::{InlineQueryResult, InlineQueryResultArticle, InputTextMessageContent};

const LETTERBOXD_URL: &str = "https://letterboxd.com";

pub async fn search(query: &str) -> Result<Vec<InlineQueryResult>, Error> {
    let https = HttpsConnector::new(1)?;
    let client = Client::builder().build::<_, hyper::Body>(https);

    let url = format!("{}/search/{}/", LETTERBOXD_URL, urlencoding::encode(&query))
        .parse::<hyper::Uri>()
        .unwrap();
    let res = client.get(url).await?;

    let body = res.into_body().try_concat().await?;

    let document = Document::from_read(body.as_ref())?;

    document
        .find(Class("results").descendant(Name("div")))
        .filter(|node| node.attr("data-film-name").is_some())
        .take(10)
        .map(node_to_article)
        .collect()
}

pub fn node_to_article(node: Node) -> Result<InlineQueryResult, Error> {
    let target_link = get_node_attr(&node, "data-target-link")?;
    let film_id = get_node_attr(&node, "data-film-id")?;
    let name = get_node_attr(&node, "data-film-name")?;
    let year = get_node_attr(&node, "data-film-release-year")?;

    let message = InputTextMessageContent {
        message_text: format!("{}{}", LETTERBOXD_URL, target_link),
        parse_mode: Some(telegram_bot::ParseMode::Html),
        disable_web_page_preview: false,
    };

    let mut article =
        InlineQueryResultArticle::new(film_id, format!("{} ({})", name, year), message);
    let posters: Vec<&str> = node
        .find(Name("img"))
        .filter_map(|node| node.attr("src"))
        .collect();
    if !posters.is_empty() {
        article.thumb_url(posters[0]);
    }
    Ok(From::from(article))
}

fn get_node_attr<'a>(node: &'a Node, attr: &str) -> Result<&'a str, Error> {
    node.attr(attr)
        .ok_or_else(|| format_err!("missing {}", attr))
}
