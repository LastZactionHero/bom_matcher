mod bom_items;

use bom_items::{BomItem, match_bom};
use clap::{Parser, error::Result};
use std::error::Error;
use std::fs;

// # Pass 1: Simple logic-based parsing
// - Consolidate matching BOM part items
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

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let contents = fs::read_to_string(args.filename).expect("Unable to read BOM CSV file.");

    let bom_items = match_bom(contents);
    for bom_item in bom_items {
        println!("{:?}", bom_item);
    }

    Ok(())
}
