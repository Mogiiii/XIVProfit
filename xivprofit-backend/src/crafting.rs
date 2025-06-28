use log::{error, info};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
pub(crate) struct Item {
    pub(crate) name: String,
    pub(crate) id: usize,
}
#[derive(Clone, Serialize, Debug)]
pub(crate) struct Recipe {
    pub(crate) id: usize,
    pub(crate) result_item_id: usize,
    pub(crate) result_item_quantity: usize,
    //itemid, quantity
    pub(crate) ingredients: Vec<(usize, usize)>,
}

//#[derive(Clone)]
pub(crate) struct ItemData {
    pub(crate) items: Vec<Item>,
    pub(crate) recipes: Vec<Recipe>,
    pub(crate) craftable_items: Vec<Item>,
}

#[derive(Clone, Deserialize)]
struct ItemCsvRow {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "#")]
    id: usize,
}

#[derive(Deserialize)]
struct RecipeCsvRow {
    #[serde(rename = "#")]
    id: i32,
    #[serde(rename = "Item{Result}")]
    result_item_id: i32,
    #[serde(rename = "Amount{Result}")]
    result_item_amount: i32,
    #[serde(rename = "Item{Ingredient}[0]")]
    ingredient_0_item_id: i32,
    #[serde(rename = "Item{Ingredient}[1]")]
    ingredient_1_item_id: i32,
    #[serde(rename = "Item{Ingredient}[2]")]
    ingredient_2_item_id: i32,
    #[serde(rename = "Item{Ingredient}[3]")]
    ingredient_3_item_id: i32,
    #[serde(rename = "Item{Ingredient}[4]")]
    ingredient_4_item_id: i32,
    #[serde(rename = "Item{Ingredient}[5]")]
    ingredient_5_item_id: i32,
    #[serde(rename = "Item{Ingredient}[6]")]
    ingredient_6_item_id: i32,
    #[serde(rename = "Item{Ingredient}[7]")]
    ingredient_7_item_id: i32,
    #[serde(rename = "Amount{Ingredient}[0]")]
    ingredient_0_amount: i32,
    #[serde(rename = "Amount{Ingredient}[1]")]
    ingredient_1_amount: i32,
    #[serde(rename = "Amount{Ingredient}[2]")]
    ingredient_2_amount: i32,
    #[serde(rename = "Amount{Ingredient}[3]")]
    ingredient_3_amount: i32,
    #[serde(rename = "Amount{Ingredient}[4]")]
    ingredient_4_amount: i32,
    #[serde(rename = "Amount{Ingredient}[5]")]
    ingredient_5_amount: i32,
    #[serde(rename = "Amount{Ingredient}[6]")]
    ingredient_6_amount: i32,
    #[serde(rename = "Amount{Ingredient}[7]")]
    ingredient_7_amount: i32,
}

impl From<ItemCsvRow> for Item {
    fn from(value: ItemCsvRow) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

impl From<RecipeCsvRow> for Recipe {
    fn from(value: RecipeCsvRow) -> Self {
        let mut ingredients = Vec::new();
        if value.ingredient_0_amount > 0 {
            ingredients.push((
                value.ingredient_0_item_id as usize,
                value.ingredient_0_amount as usize,
            ));
        }
        if value.ingredient_1_amount > 0 {
            ingredients.push((
                value.ingredient_1_item_id as usize,
                value.ingredient_1_amount as usize,
            ));
        }
        if value.ingredient_2_amount > 0 {
            ingredients.push((
                value.ingredient_2_item_id as usize,
                value.ingredient_2_amount as usize,
            ));
        }
        if value.ingredient_3_amount > 0 {
            ingredients.push((
                value.ingredient_3_item_id as usize,
                value.ingredient_3_amount as usize,
            ));
        }
        if value.ingredient_4_amount > 0 {
            ingredients.push((
                value.ingredient_4_item_id as usize,
                value.ingredient_4_amount as usize,
            ));
        }
        if value.ingredient_5_amount > 0 {
            ingredients.push((
                value.ingredient_5_item_id as usize,
                value.ingredient_5_amount as usize,
            ));
        }
        if value.ingredient_6_amount > 0 {
            ingredients.push((
                value.ingredient_6_item_id as usize,
                value.ingredient_6_amount as usize,
            ));
        }
        if value.ingredient_7_amount > 0 {
            ingredients.push((
                value.ingredient_7_item_id as usize,
                value.ingredient_7_amount as usize,
            ));
        }
        Self {
            id: value.id as usize,
            result_item_id: value.result_item_id as usize,
            result_item_quantity: value.result_item_amount as usize,
            ingredients,
        }
    }
}

impl ItemData {
    pub(crate) async fn new() -> Self {
        info!("loading ItemData");
        let (item_data, recipe_data) = get_data().await;
        //item data
        let item_data = clean_csv(item_data);
        let mut item_csv = csv::Reader::from_reader(item_data.as_bytes());
        let mut items = Vec::new();
        for result in item_csv.deserialize() {
            match result {
                Ok(i) => {
                    let item: ItemCsvRow = i;
                    items.push(Item::from(item));
                }
                Err(e) => {
                    error!("{e}");
                }
            }
        }
        //recipe data
        let recipe_data = clean_csv(recipe_data);
        let mut recipe_csv = csv::Reader::from_reader(recipe_data.as_bytes());
        let mut recipes = Vec::new();
        let mut craftable_items = Vec::new();
        for result in recipe_csv.deserialize() {
            match result {
                Ok(i) => {
                    let recipe: RecipeCsvRow = i;
                    let processed_recipe = Recipe::from(recipe);
                    if processed_recipe.ingredients.len() > 0 {
                        let items = items.clone();
                        let item = items
                            .iter()
                            .find(|i| i.id == processed_recipe.result_item_id);
                        recipes.push(processed_recipe);
                        craftable_items.push(item.unwrap().clone());
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            }
        }
        info!("finished loading ItemData");
        Self {
            items,
            recipes,
            craftable_items,
        }
    }
}

async fn get_data() -> (String, String) {
    let client = reqwest::Client::new();
    let item_data: String = client
        .get("https://raw.githubusercontent.com/viion/ffxiv-datamining/master/csv/Item.csv")
        .send()
        .await
        .expect("failed to download item data from github")
        .text()
        .await
        .expect("failed to decode github response");

    let recipe_data = client
        .get("https://raw.githubusercontent.com/viion/ffxiv-datamining/master/csv/Recipe.csv")
        .send()
        .await
        .expect("failed to download item data from github")
        .text()
        .await
        .expect("failed to decode github response");

    // let item_data_length = item_data.chars().count();
    // let recipe_data_length = recipe_data.chars().count();
    // info!("item_data length: {item_data_length}, recipe_data length: {recipe_data_length}");
    (item_data, recipe_data)
}

fn clean_csv(mut data: String) -> String {
    //remove first line (indexes)
    data = data
        .split_at(data.find("\n").expect("failed csv parsing") + 1)
        .1
        .to_string();
    //extract header row
    let header_row = data
        .split_at(data.find("\n").expect("failed csv parsing") + 1)
        .0
        .to_string();
    //remove header row
    let data = data
        .split_at(data.find("\n").expect("failed csv parsing") + 1)
        .1
        .to_string();
    //remove 3rd line (data types)
    let data = data
        .split_at(data.find("\n").expect("failed csv parsing") + 1)
        .1
        .to_string();
    //replace header row
    let data = [header_row, data].concat();
    data
}
