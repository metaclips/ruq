use super::processor::Processor;

#[derive(Debug, Clone)]
pub struct Yaml {
    data: serde_yaml::Value,
}

impl Yaml {
    pub fn new(data: String) -> Self {
        let data = serde_yaml::from_str(&data).unwrap();
        Yaml { data }
    }

    #[allow(dead_code)]
    pub fn get_yaml(&self) -> serde_yaml::Value {
        self.data.clone()
    }
}

impl Processor for Yaml {
    type T = Yaml;

    fn from_json(json_data: serde_json::Value) -> Self::T {
        let data = serde_yaml::to_string(&json_data).unwrap();
        Yaml::new(data)
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self.data.clone()).unwrap()
    }

    fn to_string(&self) -> String {
        serde_yaml::to_string(&self.data).unwrap()
    }
}

#[test]
fn convert_json_to_yaml() {
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

    let yaml = Yaml::from_json(json_marshalled_val).get_yaml();

    let yaml_val: serde_yaml::Value = serde_yaml::from_str(
        r#"
        age: 43
        name: "John Doe"
        phones: ["+44 1234567", "+44 2345678"]
"#,
    )
    .unwrap();
    assert_eq!(yaml_val, yaml);
}

#[test]
fn convert_toml_to_json() {
    let yaml_str = r#"age: 43
name: "John Doe"
phones: ["+44 1234567", "+44 2345678"]
"#;

    let yaml = Yaml::new(yaml_str.to_string());
    let json_data = yaml.to_json();

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
