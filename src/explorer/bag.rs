use common_game::components::resource::{
    BasicResourceType, ComplexResourceType, ResourceType, BasicResource, ComplexResource,
};
use std::collections::HashMap;


#[derive(Debug)]
pub struct ExplorerResources {
    pub basic: HashMap<BasicResourceType, Vec<BasicResource>>,
    pub complex: HashMap<ComplexResourceType, Vec<ComplexResource>>,
}

impl Default for ExplorerResources {
    fn default() -> Self {
        Self {
            basic: HashMap::new(),
            complex: HashMap::new(),
        }
    }
}

impl ExplorerResources {
    pub fn add_basic(&mut self, resource: BasicResource) {
        let kind = resource.get_type();
        self.basic.entry(kind).or_default().push(resource);
    }

    pub fn add_complex(&mut self, resource: ComplexResource) {
        let kind = resource.get_type();
        self.complex.entry(kind).or_default().push(resource);
    }

    pub fn take_basic(&mut self, kind: BasicResourceType) -> Option<BasicResource> {
        self.basic.get_mut(&kind).and_then(|v| v.pop())
    }

    pub fn take_complex(&mut self, kind: ComplexResourceType) -> Option<ComplexResource> {
        self.complex.get_mut(&kind).and_then(|v| v.pop())
    }

    pub fn count_basic(&self, kind: BasicResourceType) -> usize {
        self.basic.get(&kind).map(|v| v.len()).unwrap_or(0)
    }

    pub fn count_complex(&self, kind: ComplexResourceType) -> usize {
        self.complex.get(&kind).map(|v| v.len()).unwrap_or(0)
    }

    pub fn present_resource_types(&self) -> Vec<(ResourceType, usize)> {
        let mut types = Vec::new();

        for (kind, resources) in &self.basic {
            let count = resources.len();
            if count > 0 {
                types.push((ResourceType::Basic(*kind), count));
            }
        }

        for (kind, resources) in &self.complex {
            let count = resources.len();
            if count > 0 {
                types.push((ResourceType::Complex(*kind), count));
            }
        }

        types
    }

    pub fn can_create_complex(&self, target: ComplexResourceType) -> bool {
        match target {
            ComplexResourceType::Water => {
                self.count_basic(BasicResourceType::Oxygen) >= 1 &&
                    self.count_basic(BasicResourceType::Hydrogen) >= 1
            }

            ComplexResourceType::Diamond => {
                self.count_basic(BasicResourceType::Carbon) >= 2
            }

            ComplexResourceType::Life => {
                self.count_complex(ComplexResourceType::Water) >= 1 &&
                    self.count_basic(BasicResourceType::Carbon) >= 1
            }

            ComplexResourceType::Robot => {
                self.count_basic(BasicResourceType::Silicon) >= 1 &&
                    self.count_complex(ComplexResourceType::Life) >= 1
            }

            ComplexResourceType::Dolphin => {
                self.count_complex(ComplexResourceType::Life) >= 1 &&
                    self.count_complex(ComplexResourceType::Water) >= 1
            }
            ComplexResourceType::AIPartner => {
                false
            }
        }
    }

    pub fn count_resource(&self, resource: ResourceType) -> usize {
        match resource {
            ResourceType::Basic(b) => self.count_basic(b),
            ResourceType::Complex(c) => self.count_complex(c),
        }
    }

    pub fn all_target_resources() -> Vec<ResourceType> {
        vec![
            ResourceType::Basic(BasicResourceType::Oxygen),
            ResourceType::Basic(BasicResourceType::Hydrogen),
            ResourceType::Basic(BasicResourceType::Carbon),
            ResourceType::Basic(BasicResourceType::Silicon),

            ResourceType::Complex(ComplexResourceType::Diamond),
            ResourceType::Complex(ComplexResourceType::Water),
            ResourceType::Complex(ComplexResourceType::Life),
            ResourceType::Complex(ComplexResourceType::Robot),
            ResourceType::Complex(ComplexResourceType::Dolphin),
        ]
    }

    pub fn missing_resource_types_for_target(&self, target_count: usize) -> Vec<ResourceType> {
        Self::all_target_resources()
            .into_iter()
            .filter(|resource| self.count_resource(*resource) < target_count)
            .collect()
    }

    pub fn has_all_resources_at_least(&self, target_count: usize) -> bool {
        Self::all_target_resources()
            .into_iter()
            .all(|resource| self.count_resource(resource) >= target_count)
    }
}
