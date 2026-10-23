use core::marker::PhantomData;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

use serde::Deserialize;
use serde_json::Value;

use automall::deserialize_item_map;
use automall::deserialize_weird_vec;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let mut buffer = Vec::new();
    File::open(path)
        .map(BufReader::new)
        .map(flate2::read::GzDecoder::new)
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();

    let prototypes = serde_json::from_slice::<Data>(&buffer).unwrap();
    dbg!(prototypes);
}

#[derive(Debug, Deserialize)]
struct Data<'a> {
    #[serde(borrow, deserialize_with = "deserialize_item_map")]
    recipe: Vec<(&'a str, Recipe<'a>)>,
}

#[derive(Debug, Deserialize)]
struct Recipe<'a> {
    #[serde(borrow, deserialize_with = "deserialize_weird_vec")]
    ingredients: Vec<Ingredient<'a>>,
}

#[derive(Debug, Deserialize)]
struct Ingredient<'a> {
    // #[serde(skip)]
    // _a: PhantomData<&'a ()>,
    r#type: &'a str,
    name: &'a str,
}
