use assert_cmd::{Command, cargo::cargo_bin};
use googletest::prelude::*;
use indoc::indoc;
use jsonschema::validate;
use rstest::*;
use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

#[fixture]
fn schema() -> JsonValue {
    let result = Command::new(cargo_bin!("pkgs")).arg("schema").unwrap();
    serde_json::from_slice(&result.stdout).unwrap()
}

#[fixture]
fn content() -> String {
    indoc! {r#"
        [vars]
        MY_VAR = "Hello"
        ANOTHER_VAR = "World"

        [packages.a]
        kind = "local"

        [packages.a.vars]
        local_var = "Local"

        [packages.a.maps]
        src_file = "dst_file"
        "path/to/src_dir" = "path/to/dst_dir"
        "a.with_ext" = "b.with_ext"

        [packages."empty maps"]
    "#}
    .to_string()
}

fn read_toml(str: &str) -> Result<JsonValue> {
    let toml_val: TomlValue = toml::from_str(str)?;
    Ok(serde_json::to_value(&toml_val)?)
}

#[rstest]
#[gtest]
fn parse_correct_file(schema: JsonValue, content: String) -> Result<()> {
    let value = read_toml(&content)?;
    assert_that!(validate(&schema, &value), ok(anything()));
    Ok(())
}

#[rstest]
#[gtest]
fn kind_can_omit(schema: JsonValue, mut content: String) -> Result<()> {
    content = content.replace("[packages.a]\nkind = \"local\"\n", "");

    let value = read_toml(&content)?;
    assert_that!(validate(&schema, &value), ok(anything()));
    Ok(())
}

#[rstest]
#[case("vars")]
#[case("packages.a")]
#[case("packages.a.vars")]
#[case("packages.a.maps")]
#[gtest]
fn some_field_can_omit(schema: JsonValue, mut content: String, #[case] field: &str) -> Result<()> {
    let start = content.find(&format!("[{field}]")).unwrap();
    let end = content[start..].find("\n\n").unwrap();
    content.replace_range(start..start + end + 1, "");

    let value = read_toml(&content)?;
    assert_that!(validate(&schema, &value), ok(anything()));
    Ok(())
}

#[rstest]
#[case("kind", "type")]
#[case("packages.a.vars", "packages.a.var")]
#[case("packages.a.maps", "packages.a.map")]
#[gtest]
fn unknown_fields(
    schema: JsonValue,
    mut content: String,
    #[case] from: &str,
    #[case] to: &str,
) -> Result<()> {
    content = content.replace(from, to);

    let value = read_toml(&content)?;
    assert_that!(validate(&schema, &value), err(anything()));
    Ok(())
}
