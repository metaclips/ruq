pub trait Processor {
    type T;
    fn from_json(json_data: serde_json::Value) -> Self::T;
    fn to_json(&self) -> serde_json::Value;
    fn to_string(&self) -> String;
}
