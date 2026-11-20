# runar-map

Features carry application-defined, typed properties. Define a property struct
and implement `Property`, then attach it when creating a feature:

```rust
use runar_map::{Feature, Geometry, NodeIndex, Point, Property};

struct PlaceProperties {
    name: String,
    population: u64,
}
impl Property for PlaceProperties {}

let place = Feature::new(
    Geometry::Point(Point {
        node: NodeIndex(0),
    }),
    PlaceProperties {
        name: "Runar".to_owned(),
        population: 42,
    },
);

assert_eq!(place.properties.name, "Runar");
```