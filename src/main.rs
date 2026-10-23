use core::marker::PhantomData;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

use petgraph::prelude::DiGraphMap;
use serde::Deserialize;
use serde_json::Value;

use automall::deserialize_empty_vec;
use automall::deserialize_item_map;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let mut buffer = Vec::new();
    File::open(path)
        .map(BufReader::new)
        .map(flate2::read::GzDecoder::new)
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let data = serde_json::from_slice::<Data>(&buffer).unwrap();

    let categories = data
        .machines
        .iter()
        .filter(|(name, _)| name.starts_with("automated-factory"))
        .flat_map(|(_, machine)| &machine.categories)
        .copied()
        .collect::<HashSet<&str>>();

    println!("{:?}", categories);

    // let mut graph = DiGraphMap::<&str, ()>::new();
    // 'outer: for (name, recipe) in &data.recipes {
    //     for ingredient in recipe.ingredients {
    //         match ingredient.r#type {
    //             Type::Fluid => continue 'outer,
    //         }
    //         graph.add_edge()
    //     }
    // }
}

#[derive(Debug, Deserialize)]
struct Data<'a> {
    #[serde(
        borrow,
        deserialize_with = "deserialize_item_map",
        rename = "assembling-machine"
    )]
    machines: Vec<(&'a str, Machine<'a>)>,
    #[serde(borrow, deserialize_with = "deserialize_item_map", rename = "recipe")]
    recipes: Vec<(&'a str, Recipe<'a>)>,
}

#[derive(Debug, Deserialize)]
struct Machine<'a> {
    #[serde(borrow, rename = "crafting_categories")]
    categories: Vec<&'a str>,
    // #[serde(borrow, deserialize_with = "deserialize_empty_vec")]
    // ingredients: Vec<Ingredient<'a>>,
}

#[derive(Debug, Deserialize)]
struct Recipe<'a> {
    #[serde(borrow, default)]
    categories: Vec<&'a str>,
    #[serde(borrow, deserialize_with = "deserialize_empty_vec")]
    ingredients: Vec<Ingredient<'a>>,
}

#[derive(Debug, Deserialize)]
struct Ingredient<'a> {
    r#type: Type,
    name: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Type {
    Item,
    Fluid,
}
