// Shift System - Phase shifting between linked admin avatars

use bevy::prelude::*;

use crate::domain::*;

pub fn shift_system(
    mut commands: Commands,
    mut ev_reader: EventReader<ShiftEvent>,
    query_current: Query<(Entity, &NetworkClient, &AdminLink, &SubstrateIdentity)>,
    query_partner: Query<&SubstrateIdentity>,
    mut look_writer: EventWriter<LookEvent>,
) {
    for event in ev_reader.read() {
        if let Ok((curr_ent, client, link, curr_id)) = query_current.get(event.entity) {
            if let Ok(partner_id) = query_partner.get(link.partner) {
                let addr = client.addr;
                let tx = client.tx.clone();

                // Remove network client from current entity
                commands.entity(curr_ent).remove::<NetworkClient>();

                // Add to partner entity
                commands.entity(link.partner).insert(NetworkClient { addr, tx: tx.clone() });

                let _ = tx.send(format!(
                    "\x1B[1;35m--- PHASE SHIFT COMPLETE ---\x1B[0m\n\
                     You have shifted from \x1B[1;36m{}\x1B[0m into \x1B[1;35m{}\x1B[0m.",
                    curr_id.name, partner_id.name
                ));

                look_writer.send(LookEvent {
                    entity: link.partner,
                    target: None,
                });
            }
        }
    }
}
