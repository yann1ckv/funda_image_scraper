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

    let rua = get_rua();

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::ACCEPT_LANGUAGE,
        header::HeaderValue::from_str("gzip, deflate, br")?,
    );
    headers.insert(header::USER_AGENT, header::HeaderValue::from_str(&rua)?);
    headers.insert(
        header::ORIGIN,
        header::HeaderValue::from_str("https://www.funda.nl")?,
    );
    headers.insert(
        header::REFERER,
        header::HeaderValue::from_str("https://www.funda.nl")?,
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let res = client
        .get("https://www.funda.nl/koop/amsterdam/huis-42122722-teldershof-78/#overzicht")
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
