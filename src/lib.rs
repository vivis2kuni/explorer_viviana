mod explorer;
mod logging;

pub fn create_explorer(
    id: u32,
    rx_orchestrator: crossbeam_channel::Receiver<common_game::protocols::orchestrator_explorer::OrchestratorToExplorer>,
    tx_orchestrator: crossbeam_channel::Sender<common_game::protocols::explorer_orchestrator::ExplorerToOrchestrator<Vec<(common_game::components::resource::ResourceType, usize)>>>,
    rx_planet: crossbeam_channel::Receiver<common_game::protocols::planet_explorer::PlanetToExplorer>,
    tx_planet: crossbeam_channel::Sender<common_game::protocols::planet_explorer::ExplorerToPlanet>,
    starting_planet: u32
) -> Result<explorer::Explorer, String> {
    use logging::log_explorer_event;
    use common_game::logging::{EventType, Channel};

    let explorer_id = explorer::ExplorerId::new(id);

    // Log initialization
    log_explorer_event(
        explorer_id,
        EventType::InternalExplorerAction,
        Channel::Info,
        "Explorer initializing",
        None::<[(&str, String); 0]>,
    );

    // Create Explorer instance
    let explorer = match explorer::Explorer::new(
        id,
        rx_orchestrator,
        tx_orchestrator,
        rx_planet,
        tx_planet,
        starting_planet
    ) {
        Ok(ex) => ex,
        Err(e) => return Err(format!("Failed to create Explorer: {}", e)),
    };

    // Log successful initialization
    log_explorer_event(
        explorer_id,
        EventType::InternalExplorerAction,
        Channel::Info,
        "Explorer initialization complete",
        None::<[(&str, String); 0]>,
    );

    Ok(explorer)
}
