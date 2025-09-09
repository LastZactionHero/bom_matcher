mod bom;
mod digikey;
mod gemini;
mod part_match;

use anyhow::{Result, anyhow};
use bom::{BomItem, parse_bom};
use clap::Parser;
use std::error::Error;
use std::fs;

use crate::digikey::{Product, SearchResponse, digikey_keyword_search};
use crate::part_match::generate_keywords_for_bom_item;

const MAX_PARTS_PER_BOM_ITEM: u32 = 5;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(short, long)]
    filename: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    println!("Analyzing BOM: {}", args.filename);

    let mut handles = Vec::new();
    handles.push(tokio::spawn(analyze_bom(args.filename.clone())));

    for handle in handles {
        handle.await.unwrap();
    }
    Ok(())
}

async fn analyze_bom(filename: String) {
    let contents = fs::read_to_string(filename).expect("Unable to read BOM CSV file.");
    let bom = parse_bom(&contents).from_included_items();

    let mut handles = Vec::new();
    for item in bom.items {
        println!("{}", item.reference_string());
        handles.push(tokio::spawn(find_candidate_parts(item.clone())));
    }

    // TODO: Use the BomItemPartCandidates, assemble the response JSON
    for handle in handles {
        handle.await.unwrap();
    }
}

struct BomItemPartCandidates {
    part_candidates: Vec<Product>,
    bom_item_id: u32,
}

async fn find_candidate_parts(item: BomItem) -> Result<(BomItemPartCandidates)> {
    println!("Finding candidate parts for: {}", item.reference_string());
    let search_keywords = generate_keywords_for_bom_item(&item).await?;

    println!(
        "Part {} has {} keywords to search",
        item.reference_string(),
        search_keywords.len()
    );
    let mut handles = Vec::new();
    for query in search_keywords {
        handles.push(tokio::spawn(execute_digikey_search(query.clone())));
    }

    let mut maybe_longest_query_disr: Option<DigikeyIdentifiableSearchResponse> = None;

    for handle in handles {
        let disr = handle.await??;
        match maybe_longest_query_disr.as_ref() {
            None => maybe_longest_query_disr = Some(disr),
            Some(longest_query_disr) => {
                if disr.query.len() > longest_query_disr.query.len()
                    && !disr.search_response.products.is_empty()
                {
                    maybe_longest_query_disr = Some(disr);
                }
            }
        }
    }

    match maybe_longest_query_disr.as_ref() {
        None => {
            println!("No results found for {}", item.reference_string());
            Err(anyhow!("No results found for {}", item.reference_string()))
        }
        Some(disr) => {
            println!(
                "{}: best query: {}, {} results",
                item.reference_string(),
                disr.query,
                disr.search_response.products.len()
            );
            let disr_owned = maybe_longest_query_disr.unwrap();
            let part_candidates: Vec<Product> = disr_owned
                .search_response
                .products
                .into_iter()
                .take(5)
                .collect();
            Ok(BomItemPartCandidates {
                part_candidates,
                bom_item_id: item.internal_id.unwrap(),
            })
        }
    }
}

struct DigikeyIdentifiableSearchResponse {
    search_response: SearchResponse,
    query: String,
}

async fn execute_digikey_search(query: String) -> Result<DigikeyIdentifiableSearchResponse> {
    let search_response = digikey_keyword_search(&query).await?;
    Ok(DigikeyIdentifiableSearchResponse {
        search_response,
        query,
    })
}
