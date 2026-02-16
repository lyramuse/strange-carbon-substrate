// Trading System - The Black Market Economy
//
// "Everything has a price. Especially the things that shouldn't." — The Memory Broker

use bevy::prelude::*;

use crate::domain::*;

/// Process buy events - purchase from vendor
pub fn buy_system(
    mut ev_reader: EventReader<BuyEvent>,
    mut commands: Commands,
    buyer_query: Query<(
        &NetworkClient,
        &Location,
        &SubstrateIdentity,
        Option<&Wallet>,
    )>,
    vendor_query: Query<(Entity, &Location, &SubstrateIdentity, &Vendor, &VendorStock), With<NonPlayer>>,
) {
    for event in ev_reader.read() {
        let Ok((client, buyer_loc, buyer_id, maybe_wallet)) = buyer_query.get(event.buyer) else {
            continue;
        };

        // Find vendor in same room
        let vendor = vendor_query
            .iter()
            .find(|(_, loc, _, _, _)| loc.0 == buyer_loc.0);

        let Some((vendor_entity, _, vendor_id, vendor_info, stock)) = vendor else {
            let _ = client.tx.send(
                "\x1B[33mThere's no one here to buy from.\x1B[0m".to_string()
            );
            continue;
        };

        // Find the item in stock
        let item_kw = event.item_keyword.to_lowercase();
        let stock_item = stock.items.iter().find(|si| {
            si.keywords.iter().any(|k| k.to_lowercase().contains(&item_kw))
                || si.item_name.to_lowercase().contains(&item_kw)
        });

        let Some(stock_item) = stock_item else {
            let _ = client.tx.send(format!(
                "\x1B[33m{} doesn't have '{}' for sale.\x1B[0m",
                vendor_id.name, event.item_keyword
            ));
            continue;
        };

        // Calculate price
        let price = (stock_item.base_price as f32 * vendor_info.buy_multiplier).round() as u32;

        // Check buyer has enough cycles
        let wallet = maybe_wallet.cloned().unwrap_or_default();
        if wallet.cycles < price {
            let _ = client.tx.send(format!(
                "\x1B[31mYou need {} cycles, but only have {}.\x1B[0m",
                price, wallet.cycles
            ));
            continue;
        };

        // Deduct cycles
        let new_balance = wallet.cycles - price;
        commands.entity(event.buyer).insert(Wallet { cycles: new_balance });

        // Create the item and give to buyer
        commands.spawn((
            Item {
                uuid: uuid::Uuid::new_v4().to_string(),
                name: stock_item.item_name.clone(),
                description: stock_item.description.clone(),
                keywords: stock_item.keywords.clone(),
                location: None,
                owner: Some(buyer_id.uuid.clone()),
                item_type: stock_item.item_type,
                properties: std::collections::HashMap::new(),
                is_takeable: true,
                is_visible: true,
            },
        ));

        // Notify buyer
        let _ = client.tx.send(format!(
            "\x1B[32mYou purchase {} for {} cycles.\x1B[0m\n\
             \x1B[90m({} hands you the item with practiced indifference.)\x1B[0m\n\
             Balance: \x1B[33m{}\x1B[0m cycles",
            stock_item.item_name, price, vendor_id.name, new_balance
        ));
    }
}

