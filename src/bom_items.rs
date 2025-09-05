use csv::ReaderBuilder;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Bom {
    pub items: Vec<BomItem>,
}

impl Bom {
    pub fn included_items(&self) -> Vec<BomItem> {
        self.items
            .iter()
            .filter(|&item| item.exclude_from_bom.is_empty() && item.exclude_from_board.is_empty())
            .cloned()
            .collect()
    }
}

#[derive(Debug, Deserialize, Clone)]
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
    pub internal_id: Option<u32>,
}

impl BomItem {
    pub fn search_keywords(&self) -> Vec<String> {
        let mut keywords = Vec::new();
        keywords.push(format!("{} {}", self.value, self.footprint));
        keywords
    }
}

pub fn match_bom(contents: String) -> Bom {
    let mut rdr = ReaderBuilder::new().from_reader(contents.as_bytes());

    let mut items = Vec::new();
    let mut internal_id = 0;
    for item_result in rdr.deserialize::<BomItem>() {
        let mut item = item_result.expect("Failed to deserialize row");
        item.internal_id = Option::Some(internal_id);
        items.push(item);
    }
    Bom { items }
}
