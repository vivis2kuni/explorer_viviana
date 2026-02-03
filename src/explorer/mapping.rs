use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::Reverse;

use common_game::components::resource::{
    BasicResourceType,
    ComplexResourceType,
    ResourceType,
};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Ord, PartialOrd)]
pub struct PlanetNodeId(pub u32);

#[derive(Debug, Clone)]
pub struct PlanetCapabilities {
    pub generates: HashSet<BasicResourceType>,
    pub combines: HashSet<ComplexResourceType>,
}
impl PlanetCapabilities {
    pub fn new() -> Self {
        Self {
            generates: HashSet::new(),
            combines: HashSet::new(),
        }
    }
}

/// Connessione pesata tra pianeti
#[derive(Debug, Clone)]
pub struct PlanetConnection {
    pub to: PlanetNodeId,
    pub cost: u32,
}

pub struct Mapping {
    planets: HashMap<PlanetNodeId, PlanetCapabilities>,
    connections: HashMap<PlanetNodeId, Vec<PlanetConnection>>,
    pub(crate) explorer_position: PlanetNodeId,
}

impl Mapping {

    pub fn new(planet_id : u32) -> Self {
        let mut out = Self {
            planets: HashMap::new(),
            connections: HashMap::new(),
            explorer_position: PlanetNodeId(planet_id),
        };
        out.add_planet(planet_id);
        return out;
    }

    pub fn add_planet(&mut self, id: u32) -> PlanetNodeId {
        let id = PlanetNodeId(id);

        self.planets.insert(id, PlanetCapabilities::new());
        self.connections.insert(id, Vec::new());

        id
    }

    pub fn set_basic_resources_for_planet(
        &mut self,
        resources: HashSet<BasicResourceType>,
    ) {
        let caps = self.planets
            .get_mut(&self.explorer_position)
            .ok_or("planet not found");

        caps.unwrap().generates = resources;
    }

    pub fn set_complex_resources_for_planet(
        &mut self,
        resources: HashSet<ComplexResourceType>,
    ) {
        let caps = self.planets
            .get_mut(&self.explorer_position)
            .ok_or("planet not found");

        caps.unwrap().combines = resources;
    }


    pub fn connect(
        &mut self,
        from: PlanetNodeId,
        to: PlanetNodeId,
        cost: u32,
    ) {
        if let Some(v) = self.connections.get_mut(&from) {
            v.push(PlanetConnection { to, cost });
        }
    }

    pub fn remove_planet(&mut self, id: PlanetNodeId) {
        self.planets.remove(&id);
        self.connections.remove(&id);

        for v in self.connections.values_mut() {
            v.retain(|c| c.to != id);
        }
    }


    pub fn set_explorer_position(&mut self, id: PlanetNodeId) {
        self.explorer_position = id;
    }

    pub fn move_explorer(&mut self, to: PlanetNodeId) -> Result<(), &'static str> {
        let connected = self.connections
            .get(&self.explorer_position)
            .map(|v| v.iter().any(|c| c.to == to))
            .unwrap_or(false);

        if connected {
            self.explorer_position = to;
            Ok(())
        } else {
            Err("planets not connected")
        }
    }

    pub fn current_planet_can_produce(
        &self,
        target: ResourceType,
    ) -> bool {
        let caps = match self.planets.get(&self.explorer_position) {
            Some(caps) => caps,
            None => return false,
        };

        match target {
            ResourceType::Basic(b) => caps.generates.contains(&b),
            ResourceType::Complex(c) => caps.combines.contains(&c),
        }
    }


    fn planet_can_produce(
        caps: &PlanetCapabilities,
        target: ResourceType,
    ) -> bool {
        match target {
            ResourceType::Basic(b) => caps.generates.contains(&b),
            ResourceType::Complex(c) => caps.combines.contains(&c),
        }
    }

    /// Ritorna il pianeta più vicino che può produrre la risorsa target
    pub fn shortest_path_to_resource(
        &self,
        start: PlanetNodeId,
        target: ResourceType,
    ) -> Option<(u32, PlanetNodeId)> {
        let mut dist: HashMap<PlanetNodeId, u32> = HashMap::new();
        let mut heap = BinaryHeap::new();

        dist.insert(start, 0);
        heap.push((Reverse(0), start));

        while let Some((Reverse(cost), node)) = heap.pop() {
            let caps = self.planets.get(&node)?;

            if Self::planet_can_produce(caps, target) {
                return Some((cost, node));
            }

            if cost > *dist.get(&node).unwrap_or(&u32::MAX) {
                continue;
            }

            if let Some(conns) = self.connections.get(&node) {
                for conn in conns {
                    let next = cost + conn.cost;
                    if next < *dist.get(&conn.to).unwrap_or(&u32::MAX) {
                        dist.insert(conn.to, next);
                        heap.push((Reverse(next), conn.to));
                    }
                }
            }
        }

        None
    }

    pub fn next_hop_to_resource(&self, target: ResourceType) -> Option<PlanetNodeId> {
        let start = self.explorer_position;
        let mut dist: HashMap<PlanetNodeId, u32> = HashMap::new();
        let mut heap = BinaryHeap::new();
        let mut came_from: HashMap<PlanetNodeId, PlanetNodeId> = HashMap::new();

        dist.insert(start, 0);
        heap.push((Reverse(0), start));

        while let Some((Reverse(cost), node)) = heap.pop() {
            let caps = self.planets.get(&node)?;
            if Self::planet_can_produce(caps, target) {
                // Risali per trovare il next hop
                let mut current = node;
                let mut prev = came_from.get(&current);

                // Se il nodo target è adiacente allo start, next hop è il target stesso
                if prev.is_none() {
                    return Some(current);
                }

                // Risali fino al primo passo dopo start
                while let Some(&p) = prev {
                    if p == start {
                        return Some(current);
                    }
                    current = p;
                    prev = came_from.get(&current);
                }
                return Some(current);
            }

            if cost > *dist.get(&node).unwrap_or(&u32::MAX) {
                continue;
            }

            if let Some(conns) = self.connections.get(&node) {
                for conn in conns {
                    let next_cost = cost + conn.cost;
                    if next_cost < *dist.get(&conn.to).unwrap_or(&u32::MAX) {
                        dist.insert(conn.to, next_cost);
                        came_from.insert(conn.to, node);
                        heap.push((Reverse(next_cost), conn.to));
                    }
                }
            }
        }

        None
    }
}
