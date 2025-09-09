use anyhow::{Result, anyhow};
use dotenv::dotenv;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, str::FromStr};

// Represents the response from the OAuth2 token endpoint.
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub token_type: String,
}

// Represents the main request body for the keyword search.
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SearchRequest<'a> {
    pub keywords: &'a str,
    pub limit: u32,
    pub offset: u32,
}

// --- START: Full Data Structures for Search Response ---

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SearchResponse {
    pub products: Vec<Product>,
    pub products_count: u32,
    // We are not mapping the full filter options for this example,
    // but you could create structs for them here.
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Product {
    pub description: Description,
    pub manufacturer: Manufacturer,
    pub manufacturer_product_number: String,
    pub unit_price: f64,
    pub product_url: Option<String>,
    pub datasheet_url: Option<String>,
    pub photo_url: Option<String>,
    pub product_variations: Vec<ProductVariation>,
    pub quantity_available: u32,
    pub product_status: ProductStatus,
    pub parameters: Vec<Parameter>,
    pub category: Category,
    pub series: Series,
    pub classifications: Classifications,
    pub other_names: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Description {
    pub product_description: String,
    pub detailed_description: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Manufacturer {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ProductVariation {
    pub digi_key_product_number: String,
    pub package_type: PackageType,
    pub standard_pricing: Vec<StandardPricing>,
    pub quantity_availablefor_package_type: u32,
    pub minimum_order_quantity: u32,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PackageType {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct StandardPricing {
    pub break_quantity: u32,
    pub unit_price: f64,
    pub total_price: f64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ProductStatus {
    pub id: u32,
    pub status: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Parameter {
    pub parameter_id: u32,
    pub parameter_text: String,
    pub value_text: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Category {
    pub category_id: u32,
    pub name: String,
    pub child_categories: Vec<Category>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Series {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Classifications {
    pub reach_status: String,
    pub rohs_status: String,
    #[serde(rename = "MoistureSensitivityLevel")]
    pub moisture_sensitivity_level: String,
    #[serde(rename = "ExportControlClassNumber")]
    pub export_control_class_number: String,
    #[serde(rename = "HtsusCode")]
    pub htsus_code: String,
}

async fn get_access_token(client_keys: &ClientKeys) -> Result<TokenResponse> {
    let client = Client::new();

    let token_url = "https://api.digikey.com/v1/oauth2/token";
    let params = [
        ("grant_type", "client_credentials"),
        ("client_id", client_keys.client_id.as_str()),
        ("client_secret", client_keys.client_secret.as_str()),
    ];
    let response = client.post(token_url).form(&params).send().await?;
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Digikey access token fetch failed: {}", error_text).into());
    }

    let token_response: TokenResponse = response.json::<TokenResponse>().await?;
    Ok(token_response)
}

struct ClientKeys {
    client_id: String,
    client_secret: String,
}

fn get_client_keys() -> ClientKeys {
    dotenv().ok();

    let digikey_client_id =
        std::env::var("DIGIKEY_CLIENT_ID").expect("DIGIKEY_CLIENT_ID must be set");
    let digikey_client_secret =
        std::env::var("DIGIKEY_CLIENT_SECRET").expect("DIGIKEY_CLIENT_SECRET must be set");

    ClientKeys {
        client_id: digikey_client_id,
        client_secret: digikey_client_secret,
    }
}

pub async fn digikey_keyword_search(query: &String) -> Result<SearchResponse> {
    let client_keys = get_client_keys();
    let access_token_response = get_access_token(&client_keys).await?;

    let search_request = SearchRequest {
        keywords: query.as_str(),
        limit: 30,
        offset: 0,
    };

    let search_url = "https://api.digikey.com/products/v4/search/keyword";

    let client = reqwest::Client::new();
    let response = client
        .post(search_url)
        .header(
            "Authorization",
            format!("Bearer {}", access_token_response.access_token),
        )
        .header("X-DIGIKEY-Client-Id", client_keys.client_id)
        .header("Content-Type", "application/json")
        .json(&search_request)
        .send()
        .await?;
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Search request failed: {}", error_text).into());
    }

    let search_response: SearchResponse = response.json::<SearchResponse>().await?;
    Ok(search_response)
}
