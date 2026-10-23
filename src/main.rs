use core::fmt::Display;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write as _;

use petgraph::prelude::DiGraphMap;
use serde::Deserialize;

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

    let graph = data
        .recipes
        .iter()
        .filter(|(_, recipe)| {
            recipe
                .categories
                .iter()
                .any(|category| categories.contains(category))
        })
        .filter(|(name, _)| !name.contains("methanol-gas"))
        .filter(|(name, _)| !name.contains("cage"))
        .filter(|(name, _)| !name.contains("barrel"))
        .flat_map(|(_, recipe)| {
            recipe.ingredients.iter().flat_map(|ingredient| {
                recipe
                    .results
                    .iter()
                    .map(|result| (ingredient.name, result.name))
            })
        })
        .map(|(ingredient, result)| (ingredient, result, Empty))
        .collect::<DiGraphMap<_, _>>();

    // println!("{}", petgraph::dot::Dot::new(&graph));

    // use graph_cycles::Cycles;
    // let cycles = graph.cycles();
    // println!("{:?}", cycles);

    let mut stdout = std::io::stdout().lock();
    petgraph::algo::toposort(&graph, None)
        .unwrap()
        .iter()
        .enumerate()
        .for_each(|(index, name)| writeln!(&mut stdout, "{},{}", index, name).unwrap());
}

#[derive(Debug)]
struct Empty;

impl Display for Empty {
    fn fmt(&self, _: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
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

    #[serde(borrow, deserialize_with = "deserialize_empty_vec")]
    results: Vec<Ingredient<'a>>,
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
