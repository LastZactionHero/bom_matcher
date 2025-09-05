mod bom_items;
mod digikey;

use bom_items::{BomItem, match_bom};
use clap::{Parser, error::Result};
use digikey::digikey_keyword_search;
use std::error::Error;
use std::fs;

// - Gemini calls
// - Turn into search queries
// - Search on Digikey
// - Store Digikey results as part items
// - Identify Digikey matches based on simple heuristics
// - Save results as JSON

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(short, long)]
    filename: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let contents = fs::read_to_string(args.filename).expect("Unable to read BOM CSV file.");

    let bom = match_bom(contents);
    for bom_item in bom.included_items() {
        // println!("{:?}", bom_item);
        println!("{:?}", bom_item.search_keywords())
    }

    // let result = digikey_keyword_search("10k 0402".to_string()).await;

    // if let Ok(response) = result {
    // println!("{:?}", response);
    // }

    Ok(())
}
