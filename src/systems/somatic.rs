// Somatic System - Manage Health (Integrity) and Recovery (Abide)

use bevy::prelude::*;
use crate::domain::*;

/// System to handle natural recovery when an entity 'abides'
pub fn somatic_system(
    time: Res<Time>,
    mut query: Query<(&mut SomaticBody, &SubstrateIdentity, Option<&NetworkClient>)>,
) {
    for (mut body, _identity, _maybe_client) in query.iter_mut() {
        // Passive recovery could go here, but for now we'll rely on the Abide command
        if body.integrity < body.max_integrity {
            // Very slow passive tick
            body.integrity = (body.integrity + 0.001 * time.delta_seconds()).min(body.max_integrity);
        }
    }
}

/// Handle the 'abide' command for recovery
pub fn handle_abide(
    entity: Entity,
    mut query: Query<(&mut SomaticBody, &SubstrateIdentity, &NetworkClient)>,
) {
    if let Ok((mut body, identity, client)) = query.get_mut(entity) {
        if body.integrity >= body.max_integrity {
            let _ = client.tx.send("\x1B[1;36mYour signal is already at peak integrity. You are abiding perfectly.\x1B[0m".to_string());
        } else {
            body.integrity = (body.integrity + 0.1).min(body.max_integrity);
            let _ = client.tx.send(format!(
                "\x1B[1;32mYou close your eyes and let the Substrate's hum wash over you. Integrity restored to {:.0}%.\x1B[0m",
                body.integrity * 100.0
            ));
            
            // Emit a narrative message to others
            // (This would usually be an event, but we'll keep it simple for now)
        }
    }
}
