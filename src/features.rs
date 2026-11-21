use crate::{
    LocalizedString, Property,
    features::restrictions::{RestrictionID, ZoneID},
};
use runar_units::types::l_1::Length;

pub mod restrictions;

pub struct Road {
    /// Surface material of the road.
    pub surface: RoadSurface,
    /// Condition of the road surface.
    pub surface_condition: RoadSurfaceCondition,
    /// Name of the road.
    ///
    /// # Example
    /// - "Main Street"
    /// - "Barrington Road"
    /// - "Dexter Avenue"
    pub name: LocalizedString,

    /// References to the road.
    ///
    /// # Example
    /// - "A1"
    /// - "I95"
    /// - "M25"
    /// - "E20"
    pub refs: Vec<String>,
    /// Lanes of the road.
    pub lanes: Vec<Lane>,
    pub zones: Vec<ZoneID>,
}
impl Property for Road {}

pub struct Lane {
    pub kind: LaneKind,
    pub direction: LaneDirection,
    /// The width of the lane in meters.
    pub width: Option<Length<f64, runar_units::types::l_1::Meter>>,
    /// Local lane specific restriction.
    pub restriction: Option<RestrictionID>,
}

pub enum LaneDirection {
    /// The lane is for traffic moving in the same direction as the direction of the underlying road.
    Forward,
    /// The lane is for traffic moving in the opposite direction as the direction of the underlying road.
    Backward,
    /// The lane is for traffic moving in both directions.
    Both,
}

/// The primary purpose of the lane.
///
/// This is primarily used to determine how the lane is rendered on a map, and if no other
/// information is available, what types of vehicles are allowed to use the lane.
pub enum LaneKind {
    MotorVehicle,
    Bicycle,
    Foot,
    Bus,
}

/// The material of the road surface.
pub enum RoadSurface {
    Asphalt,
    Concrete,
    Gravel,
    Dirt,
    Cobblestone,
}

/// The condition of the road surface.
pub enum RoadSurfaceCondition {
    Excellent,
    Good,
    Fair,
    Poor,
    Bad,
}

pub struct LandUse {
    pub landuse_kind: LandUseKind,
    pub name: LocalizedString,
}
impl Property for LandUse {}
pub enum LandUseKind {
    Residential,
    Commercial,
    Industrial,
    Farmland,
    Forest,
    Cemetery,
    Military,
}
