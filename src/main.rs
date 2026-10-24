use core::fmt::Display;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write as _;

use automall::blueprint::BlueprintString;
use petgraph::prelude::DiGraphMap;
use serde::Deserialize;

use automall::deserialize_empty_vec;
use automall::deserialize_item_map;

fn main() {
    let path = std::env::args().nth(1).unwrap();

    if path.starts_with("0") {
        let blueprint = BlueprintString::decode(&path);
        println!("{:?}", blueprint);
        return;
    }

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

    let mut craftable = HashSet::new();

    let graph = data
        .recipes
        .iter()
        // methanol <-> methanol-gas-canister <-> empty-gas-canister
        .filter(|(name, _)| !name.contains("canister"))
        .filter(|(name, _)| !name.contains("cage"))
        .filter(|(name, _)| !name.contains("barrel"))
        // nuclear-reactor <-> nuclear-reactor-mox
        .filter(|(name, _)| !name.ends_with("uncraft"))
        // auog-paddock-mk01-with-dirt
        .filter(|(name, _)| !name.ends_with("with-dirt"))
        .filter(|(_, recipe)| {
            assert!(recipe.categories.len() <= 1);
            recipe
                .categories
                .first()
                .is_none_or(|category| categories.contains(category))
        })
        // .filter(|(_, recipe)| {
        //     recipe
        //         .ingredients
        //         .iter()
        //         .chain(&recipe.results)
        //         .all(|item| matches!(item.r#type, Type::Item))
        // })
        // .inspect(|(name, recipe)| {
        //     if recipe
        //         .results
        //         .iter()
        //         .any(|item| item.name == "proton-donor")
        //     {
        //         eprintln!("{name}: {recipe:?}");
        //     }
        // })
        .flat_map(|(_, recipe)| {
            craftable.extend(recipe.results.iter().map(|result| result.name));

            recipe.results.iter().flat_map(|result| {
                recipe
                    .ingredients
                    .iter()
                    .map(|ingredient| (ingredient.name, result.name))
                    .filter(|(ingredient, result)| {
                        *ingredient != "small-lamp" && *result != "small-lamp"
                    })
            })
        })
        .map(|(ingredient, result)| (ingredient, result, Empty))
        // .map(|(ingredient, result)| {
        //     (
        //         &*ingredient.to_owned().leak(),
        //         &*result.to_owned().leak(),
        //         Empty,
        //     )
        // })
        .collect::<DiGraphMap<_, _>>();

    // println!("{}", petgraph::dot::Dot::new(&graph));

    // use graph_cycles::Cycles;
    // let cycles = graph.cycles();
    // eprintln!("{:?}", cycles);

    let blueprint = BlueprintString::constant_combinator(
        String::from("Topological Sort"),
        petgraph::algo::toposort(&graph, None)
            .unwrap()
            .iter()
            .copied()
            .filter(|name| craftable.contains(name))
            .enumerate()
            // .inspect(|(index, name)| eprintln!("{index}: {name}"))
            .map(|(index, name)| (String::from(name), index as i32)),
    )
    .encode();

    std::io::stdout().lock().write_all(&blueprint).unwrap();
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
