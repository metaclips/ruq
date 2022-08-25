use super::processor::Processor;

#[derive(Debug, Clone)]
pub struct Toml {
    data: toml::Value,
}

impl Toml {
    pub fn new(data: &str) -> Self {
        let data = toml::from_str(data).unwrap();
        Toml { data }
    }

    #[allow(dead_code)]
    pub fn get_toml(&self) -> toml::Value {
        self.data.clone()
    }
}

impl Processor for Toml {
    type T = Toml;

    fn from_json(json_data: serde_json::Value) -> Self::T {
        let data = toml::to_string(&json_data).unwrap();
        Toml::new(&data)
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self.data.clone()).unwrap()
    }

    fn to_string(&self) -> String {
        self.data.to_string()
    }
}

#[test]
fn convert_json_to_toml() {
    let json_data = r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#;
    let json_marshalled_val = serde_json::from_str(json_data).unwrap();

    let toml = Toml::from_json(json_marshalled_val).get_toml();

    let toml_val: toml::Value = toml::from_str(
        r#"
        age = 43
        name = "John Doe"
        phones = ["+44 1234567", "+44 2345678"]
"#,
    )
    .unwrap();
    assert_eq!(toml_val, toml);
}

#[test]
fn convert_toml_to_json() {
    let toml_str = r#"age = 43
name = "John Doe"
phones = ["+44 1234567", "+44 2345678"]
"#;

    let toml = Toml::new(&toml_str);
    let json_data = toml.to_json();

    let json_val: serde_json::Value = serde_json::from_str(
        r#"
    {
        "name": "John Doe",
        "age": 43,
        "phones": [
            "+44 1234567",
            "+44 2345678"
        ]
    }"#,
    )
    .unwrap();
    assert_eq!(json_data, json_val)
}
