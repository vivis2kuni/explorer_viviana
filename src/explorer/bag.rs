use common_game::components::resource::{
    BasicResourceType, ComplexResourceType, ResourceType, BasicResource, ComplexResource,
};
use std::collections::HashMap;

/// Alias per la bag dell'explorer
pub type BagContent = ExplorerResources;

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


    pub fn missing_resource_types(&self) -> Vec<ResourceType> {
        use common_game::components::resource::*;

        let mut missing = Vec::new();

        let all_basic = vec![
            BasicResourceType::Oxygen,
            BasicResourceType::Hydrogen,
            BasicResourceType::Carbon,
            BasicResourceType::Silicon,
        ];
        for &b in &all_basic {
            if !self.basic.contains_key(&b) || self.basic[&b].is_empty() {
                missing.push(ResourceType::Basic(b));
            }
        }

        let all_complex = vec![
            ComplexResourceType::Diamond,
            ComplexResourceType::Water,
            ComplexResourceType::Life,
            ComplexResourceType::Robot,
            ComplexResourceType::Dolphin,
            //ComplexResourceType::AIPartner,
        ];
        for &c in &all_complex {
            if !self.complex.contains_key(&c) || self.complex[&c].is_empty() {
                missing.push(ResourceType::Complex(c));
            }
        }

        missing
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
}
