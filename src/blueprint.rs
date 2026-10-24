//! https://wiki.factorio.com/Blueprint_string_format

use core::num::NonZeroUsize;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
pub struct BlueprintString {
    blueprint: Blueprint,
}

impl BlueprintString {
    pub fn constant_combinator(
        label: String,
        signals: impl IntoIterator<Item = (String, i32)>,
    ) -> Self {
        Self {
            blueprint: Blueprint {
                label,
                entities: vec![Entity {
                    entity_number: 1,
                    name: String::from("constant-combinator"),
                    position: Position { x: 0.0, y: 0.0 },
                    control_behavior: Some(ControlBehavior {
                        filters: signals
                            .into_iter()
                            .enumerate()
                            .map(|(index, (name, count))| Filter {
                                signal: Signal {
                                    r#type: String::from("item"),
                                    name,
                                },
                                index: NonZeroUsize::new(index + 1).unwrap(),
                                count,
                            })
                            .collect(),
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
    pub fn decode(string: &str) -> Self {
        assert_eq!(string.as_bytes()[0], b'0');
        serde_json::de::from_reader(flate2::read::ZlibDecoder::new(
            base64::read::DecoderReader::new(
                &string.as_bytes()[1..],
                &base64::engine::general_purpose::STANDARD,
            ),
        ))
        .unwrap()
    }
}

/// https://wiki.factorio.com/Blueprint_string_format#Blueprint_object
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "item", rename = "blueprint")]
pub struct Blueprint {
    label: String,
    entities: Vec<Entity>,
    version: u64,
}

/// https://wiki.factorio.com/Blueprint_string_format#Entity_object
#[derive(Debug, Deserialize, Serialize)]
pub struct Entity {
    entity_number: usize,
    name: String,
    position: Position,
    control_behavior: Option<ControlBehavior>,
}

/// https://wiki.factorio.com/Blueprint_string_format#Control_behavior_object
#[derive(Debug, Deserialize, Serialize)]
pub struct ControlBehavior {
    #[serde(default)]
    filters: Vec<Filter>,
}

/// https://wiki.factorio.com/Blueprint_string_format#Logistic_filter_object
#[derive(Debug, Deserialize, Serialize)]
pub struct Filter {
    signal: Signal,
    index: NonZeroUsize,
    count: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Signal {
    r#type: String,
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Position {
    x: f32,
    y: f32,
}
