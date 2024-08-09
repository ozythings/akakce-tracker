use fantoccini::{Client, Locator};
use scraper::{Html, Selector};
use reqwest::Client as ReqwestClient;
use tokio::time::{interval, Duration};
use std::error::Error;

async fn load_page() -> Result<String, Box<dyn Error>> {
    let client = Client::new("http://localhost:4444").await?;
    client.goto("https://www.akakce.com/akilli-saat/en-ucuz-samsung-galaxy-watch-7-44mm-fiyati,708620942.html").await?;
    let _container = client.find(Locator::Css("#PL_h")).await?;
    let page_source = client.source().await?;
    Ok(page_source)
}

async fn process_page() -> Result<(), Box<dyn Error>> {
    let page_source = load_page().await?;
    let document = Html::parse_document(&page_source);
    let container_selector = Selector::parse("div#PL_h").unwrap();
    let price_selector = Selector::parse("span.pt_v8").unwrap();

    let mut prices_as_string = Vec::new();

    if let Some(container) = document.select(&container_selector).next() {
        for price_element in container.select(&price_selector) {
            let price = price_element.text().collect::<String>().trim().to_string();
            prices_as_string.push(price.clone());
            println!("Price: {}", price);
        }
    } else {
        println!("No container with id 'PL_h' found.");
    }

    let mut prices_int: Vec<i32> = prices_as_string.iter()
        .filter_map(|price_str| {
            let cleaned_price = price_str
                .replace(" TL", "")
                .replace(".", "")
                .replace(",", ".");

            if let Ok(price_float) = cleaned_price.parse::<f64>() {
                Some(price_float.round() as i32)
            } else {
                None
            }
        })
        .collect();

    prices_int.sort();

    println!("Sorted Prices:");
    for price in &prices_int {
        println!("{}", price);
    }

    send_pushbullet_notification(prices_int[0].to_string()).await?;

    Ok(())
}

async fn send_pushbullet_notification(message: String) -> Result<(), Box<dyn Error>> {
    let api_key = "YOUR_API_KEY";
    let client = ReqwestClient::new();
    let url = "https://api.pushbullet.com/v2/pushes";

    let response = client.post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "type": "note",
            "title": "Last Price",
            "body": message
        }))
        .send()
        .await?;

    if response.status().is_success() {
        println!("Notification sent successfully.");
    } else {
        eprintln!("Failed to send notification: {:?}", response.status());
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut interval = interval(Duration::from_secs(21600)); // 24 hours

    loop {
        interval.tick().await;
        if let Err(e) = process_page().await {
            eprintln!("Error processing page: {}", e);
        }
    }
}

