use std::num::NonZeroU32;

use runar_units::types::{l_1::Length, m_1::Mass};

pub struct Restriction {
    /// Vehicle types that the restriction applies to.
    pub applies_to: Vehicles,
    /// Role of vehicles, that the restriction applies to.
    pub applies_to_role: RestrictionRole,
    /// Conditions, that are stipulated by the restriction. If no conditions are specified, the
    /// restriction applies at all times.
    pub conditions: Vec<RestrictionCondition>,
}

pub struct RestrictionID {
    pub id: u32,
}

pub struct ZoneID {
    pub id: NonZeroU32,
}

pub enum RestrictionScope {
    Zone(ZoneID),
    Point,
}

bitflags::bitflags! {
    /// Generic vehicle types.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Vehicles: u16 {
        const FOOT = 1 << 0;
        const BICYCLE = 1 << 1;
        const MOPED = 1 << 2;
        const MOTORCYCLE = 1 << 3;
        const CAR = 1 << 4;
        const BUS = 1 << 5;
        const TRUCK = 1 << 6;
        const TRACTOR = 1 << 7;
    }
}

pub enum RestrictionCondition {
    Time(),
    MaxWeight(Mass<f64, runar_units::types::m_1::Kilogram>),
    MaxHeight(Length<f64, runar_units::types::l_1::Meter>),
    MaxWidth(Length<f64, runar_units::types::l_1::Meter>),
    MaxLength(Length<f64, runar_units::types::l_1::Meter>),
    MinOccupancy(u8),
}

bitflags::bitflags! {
    /// The role of vehicles, that the restriction applies to.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RestrictionRole: u16 {
        const DESTINATION = 1 << 0;
        const DELIVERY = 1 << 1;
        const RESIDENTS = 1 << 2;
        const EMERGENCY = 1 << 3;
        const PUBLIC_TRANSPORT = 1 << 4;
        const HAZARDOUS = 1 << 5;
        const AGRICULTURAL = 1 << 6;
    }
}
