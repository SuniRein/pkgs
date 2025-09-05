use std::fmt::{self, Formatter};
use std::marker::PhantomData;

use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

pub fn deserialize_map_as_vec<'de, D, K, V>(deserializer: D) -> Result<Vec<(K, V)>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    struct MapAsVecVisitor<K, V> {
        marker: PhantomData<Vec<(K, V)>>,
    }

    impl<'de, K, V> Visitor<'de> for MapAsVecVisitor<K, V>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        type Value = Vec<(K, V)>;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            formatter.write_str("a map to be deserialized as a vector of key-value pairs")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut vec = Vec::with_capacity(access.size_hint().unwrap_or(0));
            while let Some((key, value)) = access.next_entry()? {
                vec.push((key, value));
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_map(MapAsVecVisitor {
        marker: PhantomData,
    })
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;
    use crate::test_utils::prelude::*;

    #[derive(Debug, Deserialize)]
    struct My {
        #[serde(deserialize_with = "deserialize_map_as_vec")]
        items: Vec<(String, i32)>,
    }

    #[gtest]
    fn it_works() -> Result<()> {
        let toml = indoc! {r#"
            [items]
            a = 1
            d = 4
            g = -1
            b = 2
            c = 3
        "#};

        let my: My = toml::from_str(toml)?;

        expect_eq!(
            my.items,
            [
                ("a".into(), 1),
                ("d".into(), 4),
                ("g".into(), -1),
                ("b".into(), 2),
                ("c".into(), 3),
            ]
        );

        Ok(())
    }
}
