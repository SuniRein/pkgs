use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct Trace {
    pub packages: BTreeMap<String, PkgTrace>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PkgTrace {
    pub directory: String,

    #[serde(with = "trace_map_as_map")]
    pub maps: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TraceMap {
    pub src: String,
    pub dst: String,
}

mod trace_map_as_map {
    use serde::{Deserializer, Serializer};

    use super::*;

    pub fn serialize<S>(map: &BTreeMap<String, String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let v: Vec<TraceMap> = map
            .iter()
            .map(|(src, dst)| TraceMap {
                src: src.clone(),
                dst: dst.clone(),
            })
            .collect();
        v.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<BTreeMap<String, String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Vec::<TraceMap>::deserialize(deserializer)?;
        Ok(v.into_iter().map(|m| (m.src, m.dst)).collect())
    }
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_tokens, Token};

    use super::*;

    #[test]
    fn pkg_trace_ser_de() {
        let trace = PkgTrace {
            directory: "test_dir".to_string(),
            maps: BTreeMap::from([
                ("src1".to_string(), "dst1".to_string()),
                ("src2".to_string(), "dst2".to_string()),
            ]),
        };

        assert_tokens(
            &trace,
            &[
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
            ],
        )
    }
}
