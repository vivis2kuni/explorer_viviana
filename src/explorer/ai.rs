use crate::logging::*;
use crate::explorer::bag::ExplorerResources;
use common_game::protocols::planet_explorer::{
    ExplorerToPlanet, PlanetToExplorer,
};
use common_game::protocols::orchestrator_explorer::{
    OrchestratorToExplorer, ExplorerToOrchestrator,
};
use crossbeam_channel::{Receiver, Sender, select, SendError};
use std::collections::HashSet;
use common_game::components::resource::{BasicResourceType, ComplexResourceRequest, ComplexResourceType, GenericResource, ResourceType};
pub(crate) use crate::explorer::mapping::{Mapping, PlanetNodeId};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExplorerId(pub(crate) u32);

impl ExplorerId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Placeholder type for bag content
pub type BagContent = ExplorerResources;

/// Main Explorer struct
pub struct Explorer {
    pub id: ExplorerId,
    rx_orchestrator: Receiver<OrchestratorToExplorer>,
    tx_orchestrator: Sender<ExplorerToOrchestrator<Vec<(common_game::components::resource::ResourceType,usize)>>>,
    rx_planet: Receiver<PlanetToExplorer>,
    tx_planet: Sender<ExplorerToPlanet>,
    bag: BagContent,
    running: bool,
    mapping: Mapping
}

impl Explorer {
    pub fn new(
        id: u32,
        rx_orchestrator: Receiver<OrchestratorToExplorer>,
        tx_orchestrator: Sender<ExplorerToOrchestrator<Vec<(common_game::components::resource::ResourceType,usize)>>>,
        rx_planet: Receiver<PlanetToExplorer>,
        tx_planet: Sender<ExplorerToPlanet>,
        starting_planet: u32
    ) -> Result<Self, String> {
        Ok(Self {
            id: ExplorerId::new(id),
            rx_orchestrator,
            tx_orchestrator,
            rx_planet,
            tx_planet,
            bag: BagContent::default(),
            running: false,
            mapping: Mapping::new(starting_planet),
        })
    }

    pub fn run(&mut self) {
        loop {
            select! {
                recv(self.rx_orchestrator) -> msg => {
                    if let Ok(message) = msg {
                        log_orchestrator_message(0, self.id, &format!("{message:?}"));
                        self.handle_orchestrator_message(message);
                    }
                }
                recv(self.rx_planet) -> msg => {
                    if let Ok(message) = msg {
                        log_planet_message(self.mapping.explorer_position.0, self.id, &format!("{message:?}"));
                        self.handle_planet_message(message);
                    }
                }
            }
        }
    }