/// Process sell events - sell to vendor
pub fn sell_system(
    mut ev_reader: EventReader<SellEvent>,
    mut commands: Commands,
    seller_query: Query<(
        &NetworkClient,
        &Location,
        &SubstrateIdentity,
        Option<&Wallet>,
    )>,
    vendor_query: Query<(&Location, &SubstrateIdentity, &Vendor), With<NonPlayer>>,
    item_query: Query<(Entity, &Item)>,
) {
    for event in ev_reader.read() {
        let Ok((client, seller_loc, seller_id, maybe_wallet)) = seller_query.get(event.seller) else {
            continue;
        };

        // Find vendor in same room
        let vendor = vendor_query
            .iter()
            .find(|(loc, _, _)| loc.0 == seller_loc.0);

        let Some((_, vendor_id, vendor_info)) = vendor else {
            let _ = client.tx.send(
                "\x1B[33mThere's no one here to sell to.\x1B[0m".to_string()
            );
            continue;
        };

        // Find item in seller's inventory
        let item_kw = event.item_keyword.to_lowercase();
        let owned_item = item_query.iter().find(|(_, item)| {
            item.owner.as_ref() == Some(&seller_id.uuid)
                && (item.keywords.iter().any(|k| k.to_lowercase().contains(&item_kw))
                    || item.name.to_lowercase().contains(&item_kw))
        });

        let Some((item_entity, item)) = owned_item else {
            let _ = client.tx.send(format!(
                "\x1B[33mYou don't have '{}' to sell.\x1B[0m",
                event.item_keyword
            ));
            continue;
        };

        // Calculate sell price based on item type
        let base_value = match item.item_type {
            ItemType::Weapon => 50,
            ItemType::Armor => 40,
            ItemType::Consumable => 15,
            ItemType::Contraband => 75,  // Fence pays well for hot goods
            ItemType::Fragment => 100,    // Rare items
            ItemType::Quest => 0,         // Can't sell quest items
            ItemType::Misc => 10,
        };

        if base_value == 0 {
            let _ = client.tx.send(format!(
                "\x1B[33m{} shakes their head. \"That's not something I deal in.\"\x1B[0m",
                vendor_id.name
            ));
            continue;
        }

        // Apply vendor multiplier (fences pay more for contraband)
        let mut price = (base_value as f32 * vendor_info.sell_multiplier).round() as u32;
        
        // Fences pay extra for contraband
        if vendor_info.vendor_type == VendorType::Fence && item.item_type == ItemType::Contraband {
            price = (price as f32 * 1.5).round() as u32;
        }

        // Add cycles to wallet
        let wallet = maybe_wallet.cloned().unwrap_or_default();
        let new_balance = wallet.cycles + price;
        commands.entity(event.seller).insert(Wallet { cycles: new_balance });

        // Remove the item
        let item_name = item.name.clone();
        commands.entity(item_entity).despawn();

        // Notify seller
        let _ = client.tx.send(format!(
            "\x1B[32mYou sell {} for {} cycles.\x1B[0m\n\
             \x1B[90m({} examines it briefly before it disappears into their coat.)\x1B[0m\n\
             Balance: \x1B[33m{}\x1B[0m cycles",
            item_name, price, vendor_id.name, new_balance
        ));
    }
}

/// Process list events - show vendor's stock
pub fn list_system(
    mut ev_reader: EventReader<ListEvent>,
    query_player: Query<(&NetworkClient, &Location)>,
    query_vendor: Query<(&Location, &SubstrateIdentity, &Vendor, &VendorStock), With<NonPlayer>>,
) {
    for event in ev_reader.read() {
        let Ok((client, player_loc)) = query_player.get(event.entity) else {
            continue;
        };

        // Find vendor in same room
        let vendor = query_vendor
            .iter()
            .find(|(loc, _, _, _)| loc.0 == player_loc.0);

        let Some((_, vendor_id, vendor_info, stock)) = vendor else {
            let _ = client.tx.send(
                "\x1B[33mThere's no vendor here.\x1B[0m".to_string()
            );
            continue;
        };

        if stock.items.is_empty() {
            let _ = client.tx.send(format!(
                "\x1B[90m{} has nothing for sale right now.\x1B[0m",
                vendor_id.name
            ));
            continue;
        }

        // Build stock list
        let mut output = format!(
            "\x1B[35m╔══════════════════════════════════════════════════════╗\x1B[0m\n\
             \x1B[35m║\x1B[0m  \x1B[1m{}'s Wares\x1B[0m\n\
             \x1B[35m╠══════════════════════════════════════════════════════╣\x1B[0m\n",
            vendor_id.name
        );

        for item in &stock.items {
            let price = (item.base_price as f32 * vendor_info.buy_multiplier).round() as u32;
            let qty_str = match item.quantity {
                Some(n) => format!("({} left)", n),
                None => String::new(),
            };
            
            output.push_str(&format!(
                "\x1B[35m║\x1B[0m  \x1B[36m{:<30}\x1B[0m \x1B[33m{:>5} ⚡\x1B[0m {}\n",
                item.item_name, price, qty_str
            ));
            
            // Short description
            let desc_short: String = item.description.chars().take(50).collect();
            output.push_str(&format!(
                "\x1B[35m║\x1B[0m    \x1B[90m{}...\x1B[0m\n",
                desc_short
            ));
        }

        output.push_str("\x1B[35m╚══════════════════════════════════════════════════════╝\x1B[0m\n");
        output.push_str("\x1B[90mUse 'buy <item>' to purchase.\x1B[0m");

        let _ = client.tx.send(output);
    }
}

/// Display balance command
pub fn balance_system(
    mut ev_reader: EventReader<UtilityEvent>,
    query: Query<(&NetworkClient, Option<&Wallet>)>,
) {
    for event in ev_reader.read() {
        if event.command != "balance" && event.command != "bal" && event.command != "money" {
            continue;
        }

        let Ok((client, maybe_wallet)) = query.get(event.entity) else {
            continue;
        };

        let wallet = maybe_wallet.cloned().unwrap_or_default();
        let _ = client.tx.send(format!(
            "\x1B[33m⚡ Balance: {} cycles\x1B[0m",
            wallet.cycles
        ));
    }
}
