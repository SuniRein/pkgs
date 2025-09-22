use std::borrow::Cow;
use std::collections::BTreeMap;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer, de};
use serde_json::Value as JsonValue;
use serde_json::json;

use super::de_map_as_vec::deserialize_map_as_vec;
use super::{GitPkg, Package, PackageType};

impl<'de> Deserialize<'de> for Package {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut map = BTreeMap::<String, JsonValue>::deserialize(deserializer)?;

        let mut take_str = |key: &str| -> Result<Option<String>, D::Error> {
            match map.remove(key) {
                Some(v) => {
                    let s = v.as_str().ok_or_else(|| {
                        de::Error::invalid_type(
                            de::Unexpected::Other(&format!("{key} not string")),
                            &"a string",
                        )
                    })?;
                    Ok(Some(s.to_string()))
                }
                None => Ok(None),
            }
        };

        let kind = take_str("kind")?;
        let kind = match kind.as_deref() {
            Some("local") | None => PackageType::Local,
            Some("git") => {
                let url = take_str("url")?.ok_or_else(|| de::Error::missing_field("url"))?;
                PackageType::Git(GitPkg { url })
            }
            Some(other) => return Err(de::Error::unknown_variant(other, &["local", "git"])),
        };

        let vars = match map.remove("vars") {
            Some(vars) => deserialize_map_as_vec(vars).map_err(de::Error::custom)?,
            None => vec![],
        };

        let maps = match map.remove("maps") {
            Some(maps) => deserialize_map_as_vec(maps).map_err(de::Error::custom)?,
            None => vec![],
        };

        if !map.is_empty() {
            let unexpected = map.keys().next().unwrap();
            return Err(de::Error::unknown_field(
                unexpected,
                &["kind", "vars", "maps", "url"],
            ));
        }

        Ok(Package { kind, vars, maps })
    }
}

impl JsonSchema for Package {
    fn schema_name() -> Cow<'static, str> {
        "Package".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "pkgs::config::Package".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        let string_object = json!({
            "type": "object",
            "additionalProperties": {
                "type": "string"
            },
            "default": {}
        });

        let definitions = generator.definitions_mut();
        definitions.insert("Maps".into(), string_object.clone());
        definitions.insert("Vars".into(), string_object);

        definitions.insert(
            "LocalPackage".into(),
            json!({
                "type": "object",
                "properties": {
                    "kind": { "const": "local" },
                    "maps": { "$ref": "#/$defs/Maps" },
                    "vars": { "$ref": "#/$defs/Vars" },
                },
                "additionalProperties": false,
            }),
        );

        definitions.insert(
            "GitPackage".into(),
            json!({
                "type": "object",
                "properties": {
                    "kind": { "const": "git" },
                    "url": { "type": "string" },
                    "maps": { "$ref": "#/$defs/Maps" },
                    "vars": { "$ref": "#/$defs/Vars" },
                },
                "required": ["kind", "url"],
                "additionalProperties": false,
            }),
        );

        json_schema!({
            "oneOf": [
                { "$ref": "#/$defs/LocalPackage" },
                { "$ref": "#/$defs/GitPackage" },
            ]
        })
    }
}
