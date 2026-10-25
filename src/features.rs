use crate::{LocalizedString, Property};

pub struct Road {}
impl Property for Road {}

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
