use serde::Deserialize;
use csv::ReaderBuilder;

#[derive(Debug, Deserialize)]
pub struct BomItem {
    #[serde(rename = "Reference")]
    pub reference: String,
    #[serde(rename = "Qty")]
    pub qty: u32,
    #[serde(rename = "Value")]
    pub value: String,
    #[serde(rename = "Exclude from BOM")]
    pub exclude_from_bom: String,
    #[serde(rename = "Exclude from Board")]
    pub exclude_from_board: String,
    #[serde(rename = "Footprint")]
    pub footprint: String,
    #[serde(rename = "Datasheet")]
    pub datasheet: String,
}

pub fn match_bom(contents: String) -> Vec<BomItem> {
    let mut rdr = ReaderBuilder::new().from_reader(contents.as_bytes());

    let mut bom_items = Vec::new();
    for bom_item in rdr.deserialize().flatten() {
        bom_items.push(bom_item);
    }
    bom_items
}
