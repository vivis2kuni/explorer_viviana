use crate::explorer::bag::ExplorerResources;
use crate::logging::*;

use common_game::components::resource::{
    BasicResourceType,
    ComplexResourceRequest,
    ComplexResourceType,
    GenericResource,
    ResourceType,
};

use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator,
    OrchestratorToExplorer,
};

use common_game::protocols::planet_explorer::{
    ExplorerToPlanet,
    PlanetToExplorer,
};

use crossbeam_channel::{select, Receiver, Sender};

pub(crate) use crate::explorer::mapping::{Mapping, PlanetNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExplorerId(pub(crate) u32);

impl ExplorerId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Bag reale interna dell'explorer.
/// Non cambiarla in Vec<(ResourceType, usize)>, perché questa struttura
/// serve per conservare le risorse concrete e poterle usare nelle combinazioni.
pub type BagContent = ExplorerResources;

/// Tipo richiesto dal common protocol per comunicare il contenuto della bag.
pub type ExplorerBagSummary = Vec<(ResourceType, usize)>;

/// Messaggio dall'explorer all'orchestrator, parametrizzato con il tipo corretto
/// della bag secondo il protocollo comune.
pub type ExplorerToOrchestratorMsg = ExplorerToOrchestrator<ExplorerBagSummary>;

/// Main Explorer struct.
pub struct Explorer {
    pub id: ExplorerId,

    rx_orchestrator: Receiver<OrchestratorToExplorer>,
    tx_orchestrator: Sender<ExplorerToOrchestratorMsg>,

    /// Questo receiver deve essere creato una volta sola allo spawn.
    /// L'explorer ascolta sempre dallo stesso rx_planet.
    rx_planet: Receiver<PlanetToExplorer>,

    /// Sender verso il pianeta corrente.
    /// Questo può cambiare quando l'explorer viene spostato su un nuovo pianeta.
    tx_planet: Sender<ExplorerToPlanet>,

    bag: BagContent,
    running: bool,
    alive: bool,
    mapping: Mapping,
}

impl Explorer {
    pub fn new(
        id: u32,
        rx_orchestrator: Receiver<OrchestratorToExplorer>,
        tx_orchestrator: Sender<ExplorerToOrchestratorMsg>,
        rx_planet: Receiver<PlanetToExplorer>,
        tx_planet: Sender<ExplorerToPlanet>,
        starting_planet: u32,
    ) -> Result<Self, String> {
        Ok(Self {
            id: ExplorerId::new(id),
            rx_orchestrator,
            tx_orchestrator,
            rx_planet,
            tx_planet,
            bag: BagContent::default(),
            running: false,
            alive: true,
            mapping: Mapping::new(starting_planet),
        })
    }

    pub fn run(&mut self) {
        while self.alive {
            select! {
                recv(self.rx_orchestrator) -> msg => {
                    match msg {
                        Ok(message) => {
                            log_orchestrator_message(
                                0,
                                self.id,
                                &format!("{message:?}"),
                            );
                            self.handle_orchestrator_message(message);
                        }
                        Err(_) => {
                            log_explorer_event(
                                self.id,
                                "Orchestrator channel closed",
                                common_game::logging::Channel::Warning,
                            );
                            self.alive = false;
                        }
                    }
                }

                recv(self.rx_planet) -> msg => {
                    match msg {
                        Ok(message) => {
                            log_planet_message(
                                self.mapping.explorer_position.0,
                                self.id,
                                &format!("{message:?}"),
                            );
                            self.handle_planet_message(message);
                        }
                        Err(_) => {
                            log_explorer_event(
                                self.id,
                                "Planet channel closed",
                                common_game::logging::Channel::Warning,
                            );
                        }
                    }
                }
            }
        }
    }

    fn handle_orchestrator_message(&mut self, msg: OrchestratorToExplorer) {
        use OrchestratorToExplorer::*;

        match msg {
            StartExplorerAI => {
                log_explorer_event(
                    self.id,
                    "StartExplorerAI received",
                    common_game::logging::Channel::Info,
                );

                self.running = true;

                let _ = self.tx_orchestrator.send(
                    ExplorerToOrchestrator::StartExplorerAIResult {
                        explorer_id: self.id.0,
                    },
                );

                self.request_current_planet_info();
            }

            StopExplorerAI => {
                log_explorer_event(
                    self.id,
                    "StopExplorerAI received",
                    common_game::logging::Channel::Info,
                );

                self.running = false;

                let _ = self.tx_orchestrator.send(
                    ExplorerToOrchestrator::StopExplorerAIResult {
                        explorer_id: self.id.0,
                    },
                );
            }

            GenerateResourceRequest { to_generate } => {
                log_explorer_event(
                    self.id,
                    &format!("GenerateResourceRequest {to_generate:?}"),
                    common_game::logging::Channel::Debug,
                );

                let _ = self.tx_planet.send(
                    ExplorerToPlanet::GenerateResourceRequest {
                        explorer_id: self.id.0,
                        resource: to_generate,
                    },
                );
            }

            CombineResourceRequest { to_generate } => {
                log_explorer_event(
                    self.id,
                    &format!("CombineResourceRequest {to_generate:?}"),
                    common_game::logging::Channel::Debug,
                );

                if !self.send_combine_request(to_generate) {
                    let _ = self.tx_orchestrator.send(
                        ExplorerToOrchestrator::CombineResourceResponse {
                            explorer_id: self.id.0,
                            generated: Err(format!(
                                "Cannot create complex resource: {to_generate:?}"
                            )),
                        },
                    );
                }
            }

            CurrentPlanetRequest => {
                let _ = self.tx_orchestrator.send(
                    ExplorerToOrchestrator::CurrentPlanetResult {
                        explorer_id: self.id.0,
                        planet_id: self.mapping.explorer_position.0,
                    },
                );
            }

            SupportedResourceRequest => {
                let _ = self.tx_planet.send(
                    ExplorerToPlanet::SupportedResourceRequest {
                        explorer_id: self.id.0,
                    },
                );
            }

            SupportedCombinationRequest => {
                let _ = self.tx_planet.send(
                    ExplorerToPlanet::SupportedCombinationRequest {
                        explorer_id: self.id.0,
                    },
                );
            }

            BagContentRequest => {
                let _ = self.tx_orchestrator.send(
                    ExplorerToOrchestrator::BagContentResponse {
                        explorer_id: self.id.0,
                        bag_content: self.bag.present_resource_types(),
                    },
                );
            }

            NeighborsResponse { neighbors } => {
                for neighbor in neighbors {
                    self.mapping.connect(
                        self.mapping.explorer_position,
                        PlanetNodeId(neighbor),
                        1,
                    );
                }

                self.request_or_perform_next_action();
            }

            MoveToPlanet {
                sender_to_new_planet,
                planet_id,
            } => {
                let planet_id_u32: u32 = planet_id.into();
                let old_planet_id = self.mapping.explorer_position.0;

                match sender_to_new_planet {
                    Some(sender) => {
                        self.tx_planet = sender;
                        self.mapping
                            .set_explorer_position(PlanetNodeId(planet_id_u32));

                        log_travel(
                            self.id,
                            old_planet_id,
                            planet_id_u32,
                            true,
                        );

                        let _ = self.tx_orchestrator.send(
                            ExplorerToOrchestrator::MovedToPlanetResult {
                                explorer_id: self.id.0,
                                planet_id,
                            },
                        );

                        self.request_current_planet_info();
                    }

                    None => {
                        self.mapping.remove_planet(PlanetNodeId(planet_id_u32));

                        log_travel(
                            self.id,
                            old_planet_id,
                            planet_id_u32,
                            false,
                        );

                        let _ = self.tx_orchestrator.send(
                            ExplorerToOrchestrator::MovedToPlanetResult {
                                explorer_id: self.id.0,
                                planet_id,
                            },
                        );

                        self.request_or_perform_next_action();
                    }
                }
            }

            ResetExplorerAI => {
                self.mapping = Mapping::new(self.mapping.explorer_position.0);
                self.bag = BagContent::default();
                self.running = false;

                let _ = self.tx_orchestrator.send(
                    ExplorerToOrchestrator::ResetExplorerAIResult {
                        explorer_id: self.id.0,
                    },
                );
            }

            KillExplorer => {
                let _ = self.tx_orchestrator.send(
                    ExplorerToOrchestrator::KillExplorerResult {
                        explorer_id: self.id.0,
                    },
                );

                self.alive = false;
            }
        }
    }

    fn handle_planet_message(&mut self, msg: PlanetToExplorer) {
        match msg {
            PlanetToExplorer::SupportedResourceResponse { resource_list } => {
                let supported_resources = resource_list.clone();

                self.mapping
                    .set_basic_resources_for_planet(resource_list);

                let _ = self.tx_orchestrator.send(
                    ExplorerToOrchestrator::SupportedResourceResult {
                        explorer_id: self.id.0,
                        supported_resources,
                    },
                );
            }

            PlanetToExplorer::SupportedCombinationResponse { combination_list } => {
                let supported_combinations = combination_list.clone();

                self.mapping
                    .set_complex_resources_for_planet(combination_list);

                let _ = self.tx_orchestrator.send(
                    ExplorerToOrchestrator::SupportedCombinationResult {
                        explorer_id: self.id.0,
                        combination_list: supported_combinations,
                    },
                );
            }

            PlanetToExplorer::GenerateResourceResponse { resource } => {
                match resource {
                    Some(resource) => {
                        self.bag.add_basic(resource);

                        let _ = self.tx_orchestrator.send(
                            ExplorerToOrchestrator::GenerateResourceResponse {
                                explorer_id: self.id.0,
                                generated: Ok(()),
                            },
                        );

                        if self.running {
                            self.request_or_perform_next_action();
                        }
                    }

                    None => {
                        let _ = self.tx_orchestrator.send(
                            ExplorerToOrchestrator::GenerateResourceResponse {
                                explorer_id: self.id.0,
                                generated: Err(
                                    "Planet could not generate requested resource".to_string(),
                                ),
                            },
                        );

                        if self.running {
                            self.request_neighbors();
                        }
                    }
                }
            }

            PlanetToExplorer::CombineResourceResponse { complex_response } => {
                match complex_response {
                    Ok(complex_resource) => {
                        self.bag.add_complex(complex_resource);

                        let _ = self.tx_orchestrator.send(
                            ExplorerToOrchestrator::CombineResourceResponse {
                                explorer_id: self.id.0,
                                generated: Ok(()),
                            },
                        );

                        if self.running {
                            self.request_or_perform_next_action();
                        }
                    }

                    Err((_error, r1, r2)) => {
                        self.add_generic_resource(r1);
                        self.add_generic_resource(r2);

                        let _ = self.tx_orchestrator.send(
                            ExplorerToOrchestrator::CombineResourceResponse {
                                explorer_id: self.id.0,
                                generated: Err(
                                    "Planet could not combine requested resource".to_string(),
                                ),
                            },
                        );

                        if self.running {
                            self.request_neighbors();
                        }
                    }
                }
            }

            PlanetToExplorer::AvailableEnergyCellResponse { available_cells } => {
                if !self.running {
                    return;
                }

                if available_cells == 0 {
                    self.mapping.set_energy_cell(
                        self.mapping.explorer_position,
                        false,
                    );
                    self.request_neighbors();
                } else {
                    self.request_or_perform_next_action();
                }
            }

            PlanetToExplorer::Stopped => {
                self.request_neighbors();
            }
        }
    }

    fn request_current_planet_info(&self) {
        let _ = self.tx_planet.send(
            ExplorerToPlanet::SupportedResourceRequest {
                explorer_id: self.id.0,
            },
        );

        let _ = self.tx_planet.send(
            ExplorerToPlanet::SupportedCombinationRequest {
                explorer_id: self.id.0,
            },
        );

        let _ = self.tx_planet.send(
            ExplorerToPlanet::AvailableEnergyCellRequest {
                explorer_id: self.id.0,
            },
        );
    }

    fn request_neighbors(&self) {
        let _ = self.tx_orchestrator.send(
            ExplorerToOrchestrator::NeighborsRequest {
                explorer_id: self.id.0,
                current_planet_id: self.mapping.explorer_position.0,
            },
        );
    }

    fn request_or_perform_next_action(&mut self) {
        if !self.running {
            return;
        }

        for resource in self.bag.missing_resource_types() {
            if self.mapping.current_planet_can_produce(resource) {
                match resource {
                    ResourceType::Basic(basic_type) => {
                        let _ = self.tx_planet.send(
                            ExplorerToPlanet::GenerateResourceRequest {
                                explorer_id: self.id.0,
                                resource: basic_type,
                            },
                        );
                        return;
                    }

                    ResourceType::Complex(complex_type) => {
                        if self.send_combine_request(complex_type) {
                            return;
                        }
                    }
                }
            }
        }

        for resource in self.bag.missing_resource_types() {
            if let Some(destination) = self.mapping.next_hop_to_resource(resource) {
                if destination == self.mapping.explorer_position {
                    continue;
                }

                let _ = self.tx_orchestrator.send(
                    ExplorerToOrchestrator::TravelToPlanetRequest {
                        explorer_id: self.id.0,
                        current_planet_id: self.mapping.explorer_position.0,
                        dst_planet_id: destination.0,
                    },
                );

                return;
            }
        }

        self.request_neighbors();
    }

    fn send_combine_request(&mut self, target: ComplexResourceType) -> bool {
        if !self.bag.can_create_complex(target) {
            return false;
        }

        match target {
            ComplexResourceType::Water => {
                let Some(hydrogen) = self.bag.take_basic(BasicResourceType::Hydrogen) else {
                    return false;
                };

                let Some(oxygen) = self.bag.take_basic(BasicResourceType::Oxygen) else {
                    self.bag.add_basic(hydrogen);
                    return false;
                };

                let _ = self.tx_planet.send(
                    ExplorerToPlanet::CombineResourceRequest {
                        explorer_id: self.id.0,
                        msg: ComplexResourceRequest::Water(
                            hydrogen.to_hydrogen().unwrap(),
                            oxygen.to_oxygen().unwrap(),
                        ),
                    },
                );

                true
            }

            ComplexResourceType::Diamond => {
                let Some(carbon1) = self.bag.take_basic(BasicResourceType::Carbon) else {
                    return false;
                };

                let Some(carbon2) = self.bag.take_basic(BasicResourceType::Carbon) else {
                    self.bag.add_basic(carbon1);
                    return false;
                };

                let _ = self.tx_planet.send(
                    ExplorerToPlanet::CombineResourceRequest {
                        explorer_id: self.id.0,
                        msg: ComplexResourceRequest::Diamond(
                            carbon1.to_carbon().unwrap(),
                            carbon2.to_carbon().unwrap(),
                        ),
                    },
                );

                true
            }

            ComplexResourceType::Life => {
                let Some(water) = self.bag.take_complex(ComplexResourceType::Water) else {
                    return false;
                };

                let Some(carbon) = self.bag.take_basic(BasicResourceType::Carbon) else {
                    self.bag.add_complex(water);
                    return false;
                };

                let _ = self.tx_planet.send(
                    ExplorerToPlanet::CombineResourceRequest {
                        explorer_id: self.id.0,
                        msg: ComplexResourceRequest::Life(
                            water.to_water().unwrap(),
                            carbon.to_carbon().unwrap(),
                        ),
                    },
                );

                true
            }

            ComplexResourceType::Robot => {
                let Some(silicon) = self.bag.take_basic(BasicResourceType::Silicon) else {
                    return false;
                };

                let Some(life) = self.bag.take_complex(ComplexResourceType::Life) else {
                    self.bag.add_basic(silicon);
                    return false;
                };

                let _ = self.tx_planet.send(
                    ExplorerToPlanet::CombineResourceRequest {
                        explorer_id: self.id.0,
                        msg: ComplexResourceRequest::Robot(
                            silicon.to_silicon().unwrap(),
                            life.to_life().unwrap(),
                        ),
                    },
                );

                true
            }

            ComplexResourceType::Dolphin => {
                let Some(life) = self.bag.take_complex(ComplexResourceType::Life) else {
                    return false;
                };

                let Some(water) = self.bag.take_complex(ComplexResourceType::Water) else {
                    self.bag.add_complex(life);
                    return false;
                };

                let _ = self.tx_planet.send(
                    ExplorerToPlanet::CombineResourceRequest {
                        explorer_id: self.id.0,
                        msg: ComplexResourceRequest::Dolphin(
                            water.to_water().unwrap(),
                            life.to_life().unwrap(),
                        ),
                    },
                );

                true
            }

            ComplexResourceType::AIPartner => false,
        }
    }

    fn add_generic_resource(&mut self, res: GenericResource) {
        match res {
            GenericResource::BasicResources(basic) => {
                self.bag.add_basic(basic);
            }

            GenericResource::ComplexResources(complex) => {
                self.bag.add_complex(complex);
            }
        }
    }
}