    fn handle_orchestrator_message(&mut self, msg: OrchestratorToExplorer) {
        use OrchestratorToExplorer::*;
        match msg {
            StartExplorerAI => {
                log_explorer_event(self.id, "StartExplorerAI received", common_game::logging::Channel::Info);
                self.running = true;
                let _ = self.tx_orchestrator.send(
                    ExplorerToOrchestrator::StartExplorerAIResult { explorer_id: self.id.0 }
                );

                let _ = self.tx_planet.send(
                    ExplorerToPlanet::SupportedResourceRequest { explorer_id : self.id.0}
                );

                let _ = self.tx_planet.send(
                    ExplorerToPlanet::SupportedCombinationRequest { explorer_id : self.id.0}
                );

                let _ = self.tx_planet.send(
                    ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id : self.id.0}
                );
            }
            StopExplorerAI => {
                log_explorer_event(self.id, "StopExplorerAI received", common_game::logging::Channel::Info);
                self.running = false;
                let _ = self.tx_orchestrator.send(
                    ExplorerToOrchestrator::StopExplorerAIResult { explorer_id: self.id.0 }
                );
            }
            GenerateResourceRequest { to_generate } => {
                log_explorer_event(self.id, &format!("GenerateResourceRequest {:?}", to_generate), common_game::logging::Channel::Debug);
                let _ = self.tx_planet.send(
                    ExplorerToPlanet::GenerateResourceRequest {explorer_id : self.id.0, resource : to_generate}
                );
            }
            CombineResourceRequest { to_generate } => {
                log_explorer_event(self.id, &format!("CombineResourceRequest {:?}", to_generate), common_game::logging::Channel::Debug);
                if self.bag.can_create_complex(to_generate) {
                    match to_generate {
                        ComplexResourceType::Water => {
                            if let (Some(hydrogen), Some(oxygen)) = (
                                self.bag.take_basic(BasicResourceType::Hydrogen),
                                self.bag.take_basic(BasicResourceType::Oxygen)
                            ) {
                                let _ = self.tx_planet.send(
                                    ExplorerToPlanet::CombineResourceRequest {
                                        explorer_id: self.id.0,
                                        msg: ComplexResourceRequest::Water(hydrogen.to_hydrogen().unwrap(), oxygen.to_oxygen().unwrap()),
                                    }
                                );
                            }
                        }
                        ComplexResourceType::Diamond => {
                            if let (Some(carbon1), Some(carbon2)) = (
                                self.bag.take_basic(BasicResourceType::Carbon),
                                self.bag.take_basic(BasicResourceType::Carbon)
                            ) {
                                let _ = self.tx_planet.send(
                                    ExplorerToPlanet::CombineResourceRequest {
                                        explorer_id: self.id.0,
                                        msg: ComplexResourceRequest::Diamond(carbon1.to_carbon().unwrap(), carbon2.to_carbon().unwrap())
                                    }
                                );
                            }
                        }
                        ComplexResourceType::Life => {
                            if let (Some(water), Some(carbon)) = (
                                self.bag.take_complex(ComplexResourceType::Water),
                                self.bag.take_basic(BasicResourceType::Carbon)
                            ) {
                                let _ = self.tx_planet.send(
                                    ExplorerToPlanet::CombineResourceRequest {
                                        explorer_id: self.id.0,
                                        msg: ComplexResourceRequest::Life(water.to_water().unwrap(), carbon.to_carbon().unwrap())
                                    }
                                );
                            }
                        }
                        ComplexResourceType::Robot => {
                            if let (Some(silicon), Some(life)) = (
                                self.bag.take_basic(BasicResourceType::Silicon),
                                self.bag.take_complex(ComplexResourceType::Life)
                            ) {
                                let _ = self.tx_planet.send(
                                    ExplorerToPlanet::CombineResourceRequest {
                                        explorer_id: self.id.0,
                                        msg: ComplexResourceRequest::Robot(silicon.to_silicon().unwrap(), life.to_life().unwrap())
                                    }
                                );
                            }
                        }
                        ComplexResourceType::Dolphin => {
                            if let (Some(life), Some(water)) = (
                                self.bag.take_complex(ComplexResourceType::Life),
                                self.bag.take_complex(ComplexResourceType::Water)
                            ) {
                                let _ = self.tx_planet.send(
                                    ExplorerToPlanet::CombineResourceRequest {
                                        explorer_id: self.id.0,
                                        msg: ComplexResourceRequest::Dolphin(water.to_water().unwrap(), life.to_life().unwrap())
                                    }
                                );
                            }
                        }
                        ComplexResourceType::AIPartner => {}
                    }
                }
            }
            CurrentPlanetRequest => {
                let _ = self.tx_orchestrator.send(
                    ExplorerToOrchestrator::CurrentPlanetResult { explorer_id: self.id.0, planet_id: self.mapping.explorer_position.0 }
                );
            }
            SupportedResourceRequest => {
                let _ = self.tx_planet.send(
                    ExplorerToPlanet::SupportedResourceRequest {
                        explorer_id: self.id.0,
                    }
                );
            }
            SupportedCombinationRequest => {
                let _ = self.tx_planet.send(
                    ExplorerToPlanet::SupportedCombinationRequest {
                        explorer_id: self.id.0,
                    }
                );
            }
            BagContentRequest => {
                let _ = self.tx_orchestrator.send(
                    ExplorerToOrchestrator::BagContentResponse {
                        explorer_id: self.id.0,
                        bag_content: self.bag.present_resource_types(),
                    }
                );
            }
            NeighborsResponse { neighbors} => {
                for neighbor in neighbors {
                    self.mapping.connect(self.mapping.explorer_position, PlanetNodeId(neighbor), 1)
                }
                for r in self.bag.missing_resource_types() {
                    if let Some(destination) = self.mapping.next_hop_to_resource(r){
                        let _ = self.tx_orchestrator.send(
                            ExplorerToOrchestrator::TravelToPlanetRequest {
                                explorer_id: self.id.0,
                                current_planet_id: self.mapping.explorer_position.0,
                                dst_planet_id: destination.0,
                            }
                        );
                        break;
                    }else{
                        //change state to final
                    }
                }


            }
            MoveToPlanet { sender_to_new_planet, planet_id } => {
                if let Some(sender) = sender_to_new_planet {
                    self.tx_planet = sender; // aggiorna il canale attivo
                    self.mapping.set_explorer_position(PlanetNodeId(planet_id.into()));
                }
            }
            ResetExplorerAI => {

            }
            KillExplorer => {
                let _ = self.tx_orchestrator.send(
                    ExplorerToOrchestrator::KillExplorerResult {
                        explorer_id: self.id.0,
                    }
                );
            }
        }
    }

    fn handle_planet_message(&mut self, msg: PlanetToExplorer) {
        match msg {
            PlanetToExplorer::SupportedResourceResponse { resource_list } => {
                self.mapping.set_basic_resources_for_planet(resource_list)
            }
            PlanetToExplorer::SupportedCombinationResponse { combination_list } => {
                self.mapping.set_complex_resources_for_planet(combination_list)
            }
            PlanetToExplorer::GenerateResourceResponse { resource } => {
                if self.running{
                    match resource {
                        Some(resource) => {
                            self.bag.add_basic(resource);
                        },
                        None => {
                            let _ = self.tx_orchestrator.send(
                                ExplorerToOrchestrator::NeighborsRequest {
                                    explorer_id : self.id.0,
                                    current_planet_id : self.mapping.explorer_position.0
                                }
                            );
                        }
                    }
                }
            }
            PlanetToExplorer::CombineResourceResponse { complex_response } => {
                if self.running{
                    match complex_response {
                        Ok(complex_response) => {
                            self.bag.add_complex(complex_response);
                        },
                        Err((error, r1, r2)) => {
                            self.add_generic_resource(r1);
                            self.add_generic_resource(r2);
                            let _ = self.tx_orchestrator.send(
                                ExplorerToOrchestrator::NeighborsRequest {
                                    explorer_id : self.id.0,
                                    current_planet_id : self.mapping.explorer_position.0
                                }
                            );
                        }
                    }
                }
            }
            PlanetToExplorer::AvailableEnergyCellResponse { available_cells } => {
                if self.running{
                    if available_cells == 0 {
                        let _ = self.tx_orchestrator.send(
                            ExplorerToOrchestrator::NeighborsRequest {
                                explorer_id : self.id.0,
                                current_planet_id : self.mapping.explorer_position.0
                            }
                        );
                    }else{
                        for r in self.bag.missing_resource_types(){
                            if self.mapping.current_planet_can_produce(r){
                                match r{
                                    ResourceType::Basic(basic_type) => {
                                        let _ = self.tx_planet.send(
                                        ExplorerToPlanet::GenerateResourceRequest {explorer_id : self.id.0, resource : basic_type}
                                        );
                                        break;
                                    },
                                    ResourceType::Complex(complex_type) => {
                                        if self.bag.can_create_complex(complex_type){
                                            match complex_type {
                                                ComplexResourceType::Water => {
                                                    if let (Some(hydrogen), Some(oxygen)) = (
                                                        self.bag.take_basic(BasicResourceType::Hydrogen),
                                                        self.bag.take_basic(BasicResourceType::Oxygen)
                                                    ) {
                                                        let _ = self.tx_planet.send(
                                                            ExplorerToPlanet::CombineResourceRequest {
                                                                explorer_id: self.id.0,
                                                                msg: ComplexResourceRequest::Water(hydrogen.to_hydrogen().unwrap(), oxygen.to_oxygen().unwrap()),
                                                            }
                                                        );
                                                    }
                                                }
                                                ComplexResourceType::Diamond => {
                                                    if let (Some(carbon1), Some(carbon2)) = (
                                                        self.bag.take_basic(BasicResourceType::Carbon),
                                                        self.bag.take_basic(BasicResourceType::Carbon)
                                                        ){
                                                        let _ = self.tx_planet.send(
                                                            ExplorerToPlanet::CombineResourceRequest {
                                                                explorer_id: self.id.0,
                                                                msg: ComplexResourceRequest::Diamond(carbon1.to_carbon().unwrap(), carbon2.to_carbon().unwrap())
                                                            }
                                                        );
                                                    }
                                                }
                                                ComplexResourceType::Life => {
                                                    if let (Some(water), Some(carbon)) = (
                                                        self.bag.take_complex(ComplexResourceType::Water),
                                                        self.bag.take_basic(BasicResourceType::Carbon)
                                                        ){
                                                        let _ = self.tx_planet.send(
                                                            ExplorerToPlanet::CombineResourceRequest {
                                                                explorer_id: self.id.0,
                                                                msg: ComplexResourceRequest::Life(water.to_water().unwrap(), carbon.to_carbon().unwrap())
                                                            }
                                                        );
                                                    }
                                                }
                                                ComplexResourceType::Robot => {
                                                    if let (Some(silicon), Some(life)) = (
                                                        self.bag.take_basic(BasicResourceType::Silicon),
                                                        self.bag.take_complex(ComplexResourceType::Life)
                                                        ) {
                                                        let _ = self.tx_planet.send(
                                                            ExplorerToPlanet::CombineResourceRequest {
                                                                explorer_id: self.id.0,
                                                                msg: ComplexResourceRequest::Robot(silicon.to_silicon().unwrap(), life.to_life().unwrap())
                                                            }
                                                        );
                                                    }
                                                }
                                                ComplexResourceType::Dolphin => {
                                                    if let (Some(life), Some(water)) = (
                                                        self.bag.take_complex(ComplexResourceType::Life),
                                                        self.bag.take_complex(ComplexResourceType::Water)
                                                        ){
                                                        let _ = self.tx_planet.send(
                                                            ExplorerToPlanet::CombineResourceRequest {
                                                                explorer_id: self.id.0,
                                                                msg: ComplexResourceRequest::Dolphin(water.to_water().unwrap(), life.to_life().unwrap())
                                                            }
                                                        );
                                                    }
                                                }
                                                ComplexResourceType::AIPartner => {}
                                            }
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        let _ = self.tx_orchestrator.send(
                            ExplorerToOrchestrator::NeighborsRequest {
                                explorer_id : self.id.0,
                                current_planet_id : self.mapping.explorer_position.0
                            }
                        );
                    }
                }
            }
            PlanetToExplorer::Stopped => {
                let _ = self.tx_orchestrator.send(
                    ExplorerToOrchestrator::NeighborsRequest {
                        explorer_id : self.id.0,
                        current_planet_id : self.mapping.explorer_position.0
                    }
                );
            }
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


