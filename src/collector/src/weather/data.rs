use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataResponse {
    pub value: Vec<Value>,
    pub updated: i64,
    pub parameter: Parameter,
    pub station: Station,
    pub period: Period,
    pub position: Vec<Position>,
    pub link: Vec<Link>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value {
    pub date: u128,
    pub value: String,
    pub quality: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub key: String,
    pub name: String,
    pub summary: String,
    pub unit: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Station {
    pub key: String,
    pub name: String,
    pub owner: String,
    pub owner_category: String,
    pub height: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Period {
    pub key: String,
    pub from: i64,
    pub to: i64,
    pub summary: String,
    pub sampling: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub from: i64,
    pub to: i64,
    pub height: f64,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub rel: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub href: String,
}
