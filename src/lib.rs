// Source - https://stackoverflow.com/a/53130703
// Posted by Stargateur, modified by community. See post 'Timeline' for change history
// Retrieved 2026-10-23, License - CC BY-SA 4.0

use core::fmt;
use core::fmt::Debug;
use core::marker::PhantomData;

use serde::Deserialize;
use serde::de::Deserializer;
use serde::de::Error as _;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;

struct ItemMapVisitor<T> {
    _type: PhantomData<T>,
}

impl<T> ItemMapVisitor<T> {
    fn new() -> Self {
        Self { _type: PhantomData }
    }
}

impl<'de, T> Visitor<'de> for ItemMapVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = Vec<(&'de str, T)>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("name: ")?;
        formatter.write_str(core::any::type_name::<T>())
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut items = Vec::with_capacity(access.size_hint().unwrap_or(0));
        while let Some((key, value)) = access.next_entry::<&'de str, T>()? {
            items.push((key, value));
        }
        Ok(items)
    }
}

pub fn deserialize_item_map<'de, D, T>(deserializer: D) -> Result<Vec<(&'de str, T)>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    deserializer.deserialize_map(ItemMapVisitor::new())
}

struct EmptyVecVisitor<T> {
    _type: PhantomData<T>,
}

impl<T> EmptyVecVisitor<T> {
    fn new() -> Self {
        Self { _type: PhantomData }
    }
}

impl<'de, T> Visitor<'de> for EmptyVecVisitor<T>
where
    T: Deserialize<'de>,
    T: Debug,
{
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("sequence: ")?;
        formatter.write_str(core::any::type_name::<T>())
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let None = access.next_entry::<&'de str, T>()? else {
            return Err(M::Error::custom("expected empty map"));
        };
        Ok(Vec::new())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut buffer = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(item) = seq.next_element::<T>()? {
            buffer.push(item);
        }
        Ok(buffer)
    }
}

pub fn deserialize_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Debug,
{
    deserializer.deserialize_any(EmptyVecVisitor::new())
}
