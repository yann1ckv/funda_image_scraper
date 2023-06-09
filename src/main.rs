use fake_user_agent::get_rua;
use reqwest::{self, header};
use scraper::{Html, Selector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url: String = std::env::args().nth(1).expect("Please provide a URL");

    struct ImageResult<'a> {
        title: &'a str,
        images: Vec<Image<'a>>,
    }

    struct Image<'a> {
        url: &'a str,
        size: u8,
    }

    // Gets a random user agent (Chrome, Opera, Firefox, Safari, Edge, or IE).
    let rua = get_rua();

    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}#overzicht", url))
        .header(header::USER_AGENT, &rua)
        .header(header::ACCEPT_LANGUAGE, "gzip, deflate, br")
        .header(header::ORIGIN, "https://www.funda.nl")
        .header(header::REFERER, "https://www.funda.nl")
        .send()
        .await?;

    let body = res.text().await?;
    let document = Html::parse_document(&body);
    let image_selector = Selector::parse("li").unwrap();
    let ul = document.select(&image_selector).next().unwrap();

    println!("{:?}", ul);
    for media_element in document.select(&image_selector) {
        println!("{:?}", media_element);
        let title = media_element.text().collect::<Vec<_>>();
        println!("Title: {}", title[0]);
    }
    Ok(())
}
