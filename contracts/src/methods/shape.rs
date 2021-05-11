use geojson::{Feature, Geometry};
use serde_json::{Map, Value};
use std::collections::HashMap;
use uuid::Uuid;

pub use geojson;

pub mod add_shape;
pub mod add_shape_tag;
pub mod add_shapes;
pub mod delete_shape;
pub mod delete_shape_tag;
pub mod get_nearby_shapes;
pub mod get_shape;
pub mod search_shapes_by_tags;

#[derive(serde::Serialize, Clone, Debug, serde::Deserialize)]
#[non_exhaustive]
pub struct Shape {
    pub id: Uuid,
    pub name: Option<String>,
    pub geo: Geometry,
    pub tags: HashMap<String, String>,
}

impl Shape {
    pub fn new(
        id: Uuid,
        name: Option<String>,
        geo: Geometry,
        tags: HashMap<String, String>,
    ) -> Self {
        Self {
            id,
            name,
            geo,
            tags,
        }
    }

    pub fn coordinates(&self) -> Vec<Coord> {
        coordinates_in_geo(&self.geo)
    }
}

pub struct Coord {
    pub lat: f64,
    pub lon: f64,
}

impl Coord {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }
}

pub fn coordinates_in_geo(geom: &Geometry) -> Vec<Coord> {
    match &geom.value {
        geojson::Value::Point(p) => vec![Coord::new(p[1], p[0])],
        geojson::Value::MultiPoint(mp) => mp.iter().map(|p| Coord::new(p[1], p[0])).collect(),
        geojson::Value::LineString(ls) => ls.iter().map(|p| Coord::new(p[1], p[0])).collect(),
        geojson::Value::MultiLineString(mls) => mls
            .iter()
            .flat_map(|ls| ls.iter().map(|p| Coord::new(p[1], p[0])))
            .collect(),
        geojson::Value::Polygon(poly) => poly
            .iter()
            .flat_map(|ls| ls.iter().map(|p| Coord::new(p[1], p[0])))
            .collect(),
        geojson::Value::MultiPolygon(m_poly) => m_poly
            .iter()
            .flatten()
            .flatten()
            .map(|p| Coord::new(p[1], p[0]))
            .collect(),
        geojson::Value::GeometryCollection(g) => {
            g.iter().flat_map(|g| coordinates_in_geo(&g)).collect()
        }
    }
}

impl From<Shape> for Feature {
    fn from(s: Shape) -> Self {
        let mut properties = Map::new();
        properties.insert(
            "name".to_string(),
            match s.name {
                Some(name) => Value::String(name),
                None => Value::Null,
            },
        );
        for (name, value) in s.tags {
            properties.insert(name, Value::String(value));
        }
        Feature {
            bbox: None,
            geometry: Some(s.geo),
            id: None,
            properties: Some(properties),
            foreign_members: None,
        }
    }
}
