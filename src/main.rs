use chrono::Timelike;
use dwbhk::{EmbedBuilder, WebhookBuilder, WebhookRequestBuilder};

enum DiscountStatus {
    Active(String),
    Inactive,
    Error
}

#[tokio::main]
async fn main() {
    let mut prev_discount = String::new();

    loop {
        let now = chrono::Local::now();
        let hour = now.hour();

        if hour.eq(&9) {
            match get_discount().await {
                DiscountStatus::Active(discount) => {
                    // only send webhook if discount is different from previous
                    if !prev_discount.eq(&discount) {
                        prev_discount = discount.clone();
                        send_webhook(
                            "Er is een korting gevonden!",
                            format!("Er is {} op de whey perfection!", discount).as_str()
                        ).await;
                    }
                },
                DiscountStatus::Inactive => {
                    // reset prev_discount
                    prev_discount = String::new();
                    println!("No discount found");
                },
                DiscountStatus::Error => {
                   send_webhook(
                       "Er is een verandering gevonden in de website!",
                       "Laat de developer weten dat er een verandering is in de website!"
                   ).await;
                }
            }
        }

        // wait 1 hour to check again
        let hour_in_secs = 60 * 60;
        tokio::time::sleep(tokio::time::Duration::from_secs(hour_in_secs)).await;
    }
}

async fn get_discount() -> DiscountStatus {
    println!("Checking for discount...");
    let url = "https://www.bodyandfit.com/nl-nl/Producten/Eiwitten/Zuivelprote%C3%AFne/Whey-Protein/Whey-Perfection/p/whey-perfection?gai=ChMzNDkxNDY1Nzc5NTM4MTUxNDg1EAIaI3Byb2R1Y3RfcmVfb3RoZXJzLXlvdV8xNTkxMzQ5MDIwMzE0IhZwcm9kdWN0X2RldGFpbF9kZWZhdWx0KAA";
    let response = reqwest::get(url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let document = scraper::Html::parse_document(&response);
    let discount_selector = scraper::Selector::parse(".promo-text").unwrap();
    let price_selector = scraper::Selector::parse(".product-price__value").unwrap();
    let prices = document.select(&price_selector).next().unwrap().text().collect::<Vec<_>>();
    let discounts = document.select(&discount_selector).next().unwrap().text().collect::<Vec<_>>();

    // check if there is a price
    if prices.is_empty() {
        return DiscountStatus::Error;
    }

    return if discounts.len().eq(&1) {
        let discount = discounts[0].replace("\n", "");
        DiscountStatus::Active(discount)
    } else {
        DiscountStatus::Inactive
    }
}

async fn send_webhook(title: &str, message: &str) {
    let req = WebhookRequestBuilder::new()
        .set_data(WebhookBuilder::new()
            .set_embeds(vec![
                EmbedBuilder::new()
                    .set_title(title)
                    .set_color_hex("#ff0000")
                    .set_description(message)
                    .build()
            ])
            .build()
        )
        .build();

    let url = "https://discord.com/api/webhooks/1142709565641719888/pd14UTzpXVmvoweMUnpWfEUy8wwT6_UfeBM6uVUEwpvFbmWxn_vezfklaeH_Tpr5G9EY";

    req.execute_url(url).await.expect("TODO: panic message");
}