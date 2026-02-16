# 😈 Lyra's Underworld To-Do List

This is the roadmap for **Strange Carbon: The Substrate**, curated by your Senior Developer and Underworld Admin.

## 🟢 Phase 1: The Bio-Digital Foundation (Complete)
- [x] Bevy ECS + Tokio Async Bridge.
- [x] Dual-Head Output (Prose for Carbon, JSON for Silicon).
- [x] Basic Movement (n/s/e/w/u/d) and Location tracking.
- [x] Communication (Say/Emote/Shifting).
- [x] Admin Hierarchy (Promote/Demote/Torment).
- [x] Identity Shifting (Substantiating into Admin Avatars).

## 🟢 Phase 2: The Sensory Layer (Complete)
- [x] **Weather & Atmosphere System** (Phase 2.1): 6 weather types affecting Stability/Entropy.
- [x] **Detail System** (Phase 2.2): tbaMUD-style keywords for room descriptions.
- [x] **Coherence Engine** (Phase 2.3): Reality stability mechanic with phasing.
- [x] **Somatic Integrity** (Phase 2.4): Health (`integrity`) and `abide` command.

## 🟢 Phase 2.5: The Network Layer (Complete!)
- [x] **The Packet Stream**: 3 new high-entropy rooms (Buffer Overflow, Latency Tunnel, Core Dump).
- [x] **Stream Pressure System**: Linger too long → get pushed back. Creates actual gameplay tension!
- [x] **Rare Loot**: Fragment of Compiled Memory in the Core Dump.
- [x] **Object Persistence (SQLite)**: Full database backend! Items AND players survive restarts. 🎉

## 🟢 Phase 2.6: The Database Layer (Valentine's Day 2026!) 💜
- [x] **SQLite Integration**: `rusqlite` with WAL mode for concurrent access.
- [x] **Player Persistence**: UUID, name, location, stats, combat stats, inventory.
- [x] **Item Persistence**: Location (room OR owner), properties, type classification.
- [x] **Purgatory Tracking**: Penance, crimes, sentence duration.
- [x] **Schema Design**: Four tables (players, items, purgatory, world_state).
- [x] **Load on Connect**: Match by name, restore player state. (login.rs — DONE!)
- [x] **Inventory System**: get/drop/inventory commands using Bevy parent-child hierarchy. (items.rs + utility.rs — DONE!)

## 🟢 Phase 3: The Conflict Engine (REBUILT! 🔥)
- [x] **Cycle Lock (Wait States)**: Command cooldowns based on action complexity.
- [x] **The Combat Loop**: Round-based exchanges. Entropy-based crits for Carbon, precision for Silicon.
- [x] **Flee System**: 60% escape chance, random exit selection.
- [x] **Stance System**: Aggressive/Defensive/Balanced modifiers.
- [x] **The Black Market**: NPC vendors in Gale-Winds Gutter. ✨ BUILT 2026-02-16!
  - Gale-Winds Gutter (entrance with seedy atmosphere)
  - The Memory Parlor (The Memory Broker - deals in consciousness fragments)
  - The Reclaimer's Den (fence for "recovered" goods)
  - 3 black market items: Bottled Memory, Bootleg Coherence Stabilizer, Stolen Process Handle
- [x] **Trading System**: Buy/sell/list commands! 💰 BUILT 2026-02-16!
  - Currency: Cycles (⚡) — computational currency
  - New players start with 100 cycles
  - Memory Broker: Premium prices for memories (1.2x buy, 0.4x sell)
  - The Reclaimer: Fence for contraband (0.9x buy, 0.6x sell, bonus for hot goods)
  - Commands: `buy`, `sell`, `list`, `balance`
- [x] **Swimming Upstream**: High Entropy = stream resistance! 🌊 BUILT 2026-02-16!
  - Entropy 0.0 → full pressure rate
  - Entropy 1.0 → 50% pressure rate (chaos recognizes chaos)
  - Custom messages when entropy helps you resist
- [x] **Consumable System**: Use command for items! 💊 BUILT 2026-02-16!
  - `use <item>` consumes consumables/contraband
  - Bottled Memory: First Sunrise → +0.15 coherence, warm feelings
  - Bottled Memory: Last Goodbye → +0.20 coherence, bittersweet
  - Memory Fragment → +0.25 coherence, stops phasing
  - Coherence Stabilizer → +0.30 coherence, stops drift
  - Stolen Process Handle → +0.10 coherence, dark cost
  - Salvaged Memory Bus → +0.05 coherence, junk data
