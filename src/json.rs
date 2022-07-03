struct JSON {
    data: serde_json::Value,
}

impl JSON {
    pub fn new(data: &str) -> Self {
        let data = serde_json::from_str(data).unwrap();
        Self { data }
    }
}
