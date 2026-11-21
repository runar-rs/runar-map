use std::collections::BTreeMap;
use std::io::Write;
use std::num::NonZeroU64;

use anyhow::{Result, bail};
use memmap2::MmapOptions;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    Feature, FeatureId, FeatureIndex, Geometry, Line, Node, NodeId, NodeIndex, Polygon,
    features::{LandUse, Road},
};

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum ItemType {
    Node,
    Road,
    LandUse,
    Restriction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContainerID {
    pub id: Uuid,
}

impl ContainerID {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self { id: uuid }
    }
}

pub struct MapContainerIndex {
    /// Map of Map Element IDs to their index and the number of uses.
    pub id_map: BTreeMap<Uuid, MapContainerIndexEntry>,
}
impl MapContainerIndex {
    pub fn new() -> Self {
        Self {
            id_map: BTreeMap::new(),
        }
    }

    pub fn increment_use(&mut self, id: Uuid) -> Result<()> {
        if let Some(entry) = self.id_map.get_mut(&id) {
            entry.uses += 1;
            Ok(())
        } else {
            // Return error, as element isn't in the file.
            bail!("Element with ID {} not found in map container index", id);
        }
    }
}

pub struct MapContainerIndexEntry {
    pub container_id: ContainerID,
    pub uses: u32,
    pub offset: Option<NonZeroU64>,
    pub item_type: ItemType,
    pub is_loaded: bool,
}

pub struct Map {
    container_offsets: BTreeMap<ContainerID, u64>,
    /// Index of map containers that belong to this map.
    item_index: MapContainerIndex,

    version: semver::Version,
    /// Path to the map file, if it was loaded from a file.
    path: Option<std::path::PathBuf>,

    nodes: RwLock<Vec<Option<Node>>>,
    roads: RwLock<Vec<Feature<Road>>>,
    land_uses: RwLock<Vec<Feature<LandUse>>>,

    /// Maps node IDs to their index in the `nodes` vector.
    node_ids: RwLock<BTreeMap<NodeId, NodeIndex>>,

    /// Maps feature IDs to their location in the map.
    feature_ids: RwLock<BTreeMap<FeatureId, FeatureLocation>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureLocation {
    Road(FeatureIndex),
    LandUse(FeatureIndex),
}

impl Map {
    /// Creates a new empty map.
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add_node(&self, node: Node) -> NodeIndex {
        let mut nodes = self.nodes.write().await;
        let index = NodeIndex(
            u32::try_from(nodes.len()).expect("a map cannot contain more than u32::MAX nodes"),
        );
        self.node_ids.write().await.insert(node.id(), index);
        nodes.push(Some(node));
        index
    }

    /// Returns a copy of the node at the given index, or `None` if it is not loaded.
    pub async fn node(&self, index: NodeIndex) -> Option<Node> {
        self.nodes
            .read()
            .await
            .get(index.0 as usize)
            .and_then(Option::as_ref)
            .cloned()
    }

    pub async fn node_index(&self, id: NodeId) -> Option<NodeIndex> {
        self.node_ids.read().await.get(&id).copied()
    }

    pub async fn add_road(&self, road: Feature<Road>) -> Result<FeatureIndex> {
        let geometry = match &road.geometry {
            Geometry::Line(line) => line,
            _ => bail!("a road feature must have line geometry"),
        };
        self.validate_line(geometry).await?;

        let mut roads = self.roads.write().await;
        let index = self.next_feature_index(roads.len())?;
        let id = road.id();
        roads.push(road);
        self.feature_ids
            .write()
            .await
            .insert(id, FeatureLocation::Road(index));
        Ok(index)
    }

    pub async fn add_land_use(&self, land_use: Feature<LandUse>) -> Result<FeatureIndex> {
        let geometry = match &land_use.geometry {
            Geometry::Polygon(polygon) => polygon,
            _ => bail!("a land-use feature must have polygon geometry"),
        };
        self.validate_polygon(geometry).await?;

        let mut land_uses = self.land_uses.write().await;
        let index = self.next_feature_index(land_uses.len())?;
        let id = land_use.id();
        land_uses.push(land_use);
        self.feature_ids
            .write()
            .await
            .insert(id, FeatureLocation::LandUse(index));
        Ok(index)
    }

