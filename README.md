# KothanX Arena

**Master Conny Goku Vs Kapten Bobby Kalsong**

This repository is based on the community reverse-engineered Azeroth Survivors project (Rust + Bevy). The Survivors mode now exposes only the two requested class variants, with custom fighter names and themed ability names while retaining the underlying ability mechanics.

## What is changed
- The Survivors class picker contains only **Master Conny Goku** (Rogue base) and **Kapten Bobby Kalsong** (Warrior base).
- The original ability pools, damage values, cooldowns, enemy tables, boss waves, progression and arena systems are retained for those two base classes.
- Ability cards use themed names for the two fighters.

## Build and play locally
This is a Rust/Bevy game, not a standalone static HTML page.

1. Install Rust and the platform build dependencies described in the project scripts.
2. Supply a compatible **World of Warcraft 1.12.1 client Data folder** locally. The client data is not included in this repository and must not be redistributed.
3. From the repository root run:
   ```sh
   make build
   make start
   ```
4. Open `http://127.0.0.1:8090`.

The `make build` target installs the WebAssembly toolchain and builds the browser client. It also runs the project's data-fetch step if the expected local data is missing; review `scripts/fetch-data.sh` before running it.

## Controls
- Move with WASD or arrow keys.
- Use the mobility action with F.
- Escape pauses/resumes.
- See [CONTROLS.md](CONTROLS.md) for the full controls list.

## Multiplayer and hosting status
The upstream Survivors mode is currently a local single-player mode. Two-player couch co-op and online/LAN multiplayer are not implemented yet; they require a larger change to the single-hero run state, targeting, health, XP and level-up systems. The web client also depends on the user's local WoW 1.12.1 data, so it cannot be safely published as a complete public game by simply deploying this repository. A Cloudflare Pages hostname is not configured.

## Assets and licensing
The codebase retains the upstream MIT/Apache license notices. WoW/Blizzard client data, MPQ archives, models, sounds, maps and other proprietary assets are not included. Do not publish or redistribute those assets without permission. This is a fan project and is not affiliated with Blizzard Entertainment.
