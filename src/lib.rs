use std::collections::HashMap;

use anyhow::{Result, bail};
use uuid::Uuid;

pub mod features;
pub mod map;

/// A map feature with geometry and a typed set of properties.
pub struct Feature<T: Property> {
    id: FeatureId,
    pub geometry: Geometry,
    pub properties: T,
}

impl<T: Property> Feature<T> {
    /// Creates a feature with a unique ID, the given geometry, and properties.
    pub fn new(geometry: Geometry, properties: T) -> Self {
        Self {
            id: FeatureId(Uuid::new_v4()),
            geometry,
            properties,
        }
    }

    pub fn id(&self) -> FeatureId {
        self.id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeatureId(pub Uuid);

pub enum Geometry {
    Point(Point),
    Line(Line),
    Polygon(Polygon),
}

pub struct Line {
    /// Ordered list of the nodes of the line.
    pub nodes: Vec<NodeIndex>,
}

pub struct Polygon {
    /// Ordered rings of the polygon. The first ring is the exterior boundary.
    pub rings: Vec<Vec<NodeIndex>>,
}

pub struct LocalizedString {
    pub default: String,
    pub localizations: HashMap<language_tags::LanguageTag, String>,
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

/// Dense, map-local index of a node.
///
/// Node indexes are intended for hot-path geometry access. Use [`NodeId`] when
/// an identifier must remain stable outside a particular [`crate::map::Map`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeIndex(pub u32);

/// Dense index into one of the typed feature collections in a map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeatureIndex(pub u32);

/// A feature with just one node.
pub struct Point {
    pub node: NodeIndex,
}

/// Marker for types that can be used as properties for features.
///
/// Implement this trait for a struct containing the fields relevant to a
/// particular kind of feature, then use that struct as the type parameter of
/// [`Feature`].
pub trait Property {}

#[cfg(test)]
mod tests {
    use super::{Feature, Geometry, NodeIndex, Point, Property};

    struct PlaceProperties {
        name: String,
        population: u64,
    }
    impl Property for PlaceProperties {}

    #[test]
    fn creates_feature_with_typed_properties() {
        let feature = Feature::new(
            Geometry::Point(Point { node: NodeIndex(0) }),
            PlaceProperties {
                name: "Runar".to_owned(),
                population: 42,
            },
        );

        assert_ne!(
            feature.id(),
            Feature::new(
                Geometry::Point(Point { node: NodeIndex(0) }),
                PlaceProperties {
                    name: "Elsewhere".to_owned(),
                    population: 1,
                },
            )
            .id()
        );
        assert_eq!(feature.properties.name, "Runar");
        assert_eq!(feature.properties.population, 42);
    }
}