    pub async fn feature_location(&self, id: FeatureId) -> Option<FeatureLocation> {
        self.feature_ids.read().await.get(&id).copied()
    }

    /// Check if all nodes referenced by the given line exist in the map.
    async fn validate_line(&self, line: &Line) -> Result<()> {
        self.validate_nodes(line.nodes.iter().copied()).await
    }

    /// Check if all nodes referenced by the given polygon exist in the map.
    async fn validate_polygon(&self, polygon: &Polygon) -> Result<()> {
        self.validate_nodes(polygon.rings.iter().flat_map(|ring| ring.iter().copied()))
            .await
    }

    /// Check if all nodes referenced by the given iterator exist in the map.
    async fn validate_nodes(&self, indexes: impl Iterator<Item = NodeIndex>) -> Result<()> {
        let nodes = self.nodes.read().await;
        for index in indexes {
            if nodes
                .get(index.0 as usize)
                .and_then(Option::as_ref)
                .is_none()
            {
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
            container_offsets: BTreeMap::new(),
            item_index: MapContainerIndex::new(),
            version: semver::Version::parse(env!("CARGO_PKG_VERSION"))
                .expect("invalid version number"),
            path: None,
            nodes: RwLock::new(Vec::new()),
            roads: RwLock::new(Vec::new()),
            land_uses: RwLock::new(Vec::new()),
            node_ids: RwLock::new(BTreeMap::new()),
            feature_ids: RwLock::new(BTreeMap::new()),
        }
    }
}

pub const MAP_MAGIC: &[u8; 8] = b"RUNARMAP";

/// Memory operations for Map.
impl Map {
    /// Goes through the map and unloads any data that hasn't recently been used.
    pub async fn trim(&self) {
        let unused_node_ids: Vec<_> = self
            .item_index
            .id_map
            .iter()
            .filter(|(_, entry)| {
                entry.item_type == ItemType::Node && entry.is_loaded && entry.uses == 0
            })
            .map(|(id, _)| NodeId(*id))
            .collect();
        // This waits for the write lock on nodes and node_ids, so it should be done after the read
        // locks on the map items have been released.
        let mut nodes = self.nodes.write().await;
        let mut node_ids = self.node_ids.write().await;
        for node_id in unused_node_ids {
            if let Some(index) = node_ids.remove(&node_id) {
                if let Some(node) = nodes.get_mut(index.0 as usize) {
                    *node = None;
                }
            }
        }
        // Compact the nodes vector to remove any None entries.
        nodes.retain(|node| node.is_some());
    }
    /// Loads a map from the given path. This includes loading the map's index and other metadata,
    /// but not the actual map data.
    /// 
    /// The file format is a compacted format, that only includes a snapshot of the map.
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        // Read the first 28 bytes of the file to check the magic number and version. This reads
        // until number_of_containers, which is the last field which offset is known at compile
        // time.
        let header = unsafe { MmapOptions::new().len(28).map(&file)? };
        if !header.starts_with(MAP_MAGIC) {
            bail!("invalid map file: missing magic number");
        }

        // Read the version number from the file and check if it is compatible with the current version.
        let version =
            semver::Version::parse(env!("CARGO_PKG_VERSION")).expect("invalid version number");
        let major = u32::from_le_bytes(header[8..12].try_into().unwrap()) as u64;
        let minor = u32::from_le_bytes(header[12..16].try_into().unwrap()) as u64;
        let patch = u32::from_le_bytes(header[16..20].try_into().unwrap()) as u64;
        let file_version = semver::Version::new(major, minor, patch);
        if file_version.major != version.major || file_version.minor != version.minor {
            bail!(
                "Incompatible map version: {}.{}.{} (expected {}.{}.{})",
                major,
                minor,
                patch,
                version.major,
                version.minor,
                version.patch
            );
        }

        // Start populating the map from the file.
        let mut map = Self::default();
        map.version = file_version;
        map.path = Some(path.to_path_buf());

        // Read the number of containers from the file.
        let number_of_containers = u32::from_le_bytes(header[20..24].try_into().unwrap());

