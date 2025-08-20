use std::collections::{BTreeMap};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Trace {
    pub packages: BTreeMap<String, PkgTrace>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PkgTrace {
    pub directory: String,

    #[serde(with = "trace_map_as_map")]
    pub maps: BTreeMap<TraceSrc, TraceDst>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TraceMap {
    pub src: TraceSrc,
    pub dst: TraceDst,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct TraceSrc(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct TraceDst(pub String);

mod trace_map_as_map {
    use serde::{Serializer, Deserializer};

    use super::*;

    pub fn serialize<S>(
        map: &BTreeMap<TraceSrc, TraceDst>,
        serializer: S
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let v: Vec<TraceMap> = map
            .iter()
            .map(|(src, dst)| TraceMap {src: src.clone(), dst: dst.clone()})
            .collect();
        v.serialize(serializer)
    }

    pub fn deserialize<'de, D>(
        deserializer: D
    ) -> Result<BTreeMap<TraceSrc, TraceDst>, D::Error>
    where
        D: Deserializer<'de>
    {
        let v = Vec::<TraceMap>::deserialize(deserializer)?;
        Ok(v.into_iter().map(|m| (m.src, m.dst)).collect())
    }
}

#[cfg(test)]
mod tests {
    use serde_test::{Token, assert_tokens};

    use super::*;

    #[test]
    fn pkg_trace_ser_de() {
        let trace = PkgTrace {
            directory: "test_dir".to_string(),
            maps: BTreeMap::from([
                (TraceSrc("src1".to_string()), TraceDst("dst1".to_string())),
                (TraceSrc("src2".to_string()), TraceDst("dst2".to_string())),
            ]),
        };

        assert_tokens(&trace, &[
            Token::Struct {
                name: "PkgTrace",
                len: 2,
            },

            Token::Str("directory"),
            Token::Str("test_dir"),

            Token::Str("maps"),
            Token::Seq { len: Some(2) },

            Token::Struct {
                name: "TraceMap",
                len: 2,
            },
            Token::Str("src"),
            Token::Str("src1"),
            Token::Str("dst"),
            Token::Str("dst1"),
            Token::StructEnd,

            Token::Struct {
                name: "TraceMap",
                len: 2,
            },
            Token::Str("src"),
            Token::Str("src2"),
            Token::Str("dst"),
            Token::Str("dst2"),
            Token::StructEnd,

            Token::SeqEnd,
            Token::StructEnd,
        ])
    }
}
