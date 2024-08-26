use crate::DemesError;

fn fix_null_start_times_demes(value: &mut serde_json::Value) -> Result<(), DemesError> {
    if let serde_json::Value::Array(demes) = value {
        // JSON has no encoding for infinity or NaN.
        // If an input file contains .inf, the JSON reader
        // will convert it to null, which is what the JSON
        // spec requires (AFAIK).
        // However, that encoding violates the demes spec,
        // so we replace those values with a string that we
        // can interpret.
        for deme in demes {
            if let Some(value) = deme.get_mut("start_time") {
                if let &mut serde_json::Value::Null = value {
                    let inf: &str = "Infinity";
                    *value = serde_json::Value::from(inf);
                }
            }
        }
    }
    Ok(())
}

fn fix_null_start_times(
    input: std::collections::HashMap<String, serde_json::Value>,
) -> Result<std::collections::HashMap<String, serde_json::Value>, DemesError> {
    let mut input = input;
    if let Some(demes) = input.get_mut("demes") {
        fix_null_start_times_demes(demes)?;
    }
    Ok(input)
}

pub fn fix_json_input(
    input: std::collections::HashMap<String, serde_json::Value>,
) -> Result<std::collections::HashMap<String, serde_json::Value>, DemesError> {
    let input = fix_null_start_times(input)?;
    Ok(input)
}

#[test]
fn test_json_conversion() {
    use std::io::Read;

    let mut f =
        std::fs::File::open("demes-spec/test-cases/valid/defaults_deme_ancestors.yaml").unwrap();
    let mut buf = String::new();
    let _ = f.read_to_string(&mut buf).unwrap();
    let json: serde_json::Value = serde_yaml::from_str::<serde_json::Value>(&buf).unwrap();
    let json = json.to_string();
    let json: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&json).unwrap();
    let json = fix_json_input(json).unwrap();
    let json = serde_json::to_string(&json).unwrap();
    let g = crate::loads_json(&json).unwrap();
    let f =
        std::fs::File::open("demes-spec/test-cases/valid/defaults_deme_ancestors.yaml").unwrap();
    let gy = crate::load(f).unwrap();
    assert_eq!(g, gy);
}