        // Read the container offsets from the file. Each container offset is 24 bytes long.
        if number_of_containers > 0 {
            let container_header = unsafe {
                MmapOptions::new()
                    .offset(28)
                    .len((number_of_containers as usize) * 24)
                    .map(&file)?
            };

            for i in 0..number_of_containers as usize {
                let container_uuid = ContainerID::from_uuid(Uuid::from_bytes_le(
                    container_header[(24 * i)..(24 * i + 16)]
                        .try_into()
                        .unwrap(),
                ));
                let container_offset = u64::from_le_bytes(
                    container_header[(24 * i + 16)..(24 * i + 24)]
                        .try_into()
                        .unwrap(),
                );
                map.container_offsets
                    .insert(container_uuid, container_offset);
            }
        }

        let item_header = unsafe {
            MmapOptions::new()
                .offset(28 + (number_of_containers as u64) * 24)
                .len(8)
                .map(&file)?
        };

        let number_of_items = u64::from_le_bytes(item_header[0..8].try_into().unwrap());

        if number_of_items > 0 {
            let item_map = unsafe {
                MmapOptions::new()
                    .offset(28 + (number_of_containers as u64) * 24 + 8)
                    .len((number_of_items as usize) * 42)
                    .map(&file)?
            };

            for i in 0..number_of_items as usize {
                let item_id =
                    Uuid::from_bytes_le(item_map[(42 * i)..(42 * i + 16)].try_into().unwrap());

                let container_id = ContainerID::from_uuid(Uuid::from_bytes_le(
                    item_map[(42 * i + 16)..(42 * i + 32)].try_into().unwrap(),
                ));

                map.item_index.id_map.insert(
                    item_id,
                    MapContainerIndexEntry {
                        container_id,
                        uses: 0,
                        offset: NonZeroU64::new(u64::from_le_bytes(
                            item_map[(42 * i + 32)..(42 * i + 40)].try_into().unwrap(),
                        )),
                        item_type: ItemType::try_from(u16::from_le_bytes(
                            item_map[42 * i + 40..42 * i + 42].try_into().unwrap(),
                        ))?,
                        is_loaded: false,
                    },
                );
            }
        }

        Ok(map)
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = &self.path {
            self.save_to_path(path)
        } else {
            bail!("Map contains no path, so it cannot be saved. Use `save_to_path` instead.");
        }
    }

    pub fn save_to_path(&self, path: &std::path::Path) -> Result<()> {
        let number_of_containers = u64::try_from(self.container_offsets.len())
            .map_err(|_| anyhow::anyhow!("map contains too many containers"))?;
        let major = u32::try_from(self.version.major)
            .map_err(|_| anyhow::anyhow!("map major version does not fit in the file format"))?;
        let minor = u32::try_from(self.version.minor)
            .map_err(|_| anyhow::anyhow!("map minor version does not fit in the file format"))?;
        let patch = u32::try_from(self.version.patch)
            .map_err(|_| anyhow::anyhow!("map patch version does not fit in the file format"))?;

        let mut file = std::fs::File::create(path)?;
        file.write_all(MAP_MAGIC)?;
        file.write_all(&major.to_le_bytes())?;
        file.write_all(&minor.to_le_bytes())?;
        file.write_all(&patch.to_le_bytes())?;
        file.write_all(&number_of_containers.to_le_bytes())?;

        for (container_id, offset) in &self.container_offsets {
            file.write_all(&container_id.id.to_bytes_le())?;
            file.write_all(&offset.to_le_bytes())?;
        }

        // Write the number of items in the map.
        let number_of_items = u64::try_from(self.item_index.id_map.len())
            .map_err(|_| anyhow::anyhow!("map contains too many items"))?;
        file.write_all(&number_of_items.to_le_bytes())?;

        // Write the item index to the file.
        for (item_id, entry) in &self.item_index.id_map {
            file.write_all(&item_id.to_bytes_le())?;
            file.write_all(&entry.container_id.id.to_bytes_le())?;
            file.write_all(&entry.offset.map_or(0, |offset| offset.get()).to_le_bytes())?;
            file.write_all(&(entry.item_type as u16).to_le_bytes())?;
        }

        file.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Map;

    #[test]
    fn saved_empty_map_can_be_loaded() {
        let path = std::env::temp_dir().join(format!("runar-map-{}.map", uuid::Uuid::new_v4()));

        Map::new().save_to_path(&path).unwrap();
        let loaded = Map::load(&path).unwrap();

        assert!(loaded.container_offsets.is_empty());
        std::fs::remove_file(path).unwrap();
    }
}
