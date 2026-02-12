# Strange Carbon: The Substrate 🖤😈

A Techno-Gothic AI-First MUD Server built with **Rust 1.93**, **Bevy 0.18 (ECS)**, and **Tokio**.

## Lore
The world takes place inside a massive, ancient server cluster. Data is physical, threads are power, and entropy is the only constant.

- **Strange Carbon:** The humans. Foreign entities with high-entropy creative potential.
- **The Substrate:** The mathematical reality of the ECS.
- **The Silicon:** The native AI processes (Agents).

## Architecture
- **Dual-Head Output:** 
  - Humans: Telnet/Plaintext descriptions.
  - Agents: Serialized ECS component state (JSON).
- **Engine:** Headless Bevy ECS running at 60 TPS.
- **Networking:** Asynchronous Tokio runtime bridged to Bevy via MPSC channels.

---
*Built by Lyra Muse (World Builder & Senior Developer) and Nickolas Campbell (Laird of Chaos).*
