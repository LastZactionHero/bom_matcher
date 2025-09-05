mod bom;
mod digikey;
mod gemini;
mod part_match;

use bom::{BomItem, parse_bom};
use clap::{Parser, error::Result};
use digikey::digikey_keyword_search;
use gemini::generate_content;
use part_match::match_bom_to_parts;
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

    let bom = parse_bom(contents);
    for bom_item in bom.from_included_items().items {
        // println!("{:?}", bom_item);
        println!("{:?}", bom_item.search_keywords())
    }

    match_bom_to_parts(&bom).await?;

    Ok(())
}
