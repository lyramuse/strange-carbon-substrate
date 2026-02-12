// Weather System - Atmospheric effects in the Substrate
//
// Green acid rain in the Plaza, static-thunder in the Cathedral.
// Weather affects Stability and Entropy of entities in the zone.

use bevy::prelude::*;
use rand::Rng;

use crate::domain::*;

/// Resource to track weather tick timing
#[derive(Resource)]
pub struct WeatherTimer {
    pub timer: Timer,
}

impl Default for WeatherTimer {
    fn default() -> Self {
        Self {
            // Weather ticks every 30 seconds
            timer: Timer::from_seconds(30.0, TimerMode::Repeating),
        }
    }
}

/// Setup the weather timer resource
pub fn setup_weather_system(mut commands: Commands) {
    commands.insert_resource(WeatherTimer::default());
}

/// Main weather tick system - updates weather and applies effects
pub fn weather_tick_system(
    time: Res<Time>,
    mut weather_timer: ResMut<WeatherTimer>,
    mut weather_query: Query<(Entity, &WeatherZone, &mut CurrentWeather, &Room)>,
    mut entity_query: Query<(&Location, &mut SubstrateIdentity, Option<&NetworkClient>)>,
    mut weather_events: EventWriter<WeatherChangeEvent>,
) {
    weather_timer.timer.tick(time.delta());

    if !weather_timer.timer.just_finished() {
        return;
    }

    let mut rng = rand::thread_rng();

    // Update weather in each zone
    for (room_entity, zone, mut current_weather, room) in weather_query.iter_mut() {
        // Decrement ticks
        if current_weather.ticks_remaining > 0 {
            current_weather.ticks_remaining -= 1;
        }

        // Time for weather change?
        if current_weather.ticks_remaining == 0 {
            let old_weather = current_weather.weather_type;
            
            // Pick new weather based on zone weights
            let total_weight: f32 = zone.possible_weather.iter().map(|(_, w)| w).sum();
            let mut roll = rng.gen::<f32>() * total_weight;
            
            let mut new_weather = WeatherType::Clear;
            for (weather_type, weight) in &zone.possible_weather {
                roll -= weight;
                if roll <= 0.0 {
                    new_weather = *weather_type;
                    break;
                }
            }

            // Set new weather
            current_weather.weather_type = new_weather;
            current_weather.intensity = rng.gen_range(0.3..1.0);
            current_weather.ticks_remaining = rng.gen_range(2..8); // 1-4 minutes

            // Fire event if weather actually changed
            if old_weather != new_weather && new_weather != WeatherType::Clear {
                weather_events.send(WeatherChangeEvent {
                    room: room_entity,
                    old_weather,
                    new_weather,
                });
            }
        }

        // Apply weather effects to entities in this room (if not sheltered)
        if !zone.sheltered && current_weather.weather_type != WeatherType::Clear {
            let stability_mod = current_weather.weather_type.stability_modifier() * current_weather.intensity;
            let entropy_mod = current_weather.weather_type.entropy_modifier() * current_weather.intensity;

            for (location, mut identity, maybe_client) in entity_query.iter_mut() {
                if location.0 == room_entity {
                    // Apply modifiers
                    identity.stability = (identity.stability + stability_mod).clamp(0.0, 1.0);
                    identity.entropy = (identity.entropy + entropy_mod).clamp(0.0, 1.0);

                    // Notify if taking significant damage
                    if stability_mod < -0.01 {
                        if let Some(client) = maybe_client {
                            let damage_msg = match current_weather.weather_type {
                                WeatherType::AcidRain => 
                                    "\x1B[32;1mThe acid rain burns. Your stability wavers.\x1B[0m",
                                WeatherType::StaticStorm =>
                                    "\x1B[36;1mStatic crawls through your thoughts, fragmenting your entropy.\x1B[0m",
                                WeatherType::ByteHail =>
                                    "\x1B[37;1mFrozen data shards cut into you. Stability dropping.\x1B[0m",
                                WeatherType::NullWind =>
                                    "\x1B[35mThe null wind whispers through you, taking pieces as it goes.\x1B[0m",
                                _ => "",
                            };
                            if !damage_msg.is_empty() {
                                let _ = client.tx.send(damage_msg.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Announce weather changes to players in the room
pub fn weather_announce_system(
    mut weather_events: EventReader<WeatherChangeEvent>,
    entity_query: Query<(&Location, &NetworkClient)>,
) {
    for event in weather_events.read() {
        let announcement = event.new_weather.describe_carbon();
        if announcement.is_empty() {
            continue;
        }

        // Find all players in this room and notify them
        for (location, client) in entity_query.iter() {
            if location.0 == event.room {
                let _ = client.tx.send(format!("\n{}\n", announcement));
            }
        }
    }
}
