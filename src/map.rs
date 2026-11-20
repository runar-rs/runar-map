use std::collections::HashMap;

use anyhow::{Result, bail};

use crate::{
    Feature, FeatureId, FeatureIndex, Geometry, Line, Node, NodeId, NodeIndex, Polygon,
    features::{LandUse, Road},
};

pub struct Map {
    nodes: Vec<Node>,
    roads: Vec<Feature<Road>>,
    land_uses: Vec<Feature<LandUse>>,
    
    /// Maps node IDs to their index in the `nodes` vector.
    node_ids: HashMap<NodeId, NodeIndex>,

    /// Maps feature IDs to their location in the map.
    feature_ids: HashMap<FeatureId, FeatureLocation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureLocation {
    Road(FeatureIndex),
    LandUse(FeatureIndex),
}

impl Map {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: Node) -> NodeIndex {
        let index = NodeIndex(
            u32::try_from(self.nodes.len()).expect("a map cannot contain more than u32::MAX nodes"),
        );
        self.node_ids.insert(node.id(), index);
        self.nodes.push(node);
        index
    }

    pub fn node(&self, index: NodeIndex) -> Option<&Node> {
        self.nodes.get(index.0 as usize)
    }

    pub fn node_index(&self, id: NodeId) -> Option<NodeIndex> {
        self.node_ids.get(&id).copied()
    }

    pub fn add_road(&mut self, road: Feature<Road>) -> Result<FeatureIndex> {
        let geometry = match &road.geometry {
            Geometry::Line(line) => line,
            _ => bail!("a road feature must have line geometry"),
        };
        self.validate_line(geometry)?;

        let index = self.next_feature_index(self.roads.len())?;
        let id = road.id();
        self.roads.push(road);
        self.feature_ids.insert(id, FeatureLocation::Road(index));
        Ok(index)
    }

    pub fn add_land_use(&mut self, land_use: Feature<LandUse>) -> Result<FeatureIndex> {
        let geometry = match &land_use.geometry {
            Geometry::Polygon(polygon) => polygon,
            _ => bail!("a land-use feature must have polygon geometry"),
        };
        self.validate_polygon(geometry)?;

        let index = self.next_feature_index(self.land_uses.len())?;
        let id = land_use.id();
        self.land_uses.push(land_use);
        self.feature_ids.insert(id, FeatureLocation::LandUse(index));
        Ok(index)
    }

    pub fn feature_location(&self, id: FeatureId) -> Option<FeatureLocation> {
        self.feature_ids.get(&id).copied()
    }

    fn validate_line(&self, line: &Line) -> Result<()> {
        self.validate_nodes(line.nodes.iter().copied())
    }

    fn validate_polygon(&self, polygon: &Polygon) -> Result<()> {
        self.validate_nodes(polygon.rings.iter().flat_map(|ring| ring.iter().copied()))
    }

    fn validate_nodes(&self, indexes: impl Iterator<Item = NodeIndex>) -> Result<()> {
        for index in indexes {
            if self.node(index).is_none() {
                bail!("geometry references missing node index {}", index.0);
            }
        }
        Ok(())
    }

    fn next_feature_index(&self, len: usize) -> Result<FeatureIndex> {
        Ok(FeatureIndex(u32::try_from(len).map_err(|_| {
            anyhow::anyhow!("feature collection is too large")
        })?))
    }
}

impl Default for Map {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            roads: Vec::new(),
            land_uses: Vec::new(),
            node_ids: HashMap::new(),
            feature_ids: HashMap::new(),
        }
    }
}
