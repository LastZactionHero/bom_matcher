use crate::bom::{Bom, BomItem};
use crate::digikey::digikey_keyword_search;
use crate::gemini::{extract_json, generate_content};
use serde::Serialize;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Debug, Serialize)]
struct BomItemKeywordResult {
    id: u32,
    keywords: Vec<String>,
}

pub async fn match_bom_to_parts(bom: &Bom) -> Result<(), Box<dyn Error>> {
    let max_concurrent = 10;
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    let mut handles = Vec::new();

    for item in &bom.items {
        let item = item.clone();
        let semaphore = semaphore.clone();

        let handle = tokio::task::spawn_blocking(move || {
            // This runs in a blocking thread pool, but we can still use async
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async move {
                let _permit = semaphore.acquire().await.unwrap();

                let result = generate_keywords_for_bom_item(&item).await;
                if let Ok(keywords) = result {
                    Some(BomItemKeywordResult {
                        id: item.internal_id.expect("BomItem expects internal_id"),
                        keywords,
                    })
                } else {
                    None
                }
            })
        });

        handles.push(handle);
    }

    for handle in handles {
        if let Ok(Some(result)) = handle.await {
            println!("{:?}", result);
        }
    }

    Ok(())
}

async fn generate_keywords_for_bom_item(bom_item: &BomItem) -> Result<Vec<String>, Box<dyn Error>> {
    // Same implementation as before
    let prompt = format!(
        r##"
You are an expect electrical engineer and PCB designer. You are tasked with finding the appropriate part on Digikey for the following part:

{}

From the above, please construct 1-3 search keywords. These will be used to query Digikey.

Keyword terms should:
- Be descriptive enough to yield relevant results
- Avoid excessive detail that may yield no results
- Remove internal, project-specific jargon

Remember, this is the first pass in an automated system. Cast a broader net so that we can evaluate a list of parts- but no so broad we get junk. Place parsable JSON array within triple-backticks, as this will be parsed.

Examples:

> "15.4k hackrf:GSG-0402"

```
[
    "15.4k 0402"
]
```
  
> "hackrf:GSG-SWITCH-FSMRA"

```
[
    "switch fsmra"
]
```

> "GSG-XC2C64A-7VQG100C hackrf:GSG-VQ100'

```
[
    "XC2C64A 7VQG100C VQ100",
    "XC2C64A-7VQG100C",
    "XC2C64A"
] 
```
 "##,
        bom_item.reference_string()
    );

    let generate_response = generate_content(prompt).await?;
    let generate_json = extract_json(generate_response.as_str()).unwrap_or("[]".to_string());

    let keywords: Vec<String> =
        serde_json::from_str(generate_json.as_str()).expect("Unable to parse keywords");
    Ok(keywords)
}
