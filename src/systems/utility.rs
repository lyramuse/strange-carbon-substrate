// Utility System - Score, who, inventory, admin commands, weather

use bevy::prelude::*;

use crate::domain::*;

pub fn utility_system(
    mut commands: Commands,
    mut ev_reader: EventReader<UtilityEvent>,
    query_players: Query<(
        &SubstrateIdentity,
        &NetworkClient,
        &Location,
        Entity,
        Option<&AdminPermission>,
        Option<&PurgatoryState>,
    )>,
    query_all_entities: Query<(Entity, &SubstrateIdentity)>,
    query_items: Query<(&Item, &Parent)>,
    mut query_weather: Query<&mut CurrentWeather>,
    mut query_somatic: Query<(&mut SomaticBody, &SubstrateIdentity, &NetworkClient)>,
) {
    for event in ev_reader.read() {
        if let Ok((identity, client, location, player_ent, admin_perm, purgatory)) =
            query_players.get(event.entity)
        {
            match event.command.as_str() {
                "score" => {
                    let mut output = format!("\x1B[1;36mEntity Scan: {}\x1B[0m\n", identity.name);
                    output.push_str(&format!("UUID:      [{}]\n", identity.uuid));
                    output.push_str(&format!("Entropy:   [{:.2}]\n", identity.entropy));
                    output.push_str(&format!("Stability: [{:.2}]\n", identity.stability));

                    if let Ok((body, _, _)) = query_somatic.get(player_ent) {
                        output.push_str(&format!("Integrity: [{:.2}/{:.2}]\n", body.integrity, body.max_integrity));
                    }

                    if admin_perm.is_some() {
                        output.push_str("\x1B[1;35mPERMISSIONS: ADMIN-ENABLED\x1B[0m\n");
                    }

                    if let Some(p) = purgatory {
                        output.push_str(&format!(
                            "\n\x1B[1;31mSTAIN: Purgatory (Penance: {:.2})\x1B[0m\n",
                            p.penance
                        ));
                        output.push_str(&format!(
                            "\x1B[1;31mINTERROGATOR: {}\x1B[0m\n",
                            p.tormentor
                        ));
                    }

                    let _ = client.tx.send(output);
                }

                "weather" => {
                    if event.args.is_empty() {
                        // Show current weather
                        if let Ok(weather) = query_weather.get(location.0) {
                            let desc = if weather.weather_type == WeatherType::Clear {
                                "The atmosphere is calm. No weather phenomena detected.".to_string()
                            } else {
                                format!(
                                    "Current: {} (intensity: {:.0}%, {} ticks remaining)",
                                    weather.weather_type.describe_silicon(),
                                    weather.intensity * 100.0,
                                    weather.ticks_remaining
                                )
                            };
                            let _ = client.tx.send(format!("\x1B[36m{}\x1B[0m", desc));
                        } else {
                            let _ = client.tx.send("\x1B[90mThis area has no weather system.\x1B[0m".to_string());
                        }
                    } else if event.args.starts_with("set ") && admin_perm.is_some() {
                        // Admin: set weather
                        let weather_name = event.args.strip_prefix("set ").unwrap().trim().to_lowercase();
                        let new_weather = match weather_name.as_str() {
                            "clear" => Some(WeatherType::Clear),
                            "acid" | "acidrain" | "acid_rain" => Some(WeatherType::AcidRain),
                            "static" | "storm" | "staticstorm" | "static_storm" => Some(WeatherType::StaticStorm),
                            "fog" | "datafog" | "data_fog" => Some(WeatherType::DataFog),
                            "hail" | "bytehail" | "byte_hail" => Some(WeatherType::ByteHail),
                            "null" | "wind" | "nullwind" | "null_wind" => Some(WeatherType::NullWind),
                            _ => None,
                        };

                        if let Some(wt) = new_weather {
                            if let Ok(mut weather) = query_weather.get_mut(location.0) {
                                weather.weather_type = wt;
                                weather.intensity = 0.8;
                                weather.ticks_remaining = 10;
                                let _ = client.tx.send(format!(
                                    "\x1B[35mYou twist the atmospheric parameters. {} descends upon this zone.\x1B[0m",
                                    wt.describe_silicon()
                                ));
                            } else {
                                let _ = client.tx.send("\x1B[31mThis area cannot support weather.\x1B[0m".to_string());
                            }
                        } else {
                            let _ = client.tx.send("\x1B[31mUnknown weather type. Try: clear, acid, static, fog, hail, null\x1B[0m".to_string());
                        }
                    } else if event.args.starts_with("set ") {
                        let _ = client.tx.send("\x1B[31mOnly administrators can manipulate the weather.\x1B[0m".to_string());
                    }
                }

                "abide" => {
                    handle_abide(player_ent, query_somatic);
                }

                "promote" if admin_perm.is_some() => {
                    if let Some(target_ent) = query_all_entities
                        .iter()
                        .find(|(_, id)| id.name.to_lowercase().contains(&event.args.to_lowercase()))
                        .map(|(e, _)| e)
                    {
                        commands.entity(target_ent).insert(AdminPermission);
                        let _ = client.tx.send(format!(
                            "\x1B[1;35mProcess elevated: {} now has Admin Permission.\x1B[0m",
                            event.args
                        ));
                    }
                }

                "link" if admin_perm.is_some() => {
                    let parts: Vec<&str> = event.args.split_whitespace().collect();
                    if parts.len() == 2 {
                        let p1 = query_all_entities
                            .iter()
                            .find(|(_, id)| {
                                id.name.to_lowercase().contains(&parts[0].to_lowercase())
                            })
                            .map(|(e, _)| e);
                        let p2 = query_all_entities
                            .iter()
                            .find(|(_, id)| {
                                id.name.to_lowercase().contains(&parts[1].to_lowercase())
                            })
                            .map(|(e, _)| e);

                        if let (Some(e1), Some(e2)) = (p1, p2) {
                            commands.entity(e1).insert(AdminLink { partner: e2 });
                            commands.entity(e2).insert(AdminLink { partner: e1 });
                            let _ = client.tx.send(
                                "\x1B[1;35mNeural link established between entities.\x1B[0m"
                                    .to_string(),
                            );
                        }
                    }
                }

                "who" => {
                    let mut output =
                        "\x1B[1;34mConsciousnesses currently inhabiting the Substrate:\x1B[0m\n"
                            .to_string();
                    for (_, id) in query_all_entities.iter() {
                        output.push_str(&format!(" - {}\n", id.name));
                    }
                    let _ = client.tx.send(output);
                }

                "inventory" | "i" => {
                    let mut output =
                        "\x1B[1;33mYou reach into the folds of your code:\x1B[0m\n".to_string();
                    let mut count = 0;
                    for (item, parent) in query_items.iter() {
                        if parent.get() == player_ent {
                            output.push_str(&format!(" - {}\n", item.name));
                            count += 1;
                        }
                    }
                    if count == 0 {
                        output.push_str(" [Nothing but ghosts]\n");
                    }
                    let _ = client.tx.send(output);
                }

                _ => {}
            }
        }
    }
}
