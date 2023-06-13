use dialoguer::{theme::ColorfulTheme, Input, Select};
use fake_user_agent::get_rua;
use mime::Mime;
use reqwest::{self, header};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::{
    fs::{create_dir, File},
    io::{copy, Cursor},
};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Funda house url")
        .interact_text()
        .unwrap();

    // retrieves the scraped HTML document
    let document = retrieve_html_document(&url).await?;
    // returns a hashmap of the images in the HTML document, with the possible resolutions as keys
    let image_url_map = fill_image_url_map(&document)?;

    // initializes a set so unique found resolution types can be added an user is able to
    // choose between them from the command line
    let mut resolutions = Vec::from_iter(image_url_map.keys());
    resolutions.sort();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Image resolution")
        .items(&resolutions[..])
        .interact()
        .unwrap();

    let path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Folder location")
        .validate_with(|input: &String| -> Result<(), &str> {
            if Path::new(input).is_dir() {
                Ok(())
            } else {
                Err("This is not a folder on your machine")
            }
        })
        .interact_text()
        .unwrap();

    let folder_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Image folder name")
        .interact_text()
        .unwrap();

    // create the folder in which to store the images
    create_dir(format!("{}/{}", path, folder_name))?;

    // retrieve the images for the selected resolution
    let result = image_url_map
        .get(resolutions[selection])
        .unwrap()
        .to_owned();

    let mut counter = 1;

    // download and save the images
    for image_url in result {
        let response = reqwest::get(image_url).await?;
        let headers = response.headers();
        let content_type = headers.get(header::CONTENT_TYPE).unwrap();
        let content_type = Mime::from_str(content_type.to_str()?)?;

        let mut file = File::create(format!(
            "{}/{}/{}.{}",
            path,
            folder_name,
            counter,
            content_type.subtype()
        ))
        .expect("create failed");
        let mut content = Cursor::new(response.bytes().await?);

        copy(&mut content, &mut file)?;
        counter += 1;
    }

    println!("Done storing images in {}/{}", path, folder_name);
    Ok(())
}

async fn retrieve_html_document(url: &str) -> Result<Html, Box<dyn std::error::Error>> {
    let mut document;

    let client = reqwest::Client::new();

    // loop until we find the correct HTML document. It can be that a captcha page has been triggered.
    // We then wait 100ms and try again.
    loop {
        // Gets a random user agent (Chrome, Opera, Firefox, Safari, Edge, or IE).
        let rua = get_rua();

        let res: reqwest::Response = client
            .get(format!("{}#overzicht", url))
            .header(header::USER_AGENT, &rua)
            .header("scheme", "https")
            .header(header::ACCEPT, "application/json, text/plain, */*")
            .header(
                header::ACCEPT_LANGUAGE,
                "en-US,en;q=0.9,es;q=0.8,nl;q=0.7,ja;q=0.6",
            )
            .header(header::ORIGIN, "https://www.funda.nl")
            .header(header::REFERER, format!("{}#overzicht", url))
            .header(header::CACHE_CONTROL, "no-cache")
            .header(header::PRAGMA, "no-cache")
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-origin")
            .send()
            .await?;

        let body = res.text().await?;
        document = Html::parse_document(&body);

        // check for a captcha page
        let check = document
            .html()
            .find("Je bent bijna op de pagina die je zoekt");

        // if no captcha page is found, we break out of the loop, because we retrieved the listing HTML page
        if check.is_none() {
            break;
        }

        sleep(Duration::from_millis(100)).await
    }

    Ok(document)
}

fn fill_image_url_map(
    document: &Html,
) -> Result<HashMap<u16, Vec<&str>>, Box<dyn std::error::Error>> {
    let image_selector = Selector::parse("img.media-viewer-overview__section-image").unwrap();
    let mut image_url_map: HashMap<u16, Vec<&str>> = HashMap::new();

    // go through the found classes and store them in a hashmap where the key is the resolution and the values are the image URLs
    for media_element in document.select(&image_selector) {
        if media_element.value().attr("data-lazy-srcset").is_some() {
            let string = media_element.value().attr("data-lazy-srcset").unwrap();
            let split_string: Vec<&str> = string.split(',').collect::<Vec<&str>>();

            for image_string in split_string {
                let image_url = image_string.split_whitespace().next().unwrap_or("");

                let resolution = image_string
                    .split_whitespace()
                    .next_back()
                    .unwrap_or("")
                    .replace('w', "")
                    .parse::<u16>()
                    .unwrap();

                image_url_map
                    .entry(resolution)
                    .or_insert(Vec::new())
                    .push(image_url);
            }
        }
    }

    Ok(image_url_map)
}
