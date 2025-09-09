use crate::bom::{Bom, BomItem};
use crate::gemini::{extract_json, generate_content};
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct BomItemKeywordResult {
    id: u32,
    keywords: Vec<String>,
}
pub async fn generate_keywords_for_bom_item(bom_item: &BomItem) -> Result<Vec<String>> {
    let prompt = build_keyword_prompt_string(&bom_item.reference_string());
    let generate_response = generate_content(prompt.clone()).await?;
    let generate_json = extract_json(generate_response.as_str()).unwrap_or("[]".to_string());
    let keywords: Vec<String> =
        serde_json::from_str(generate_json.as_str()).expect("Unable to parse keywords");
    Ok(keywords)
}

fn build_keyword_prompt_string(reference_string: &String) -> String {
    format!(
        r##"
You are an expect electrical engineer and PCB designer. You are tasked with finding the appropriate part from Digikey.

From the above, please construct 1-3 search keywords. These will be used to query Digikey.

Keyword terms should:
- Be descriptive enough to yield relevant results
- Avoid excessive detail that may yield no results
- Remove internal, project-specific jargon

Remember, this is the first pass in an automated system. Cast a broader net so that we can evaluate a list of parts- but no so broad we get junk. Place parsable JSON array within triple-backticks, as this will be parsed.

ALWAYS RETURN AT LEAST ONE KEYWORD

Examples:

> "15.4k hackrf:GSG-0402"

```json
[
    "15.4k 0402"
]
```
  
> "hackrf:GSG-SWITCH-FSMRA"

```json
[
    "switch fsmra"
]
```

> "GSG-XC2C64A-7VQG100C hackrf:GSG-VQ100'

```json
[
    "XC2C64A 7VQG100C VQ100",
    "XC2C64A-7VQG100C",
    "XC2C64A"
]

===========================

 Return keywords for this part:

> {}
```
 "##,
        reference_string
    )
}
