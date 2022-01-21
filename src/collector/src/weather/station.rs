use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StationResponse {
    pub key: String,
    pub updated: i64,
    pub title: String,
    pub summary: String,
    pub value_type: String,
    pub link: Vec<Link>,
    pub station_set: Vec<StationSet>,
    pub station: Vec<Station>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub rel: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StationSet {
    pub key: String,
    pub updated: i64,
    pub title: String,
    pub summary: String,
    pub link: Vec<Link2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link2 {
    pub rel: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Station {
    pub name: String,
    pub owner: String,
    pub owner_category: String,
    pub id: i64,
    pub height: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub active: bool,
    pub from: i64,
    pub to: i64,
    pub key: String,
    pub updated: i64,
    pub title: String,
    pub summary: String,
    pub link: Vec<Link3>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link3 {
    pub rel: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub href: String,
}
