# KothanX Arena

**Master Conny Goku Vs Kapten Bobby Kalsong**

This repository keeps the Rust/Bevy Azeroth Survivors reconstruction and adds a self-contained static browser arena at `web/arena.html`. The browser arena uses the requested title and two fighters, with no proprietary WoW client files required.

## Play the browser arena

Open `web/arena.html` in a modern desktop browser, or serve the repository with any static HTTP server. The root `index.html` redirects to the browser arena, making it suitable as the entry page for a static-site host such as GitHub Pages once Pages is enabled for the repository.

- **Solo:** play as Master Conny Goku.
- **Local co-op:** both fighters share the keyboard.
- **Online co-op:** create a room and share the six-character code; the other player joins with that code. Online play uses PeerJS's public signaling service and a peer-to-peer WebRTC connection, so both players need an internet connection and the PeerJS CDN must be reachable.
- Survive a 20-minute run, defeat escalating enemy waves and Dread Lords, collect experience, and choose upgrades.

### Browser-game controls

- Player 1 — Master Conny Goku: **WASD** to move, **Space** for Instant Transmission.
- Player 2 — Kapten Bobby Kalsong: **Arrow keys** to move, **Right Ctrl** for Captain's Charge.
- Basic attacks are automatic. Kamehameha and Underpants Uppercut signature abilities trigger automatically during combat.
- **Escape** pauses or resumes a local run.

The browser arena draws its own fantasy-style visuals and does not bundle Blizzard artwork, models, sounds, maps, or client data.

## Rust/Bevy reconstruction

The repository also retains the original Rust/Bevy client path. Its Survivors class picker exposes the two requested variants:
- **Master Conny Goku** — Rogue base.
- **Kapten Bobby Kalsong** — Warrior base.

The Rust client still depends on a locally supplied compatible World of Warcraft 1.12.1 client Data folder and WebGPU for the browser world. It is separate from the self-contained static arena above.

## Build the Rust client locally

1. Install Rust and the platform build dependencies described in the project scripts.
2. Supply a compatible **World of Warcraft 1.12.1 client Data folder** locally. The client data is not included in this repository and must not be redistributed.
3. From the repository root run:
   ```sh
   make build
   make start
   ```
4. Open `http://127.0.0.1:8090`.

Review `scripts/fetch-data.sh` before running the build; it may fetch proprietary client data.

## Assets and licensing

The codebase retains its upstream open-source license notices. WoW/Blizzard client data and proprietary assets are not included. This is an unofficial fan project and is not affiliated with Blizzard Entertainment.
