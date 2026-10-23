// Source - https://stackoverflow.com/a/53130703
// Posted by Stargateur, modified by community. See post 'Timeline' for change history
// Retrieved 2026-10-23, License - CC BY-SA 4.0

use core::fmt;
use core::marker::PhantomData;

use serde::Deserialize;
use serde::de::Deserializer;
use serde::de::MapAccess;
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
