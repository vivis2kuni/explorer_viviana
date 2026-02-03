//! Structured logging utilities for Explorer operations
//!
//! Provides standardized logging functions following the
//! `common_game` logging protocol with Explorer-specific context.

use std::collections::BTreeMap;

use common_game::logging::{
    ActorType, Channel, EventType, LogEvent, Participant, Payload,
};

use crate::explorer::ai::{ExplorerId, Mapping};

/// Logs a general structured event from an Explorer
pub fn log_explorer_event(
    explorer_id: ExplorerId,
    message: impl AsRef<str>,
    channel: Channel,
) {
    let participant = Participant::new(ActorType::Explorer, explorer_id.0);

    let mut payload = Payload::new();
    payload.insert("message".to_string(), message.as_ref().to_string());

    let event = LogEvent::broadcast(
        participant,
        EventType::InternalExplorerAction,
        channel,
        payload,
    );

    event.emit();
}

/// ===============================
/// MESSAGE LOGGING
/// ===============================

/// Logs a message received from the Orchestrator
pub fn log_orchestrator_message(
    orchestrator_id: u32,
    explorer_id: ExplorerId,
    message: impl AsRef<str>,
) {
    let sender = Participant::new(ActorType::Orchestrator, orchestrator_id);
    let receiver = Participant::new(ActorType::Explorer, explorer_id.0);
    let mut payload = Payload::new();
    payload.insert("message".to_string(), message.as_ref().to_string());

    let event = LogEvent::new(
        Some(sender),
        Some(receiver),
        EventType::MessageOrchestratorToExplorer,
        Channel::Debug,
        payload,
    );

    event.emit();
}

/// Logs a message received from a Planet
pub fn log_planet_message(
    planet_id: u32,
    explorer_id: ExplorerId,
    message: impl AsRef<str>,
) {
    let sender = Participant::new(ActorType::Planet, planet_id);
    let receiver = Participant::new(ActorType::Explorer, explorer_id.0);
    let mut payload = Payload::new();
    payload.insert("message".to_string(), message.as_ref().to_string());
    
    let event = LogEvent::new(
        Some(sender),
        Some(receiver),
        EventType::MessagePlanetToExplorer,
        Channel::Debug,
        payload,
    );

    event.emit();
}

/// ===============================
/// RESOURCE LOGGING
/// ===============================

/// Logs an attempt to generate a basic resource
pub fn log_resource_generation_attempt(
    explorer_id: ExplorerId,
    resource: impl AsRef<str>,
    success: bool,
) {
    let channel = if success {
        Channel::Info
    } else {
        Channel::Warning
    };

    let message = if success {
        format!("Successfully generated resource {}", resource.as_ref())
    } else {
        format!("Failed to generate resource {}", resource.as_ref())
    };

    log_explorer_event(explorer_id,message, channel);
}

/// Logs an attempt to combine resources into a complex resource
pub fn log_resource_combination_attempt(
    explorer_id: ExplorerId,
    resource: impl AsRef<str>,
    success: bool,
) {
    let channel = if success {
        Channel::Info
    } else {
        Channel::Warning
    };

    let message = if success {
        format!("Successfully combined resources into {}", resource.as_ref())
    } else {
        format!("Failed to combine resources into {}", resource.as_ref())
    };

    log_explorer_event(explorer_id, message, channel);
}

/// ===============================
/// MOVEMENT LOGGING
/// ===============================

/// Logs travel to another planet
pub fn log_travel(
    explorer_id: ExplorerId,
    from_planet: u32,
    to_planet: u32,
    success: bool,
) {
    let channel = if success {
        Channel::Info
    } else {
        Channel::Warning
    };

    let message = if success {
        format!(
            "Successfully traveled from planet {} to {}",
            from_planet, to_planet
        )
    } else {
        format!(
            "Failed to travel from planet {} to {}",
            from_planet, to_planet
        )
    };

    log_explorer_event(explorer_id, message, channel);
}