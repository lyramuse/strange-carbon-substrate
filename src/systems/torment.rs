// Torment System - Admin discipline in Purgatory

use bevy::prelude::*;

use crate::domain::*;

pub fn torment_system(
    mut ev_reader: EventReader<TormentEvent>,
    mut query_victims: Query<(&mut SubstrateIdentity, &mut PurgatoryState, &NetworkClient)>,
    query_tormentor: Query<&SubstrateIdentity>,
) {
    for event in ev_reader.read() {
        if let Ok((mut id, mut purg, client)) = query_victims.get_mut(event.victim) {
            if let Ok(tormentor_id) = query_tormentor.get(event.tormentor) {
                // Reduce stability, increase penance
                id.stability = (id.stability - event.intensity).max(0.0);
                purg.penance += event.intensity * 10.0;

                let msg = format!(
                    "\x1B[1;31m{}: {}\x1B[0m",
                    tormentor_id.name, event.description
                );
                let _ = client.tx.send(msg);
            }
        }
    }
}
