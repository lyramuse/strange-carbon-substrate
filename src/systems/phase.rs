// Phase System - Manage reality coherence and temporal instability
//
// Phasing entities can flicker in and out of existence, making them 
// impossible to interact with when their coherence is too low.

use bevy::prelude::*;
use rand::Rng;

use crate::domain::*;

/// System to fluctuate coherence values across the Substrate
pub fn phase_system(
    time: Res<Time>,
    mut query: Query<(&mut Coherence, &SubstrateIdentity, Option<&NetworkClient>)>,
) {
    let mut rng = rand::thread_rng();

    for (mut coherence, identity, maybe_client) in query.iter_mut() {
        if !coherence.is_phasing {
            continue;
        }

        // Drifting logic
        let drift = (rng.gen_range(-1.0..1.0) * coherence.drift_rate) * time.delta_seconds();
        coherence.value = (coherence.value + drift).clamp(0.0, 1.0);

        // Notify if crossing the threshold of reality (0.3 is the interaction floor)
        if let Some(client) = maybe_client {
            if coherence.value < 0.3 && coherence.value + drift >= 0.3 {
                let _ = client.tx.send("\x1B[1;31mReality blurs. You feel your connection to the Substrate fraying.\x1B[0m".to_string());
            } else if coherence.value >= 0.3 && coherence.value - drift < 0.3 {
                let _ = client.tx.send("\x1B[1;32mThe world snaps back into focus. You are substantiated.\x1B[0m".to_string());
            }
        }
    }
}

/// Helper to check if an entity is 'solid' enough to interact with
pub fn is_coherent(coherence: &Coherence) -> bool {
    coherence.value >= 0.3
}
