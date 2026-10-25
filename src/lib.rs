use std::collections::HashMap;

use anyhow::{Result, bail};
use uuid::Uuid;

pub mod features;

pub struct Feature<T: Property> {
    id: FeatureId,
    pub geometry: Geometry,
    pub properties: T,
}

impl<T: Property> Feature<T> {
    pub fn id(&self) -> FeatureId {
        self.id
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeatureId(pub Uuid);

pub enum Geometry {
    Point(Point),
    Line(Line),
    Polygon,
}

pub struct Line {
    /// Ordered list of the nodes of the line.
    pub nodes: Vec<Uuid>,
}

pub struct LocalizedString {
    pub default: String,
    pub localizations: HashMap<language_tags::LanguageTag, String>
}

/// Represents a single point on the map.
pub struct Node {
    /// Id of the node.
    ///
    /// This field is private, as it shouldn't be editable from the outside.
    id: NodeId,
    /// Latitude of the node.
    pub lat: f64,
    /// Longitude of the node.
    pub lon: f64,
}

impl Node {
    /// Creates a new Node with a unique UUID.
    /// 
    /// # Errors
    /// ## Latitude
    /// The latitude MUST be greater or equal than -90.0 and less or equal than 90.0
    ///
    /// ## Longitude
    /// The longitude MUST be greater or equal than -180.0 and less or equal than 180.0
    pub fn new(lat: f64, lon: f64) -> Result<Self> {
        // Check if latitude is within bounds.
        if lat < -90.0 || lat > 90.0 {
            bail!("Latitude must be within -90 to 90 but is {}", lat)
        }
        // Check if longitude is within bounds.
        if lon < -180.0 || lon > 180.0 {
            bail!("Longitude must be within -180.0 to 180 but is {}", lon)
        }
        // All checks have passed successfully.

        Ok(Self {
            id: NodeId(Uuid::new_v4()),
            lat,
            lon,
        })
    }
    /// Get the id of the [Node].
    pub fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub Uuid);

/// A feature with just one node.
pub struct Point {
    pub node: Uuid,
}

/// Marker for types, that can be used as properties for features
pub trait Property {}
