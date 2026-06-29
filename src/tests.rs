#[cfg(test)]
mod tests {
    use crate::explorer::bag::ExplorerResources;
    use crate::explorer::mapping::{Mapping, PlanetNodeId};

    use common_game::components::resource::{
        BasicResourceType,
        ComplexResourceType,
        ResourceType,
    };

    #[test]
    fn bag_starts_empty() {
        let bag = ExplorerResources::default();

        assert_eq!(bag.count_basic(BasicResourceType::Oxygen), 0);
        assert_eq!(bag.count_basic(BasicResourceType::Hydrogen), 0);
        assert_eq!(bag.count_basic(BasicResourceType::Carbon), 0);
        assert_eq!(bag.count_basic(BasicResourceType::Silicon), 0);

        assert_eq!(bag.count_complex(ComplexResourceType::Water), 0);
        assert_eq!(bag.count_complex(ComplexResourceType::Diamond), 0);
        assert_eq!(bag.count_complex(ComplexResourceType::Life), 0);
        assert_eq!(bag.count_complex(ComplexResourceType::Robot), 0);
        assert_eq!(bag.count_complex(ComplexResourceType::Dolphin), 0);
    }

    #[test]
    fn bag_reports_all_resources_missing_for_target_1_when_empty() {
        let bag = ExplorerResources::default();

        let missing = bag.missing_resource_types_for_target(1);

        assert!(missing.contains(&ResourceType::Basic(BasicResourceType::Oxygen)));
        assert!(missing.contains(&ResourceType::Basic(BasicResourceType::Hydrogen)));
        assert!(missing.contains(&ResourceType::Basic(BasicResourceType::Carbon)));
        assert!(missing.contains(&ResourceType::Basic(BasicResourceType::Silicon)));

        assert!(missing.contains(&ResourceType::Complex(ComplexResourceType::Water)));
        assert!(missing.contains(&ResourceType::Complex(ComplexResourceType::Diamond)));
        assert!(missing.contains(&ResourceType::Complex(ComplexResourceType::Life)));
        assert!(missing.contains(&ResourceType::Complex(ComplexResourceType::Robot)));
        assert!(missing.contains(&ResourceType::Complex(ComplexResourceType::Dolphin)));
    }

    #[test]
    fn empty_bag_does_not_have_all_resources() {
        let bag = ExplorerResources::default();

        assert!(!bag.has_all_resources_at_least(1));
        assert!(!bag.has_all_resources_at_least(2));
    }

    #[test]
    fn empty_bag_cannot_create_complex_resources() {
        let bag = ExplorerResources::default();

        assert!(!bag.can_create_complex(ComplexResourceType::Water));
        assert!(!bag.can_create_complex(ComplexResourceType::Diamond));
        assert!(!bag.can_create_complex(ComplexResourceType::Life));
        assert!(!bag.can_create_complex(ComplexResourceType::Robot));
        assert!(!bag.can_create_complex(ComplexResourceType::Dolphin));
        assert!(!bag.can_create_complex(ComplexResourceType::AIPartner));
    }

    #[test]
    fn mapping_starts_on_starting_planet() {
        let mapping = Mapping::new(1);

        assert_eq!(mapping.explorer_position, PlanetNodeId(1));
    }

    #[test]
    fn mapping_can_store_current_planet_basic_resources() {
        let mut mapping = Mapping::new(1);

        let resources = std::collections::HashSet::from([
            BasicResourceType::Oxygen,
            BasicResourceType::Hydrogen,
        ]);

        mapping.set_basic_resources_for_planet(resources);

        assert!(mapping.current_planet_can_produce(ResourceType::Basic(
            BasicResourceType::Oxygen
        )));

        assert!(mapping.current_planet_can_produce(ResourceType::Basic(
            BasicResourceType::Hydrogen
        )));

        assert!(!mapping.current_planet_can_produce(ResourceType::Basic(
            BasicResourceType::Carbon
        )));
    }

    #[test]
    fn mapping_can_store_current_planet_complex_resources() {
        let mut mapping = Mapping::new(1);

        let resources = std::collections::HashSet::from([
            ComplexResourceType::Water,
            ComplexResourceType::Diamond,
        ]);

        mapping.set_complex_resources_for_planet(resources);

        assert!(mapping.current_planet_can_produce(ResourceType::Complex(
            ComplexResourceType::Water
        )));

        assert!(mapping.current_planet_can_produce(ResourceType::Complex(
            ComplexResourceType::Diamond
        )));

        assert!(!mapping.current_planet_can_produce(ResourceType::Complex(
            ComplexResourceType::Life
        )));
    }

    #[test]
    fn mapping_connects_planets_and_finds_next_hop_to_resource() {
        let mut mapping = Mapping::new(1);

        mapping.add_planet(2);
        mapping.add_planet(3);

        mapping.connect(PlanetNodeId(1), PlanetNodeId(2), 1);
        mapping.connect(PlanetNodeId(2), PlanetNodeId(3), 1);

        mapping.set_explorer_position(PlanetNodeId(3));

        let resources = std::collections::HashSet::from([
            BasicResourceType::Carbon,
        ]);

        mapping.set_basic_resources_for_planet(resources);

        mapping.set_explorer_position(PlanetNodeId(1));

        let next = mapping.next_hop_to_resource(ResourceType::Basic(
            BasicResourceType::Carbon,
        ));

        assert_eq!(next, Some(PlanetNodeId(2)));
    }

    #[test]
    fn mapping_returns_none_if_resource_is_unknown() {
        let mut mapping = Mapping::new(1);

        mapping.add_planet(2);
        mapping.connect(PlanetNodeId(1), PlanetNodeId(2), 1);

        let next = mapping.next_hop_to_resource(ResourceType::Basic(
            BasicResourceType::Silicon,
        ));

        assert_eq!(next, None);
    }

    #[test]
    fn mapping_can_remove_planet() {
        let mut mapping = Mapping::new(1);

        mapping.add_planet(2);
        mapping.connect(PlanetNodeId(1), PlanetNodeId(2), 1);

        mapping.remove_planet(PlanetNodeId(2));

        let next = mapping.next_hop_to_resource(ResourceType::Basic(
            BasicResourceType::Oxygen,
        ));

        assert_eq!(next, None);
    }
}