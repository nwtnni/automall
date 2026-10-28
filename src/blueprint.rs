//! https://wiki.factorio.com/Blueprint_string_format

use core::num::NonZeroU8;
use core::num::NonZeroU16;
use std::io::Read as _;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
pub struct BlueprintString<'a> {
    #[serde(borrow)]
    blueprint: Blueprint<'a>,
}

impl<'a> BlueprintString<'a> {
    pub fn constant_combinator(sections: Vec<LogisticSection<'a>>) -> Self {
        Self {
            blueprint: Blueprint {
                entities: vec![Entity {
                    entity_number: 1,
                    name: "constant-combinator",
                    position: Position { x: 0.5, y: 0.5 },
                    control_behavior: Some(ControlBehavior {
                        sections: Some(LogisticSections { sections }),
                    }),
                }],
                version: 0,
            },
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = vec![b'0'];
        serde_json::ser::to_writer(
            flate2::write::ZlibEncoder::new(
                base64::write::EncoderWriter::new(
                    &mut buffer,
                    &base64::engine::general_purpose::STANDARD,
                ),
                flate2::Compression::best(),
            ),
            self,
        )
        .unwrap();
        buffer
    }

    // Tested on https://factorioprints.com/view/-Kl-a5-7hM9UdJQEYzjU
    pub fn with_decode<F, T>(string: &'a str, with: F) -> T
    where
        for<'b> F: FnOnce(BlueprintString<'b>) -> T,
    {
        assert_eq!(string.as_bytes()[0], b'0');

        let mut buffer = String::new();
        flate2::read::ZlibDecoder::new(base64::read::DecoderReader::new(
            &string.as_bytes()[1..],
            &base64::engine::general_purpose::STANDARD,
        ))
        .read_to_string(&mut buffer)
        .unwrap();
        let blueprint = serde_json::de::from_str(&buffer).unwrap();
        with(blueprint)
    }
}

/// https://wiki.factorio.com/Blueprint_string_format#Blueprint_object
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "item", rename = "blueprint")]
pub struct Blueprint<'a> {
    #[serde(borrow)]
    entities: Vec<Entity<'a>>,
    version: u64,
}

/// https://wiki.factorio.com/Blueprint_string_format#Entity_object
#[derive(Debug, Deserialize, Serialize)]
pub struct Entity<'a> {
    entity_number: usize,
    name: &'a str,
    position: Position,
    #[serde(borrow)]
    control_behavior: Option<ControlBehavior<'a>>,
}

/// https://wiki.factorio.com/Blueprint_string_format#Control_behavior_object
#[derive(Debug, Deserialize, Serialize)]
pub struct ControlBehavior<'a> {
    #[serde(borrow)]
    sections: Option<LogisticSections<'a>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogisticSections<'a> {
    #[serde(borrow)]
    sections: Vec<LogisticSection<'a>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogisticSection<'a> {
    pub index: NonZeroU8,
    #[serde(borrow)]
    pub filters: Vec<LogisticFilter<'a>>,
    #[serde(borrow, skip_serializing_if = "Option::is_none")]
    pub group: Option<&'a str>,
    #[serde(default = "default_1")]
    pub multiplier: f32,
}

/// https://wiki.factorio.com/Blueprint_string_format#Logistic_filter_object
#[derive(Debug, Deserialize, Serialize)]
pub struct LogisticFilter<'a> {
    pub index: NonZeroU16,
    pub name: &'a str,
    pub count: i32,
    pub quality: &'a str,
    pub comparator: &'a str,
}

impl Default for LogisticFilter<'_> {
    fn default() -> Self {
        Self {
            index: NonZeroU16::MIN,
            name: "",
            count: 0,
            quality: "normal",
            comparator: "=",
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Position {
    x: f32,
    y: f32,
}

fn default_1() -> f32 {
    1.0
}
