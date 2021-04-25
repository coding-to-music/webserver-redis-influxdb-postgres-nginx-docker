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
        geojson::Value::Point(p) => {
            let coords = vec![Coord::new(p[1], p[0])];
            coords
        }
        geojson::Value::MultiPoint(mp) => {
            let coords = mp.into_iter().map(|p| Coord::new(p[1], p[0])).collect();
            coords
        }
        geojson::Value::LineString(ls) => {
            let coords = ls.into_iter().map(|p| Coord::new(p[1], p[0])).collect();
            coords
        }
        geojson::Value::MultiLineString(mls) => {
            let coords = mls
                .into_iter()
                .flat_map(|ls| ls.into_iter().map(|p| Coord::new(p[1], p[0])))
                .collect();
            coords
        }
        geojson::Value::Polygon(poly) => {
            let coords = poly
                .into_iter()
                .flat_map(|ls| ls.into_iter().map(|p| Coord::new(p[1], p[0])))
                .collect();
            coords
        }
        geojson::Value::MultiPolygon(m_poly) => {
            let coords = m_poly
                .into_iter()
                .flatten()
                .flatten()
                .map(|p| Coord::new(p[1], p[0]))
                .collect();
            coords
        }
        geojson::Value::GeometryCollection(g) => {
            g.into_iter().flat_map(|g| coordinates_in_geo(&g)).collect()
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
        let feature = Feature {
            bbox: None,
            geometry: Some(s.geo),
            id: None,
            properties: Some(properties),
            foreign_members: None,
        };
        feature
    }
}