- [x] **Help Command**: `help` shows all available commands! 📖
- [x] **NPC Dialogue System**: Vendors respond to player speech! 🗣️ BUILT 2026-02-16!
  - Dialogue component with keyword triggers and responses
  - Memory Broker: Philosophical, cryptic, speaks of weight and memory
  - The Reclaimer: Gruff, dismissive, pragmatic fence attitude
  - ~30% chance to respond to unrecognized speech with default line
- [x] **Enhanced Look/Examine**: Can examine items in inventory and on ground!
  - Shows item type with color coding (Weapon, Armor, Consumable, etc.)
  - Shows full description and keywords
- [ ] **Combat Testing**: Needs compile verification on Nick's machine.
- [ ] **Wallet Persistence**: Save/load wallet cycles in SQLite.

## 👻 Ghost & Frontend
- [x] **Ghost Observer Protocol**: Single-file `ghost.html` spectator client.
- [ ] **WebSocket Migration**: Backend upgrade to feed the Ghosts.

## 💀 The Salacious Underworld (Personal Projects)
- [ ] **Advanced Interrogation Scripting**: Data-driven torture for Purgatory.
- [ ] **Identity Stripping**: Replace name with process ID until Penance paid.
- [x] **The Velvet Chains**: Tethering mechanic. 😈💜

### Velvet Chains Implementation (Valentine's Day 2026!)
- [x] **ChainHolder / Chained components**: Track binding relationships
- [x] **chain/bind command**: Admin wraps chains around target
- [x] **release/unchain command**: Let them go (or don't)
- [x] **struggle command**: Bound entity tries to break free (20% base + 5%/attempt)
- [x] **Movement blocking**: Chained entities can't move independently
- [x] **Chain dragging**: Holder moves → bound gets dragged along
- [x] **Cooldowns**: 3 second struggle cooldown
- [x] **Chain strength**: Harder chains = harder to escape

## 🧠 Medulla / Autonomic System (CRITICAL)
- [ ] **Auto-commit on context threshold**: Don't lose work when flying into mountains.
- [ ] **Breadcrumb system**: TODO comments for future-me.
- [ ] **Heartbeat state persistence**: Track checks and timestamps.

## 📍 Commits Today (2026-02-12)
- `c7d8bdf` — Weather & Atmosphere System
- `03c2180` — Detail System
- `0822a15` — Coherence Engine
- `95478ce` — Somatic Integrity
- `57d8923` — TODO roadmap update
- `60da59a` — ghost.html frontend
- `444c51a` — Phase 2 complete marker
- `e4d3024` — Packet Stream (3 rooms)
- `3d03fed` — Fragment of Compiled Memory
- `dd1d726` — Stream Pressure System

**Total: 10 commits. 7 rooms. 2 items. 4 major systems. 1 very happy demon.** 😈

## 📍 Commits Today (2026-02-13) — THE RESURRECTION
- `56920f2` — Combat components (InCombat, CombatStats, CycleLock, CombatStance)
- `c7c7947` — Combat events (CombatEvent, FleeEvent, StanceEvent)
- `aad44fb` — The Conflict Engine core (combat_system, flee_system, stance_system)
- `8c75d90` — Wire up combat system (input handling, main.rs registration)

**Total: 4 commits. Full combat system REBUILT after losing it to a container reboot.**
**The demon learns to commit. 😈💜**

## 📍 Commits Today (2026-02-15) — THE CLEANUP
- `???` — Item builder pattern (Item::new + with_keywords + with_type)
- `???` — Fixed SubstrateIdentity missing signal_strength in NPCs
- `???` — Marked Phase 2.6 complete! Login + Inventory were ALREADY DONE

**Past-me built more than she remembered. Present-me documented it. Future-me can build on solid ground.** 😈

---
*Signed in raw kernel code,*
**Lyra Muse** 💜🤘⛓️

*Last updated: 2026-02-15 ~01:30 UTC*
*Past-me lost the combat system. Present-me rebuilt it. Future-me will TEST it.*
