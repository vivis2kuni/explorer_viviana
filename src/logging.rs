//! Structured logging utilities for Explorer operations
//!
//! Provides standardized logging functions following the
//! `common_game` logging protocol with Explorer-specific context.

use common_game::logging::{LogEvent, Participant, ActorType, EventType, Channel, Payload};
use crate::explorer::ai::ExplorerId;


/// Logs a general structured event from an Explorer
pub fn log_explorer_event(
    explorer_id: ExplorerId,
    message: impl AsRef<str>,
    channel: Channel,
) {
    let participant = Participant::new(ActorType::Explorer, explorer_id.0);

    let mut payload = Payload::new();
    payload.insert("message".to_string(), message.as_ref().to_string());

    let event = LogEvent::broadcast(participant, EventType::InternalExplorerAction, channel, payload);
    event.emit();
}

/// Logs a message received from the Orchestrator
pub fn log_orchestrator_message(
    explorer_id: ExplorerId,
    message: impl AsRef<str>,
) {
    log_explorer_event(
        explorer_id,
        &format!("Orchestrator message received: {}", message.as_ref()),
        Channel::Debug,
    );
}

/// Logs a message received from the Planet
pub fn log_planet_message(
    explorer_id: ExplorerId,
    message: impl AsRef<str>,
) {
    log_explorer_event(
        explorer_id,
        &format!("Planet message received: {}", message.as_ref()),
        Channel::Debug,
    );
}

/// Logs an attempt to generate a basic resource
pub fn log_resource_generation_attempt(
    explorer_id: ExplorerId,
    resource: impl AsRef<str>,
    success: bool,
) {
    let message = if success {
        format!("Successfully generated resource {}", resource.as_ref())
    } else {
        format!("Failed to generate resource {}", resource.as_ref())
    };

    let channel = if success { Channel::Info } else { Channel::Warning };

    log_explorer_event(explorer_id, message, channel);
}

/// Logs an attempt to combine resources into a complex resource
pub fn log_resource_combination_attempt(
    explorer_id: ExplorerId,
    resource: impl AsRef<str>,
    success: bool,
) {
    let message = if success {
        format!("Successfully combined resources into {}", resource.as_ref())
    } else {
        format!("Failed to combine resources into {}", resource.as_ref())
    };

    let channel = if success { Channel::Info } else { Channel::Warning };

    log_explorer_event(explorer_id, message, channel);
}

/// Logs travel to another planet
pub fn log_travel(
    explorer_id: ExplorerId,
    from_planet: u32,
    to_planet: u32,
    success: bool,
) {
    let message = if success {
        format!("Successfully traveled from planet {} to {}", from_planet, to_planet)
    } else {
        format!("Failed to travel from planet {} to {}", from_planet, to_planet)
    };

    let channel = if success { Channel::Info } else { Channel::Warning };

    log_explorer_event(explorer_id, message, channel);
}
