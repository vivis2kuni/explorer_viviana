use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::Reverse;
use std::time::{Instant, Duration};

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
    pub has_energy_cell: bool,
    pub last_visited: Option<Instant>,
}

impl PlanetCapabilities {
    pub fn new() -> Self {
        Self {
            generates: HashSet::new(),
            combines: HashSet::new(),
            has_energy_cell: true,
            last_visited: None,
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
    const ENERGY_RESPAWN: Duration = Duration::from_secs(20);

    fn ensure_planet_exists(&mut self, id: PlanetNodeId) {
        self.planets.entry(id).or_insert_with(PlanetCapabilities::new);
        self.connections.entry(id).or_insert_with(Vec::new);
    }

    pub fn new(planet_id: u32) -> Self {
        let mut out = Self {
            planets: HashMap::new(),
            connections: HashMap::new(),
            explorer_position: PlanetNodeId(planet_id),
        };
        out.add_planet(planet_id);
        out
    }

    pub fn add_planet(&mut self, id: u32) -> PlanetNodeId {
        let id = PlanetNodeId(id);
        self.planets.insert(id, PlanetCapabilities::new());
        self.connections.insert(id, Vec::new());
        id
    }

    pub fn set_basic_resources_for_planet(&mut self, resources: HashSet<BasicResourceType>) {
        if let Some(caps) = self.planets.get_mut(&self.explorer_position) {
            caps.generates = resources;
        }
    }

    pub fn set_complex_resources_for_planet(&mut self, resources: HashSet<ComplexResourceType>) {
        if let Some(caps) = self.planets.get_mut(&self.explorer_position) {
            caps.combines = resources;
        }
    }

    pub fn connect(&mut self, from: PlanetNodeId, to: PlanetNodeId, cost: u32) {
        self.ensure_planet_exists(to);
        if let Some(v) = self.connections.get_mut(&from) {
            if !v.iter().any(|c| c.to == to) {
                v.push(PlanetConnection { to, cost });
            }
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
            self.visit_planet(to);
            Ok(())
        } else {
            Err("planets not connected")
        }
    }

    pub fn current_planet_can_produce(&self, target: ResourceType) -> bool {
        let caps = match self.planets.get(&self.explorer_position) {
            Some(caps) => caps,
            None => return false,
        };
        match target {
            ResourceType::Basic(b) => caps.generates.contains(&b),
            ResourceType::Complex(c) => caps.combines.contains(&c),
        }
    }

    fn planet_can_produce(caps: &PlanetCapabilities, target: ResourceType) -> bool {
        match target {
            ResourceType::Basic(b) => caps.generates.contains(&b),
            ResourceType::Complex(c) => caps.combines.contains(&c),
        }
    }

    pub fn set_energy_cell(&mut self, planet: PlanetNodeId, has_energy: bool) {
        if let Some(caps) = self.planets.get_mut(&planet) {
            caps.has_energy_cell = has_energy;
            if has_energy {
                caps.last_visited = None;
            }
        }
    }

    /// Aggiorna le energy cells scadute
    pub fn update_energy_cells(&mut self) {
        let now = Instant::now();
        for caps in self.planets.values_mut() {
            if !caps.has_energy_cell {
                if let Some(last) = caps.last_visited {
                    if now.duration_since(last) >= Self::ENERGY_RESPAWN {
                        caps.has_energy_cell = true;
                        caps.last_visited = None;
                    }
                }
            }
        }
    }

    /// Visita un pianeta (consume energy cell)
    pub fn visit_planet(&mut self, id: PlanetNodeId) {
        if let Some(caps) = self.planets.get_mut(&id) {
            if caps.has_energy_cell {
                caps.has_energy_cell = false;
                caps.last_visited = Some(Instant::now());
            }
        }
        self.explorer_position = id;
    }

    /// Shortest path considerando energy cell se non ci sono risorse
    pub fn shortest_path_to_resource(&mut self, start: PlanetNodeId, target: ResourceType) -> Option<(u32, PlanetNodeId)> {
        self.update_energy_cells();

        let mut dist: HashMap<PlanetNodeId, u32> = HashMap::new();
        let mut heap = BinaryHeap::new();

        dist.insert(start, 0);
        heap.push((Reverse(0), start));

        while let Some((Reverse(cost), node)) = heap.pop() {
            let caps = self.planets.get(&node)?;

            if Self::planet_can_produce(caps, target) {
                return Some((cost, node));
            }

            // fallback: energy cell pianeti vuoti
            if target == ResourceType::Basic(BasicResourceType::Hydrogen) && caps.has_energy_cell
                && caps.generates.is_empty() && caps.combines.is_empty()
            {
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

    pub fn next_hop_to_resource(&mut self, target: ResourceType) -> Option<PlanetNodeId> {
        self.update_energy_cells();

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

                if prev.is_none() { return Some(current); }

                while let Some(&p) = prev {
                    if p == start { return Some(current); }
                    current = p;
                    prev = came_from.get(&current);
                }
                return Some(current);
            }

            // fallback: energy cell pianeti vuoti
            if target == ResourceType::Basic(BasicResourceType::Hydrogen)
                && caps.has_energy_cell
                && caps.generates.is_empty()
                && caps.combines.is_empty()
            {
                return Some(node);
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
