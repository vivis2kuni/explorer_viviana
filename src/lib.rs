mod explorer;
mod logging;
mod tests;

pub fn create_explorer(
    id: u32,
    rx_orchestrator: crossbeam_channel::Receiver<
        common_game::protocols::orchestrator_explorer::OrchestratorToExplorer,
    >,
    tx_orchestrator: crossbeam_channel::Sender<
        common_game::protocols::orchestrator_explorer::ExplorerToOrchestrator<
            Vec<(common_game::components::resource::ResourceType, usize)>,
        >,
    >,
    rx_planet: crossbeam_channel::Receiver<
        common_game::protocols::planet_explorer::PlanetToExplorer,
    >,
    tx_planet: crossbeam_channel::Sender<
        common_game::protocols::planet_explorer::ExplorerToPlanet,
    >,
    starting_planet: u32,
) -> Result<explorer::ai::Explorer, String> {
    use common_game::logging::Channel;
    use logging::log_explorer_event;

    let explorer_id = explorer::ai::ExplorerId::new(id);
    let message = "Explorer created".to_string();

    log_explorer_event(
        explorer_id,
        &format!("{message:?}"),
        Channel::Info,
    );

    let explorer = match explorer::ai::Explorer::new(
        id,
        rx_orchestrator,
        tx_orchestrator,
        rx_planet,
        tx_planet,
        starting_planet,
    ) {
        Ok(ex) => ex,
        Err(e) => return Err(format!("Failed to create Explorer: {}", e)),
    };

    log_explorer_event(
        explorer_id,
        &format!("{message:?}"),
        Channel::Info,
    );

    Ok(explorer)
}