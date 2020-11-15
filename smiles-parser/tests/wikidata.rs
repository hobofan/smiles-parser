// Wikidata query used to generate the data (at https://query.wikidata.org)
//
// SELECT ?item ?itemLabel ?smiles
// WHERE
// {
//   ?item wdt:P31 wd:Q11173.
//   ?item wdt:P233 ?value.
//   ?item wdt:P233 ?smiles
//   SERVICE wikibase:label { bd:serviceParam wikibase:language "[AUTO_LANGUAGE],en". }
// }
// LIMIT 5000

use serde::Deserialize;
use smiles_parser::chain;

#[derive(Debug, Clone, Deserialize)]
struct WikidataItem {
    pub item: String,
    #[serde(rename = "itemLabel")]
    pub item_label: String,
    pub smiles: String,
}

#[test]
fn parse_wikidata_items() {
    let contents = std::fs::read_to_string("./tests/wikidata_molecules.json").unwrap();
    let items: Vec<WikidataItem> = serde_json::from_str(&contents).unwrap();

    for item in items {
        let res = chain(&item.smiles.as_bytes());
        match res {
            Ok(_) => {
                // println!("Correctly parse SMILES: {}", &item.smiles);
            }
            Err(_) => {
                println!("Failed to parse SMILES: {}", &item.smiles);
            }
        }
    }
}
