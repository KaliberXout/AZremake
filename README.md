# KothanX Arena

**Master Conny Goku Vs Kapten Bobby Kalsong** — a standalone browser arena-survival game prototype.

## Play
Open [the game](https://kaliberxout.github.io/AZremake/) after GitHub Pages deployment, or open `index.html` locally in a modern desktop browser. No install or account is needed.

## Controls
- Player 1: WASD to move; E for special ability.
- Player 2: Arrow keys to move; Right Shift for special ability.
- Controller: left stick or D-pad to move; A or right shoulder button for special ability. Player 1 uses gamepad 1; Player 2 uses gamepad 2.
- Touch: on-screen directional buttons and special button on narrow screens.
- Escape or Pause button: pause/resume.

## Current gameplay
- Two playable fighters with different stats and special attacks.
- Local two-player arena survival.
- Automatically firing basic attacks.
- Three enemy types, escalating waves and a large boss.
- Experience gems, shared level-ups and six team upgrades.
- Health bars, invulnerability frames, pause and restart.

## Hosting
The repository includes a GitHub Pages workflow. In the repository's **Settings → Pages**, set **Build and deployment → Source** to **GitHub Actions** once. After that, every push to `main` publishes the game at the URL above.

A Cloudflare Pages deployment is also possible by connecting this GitHub repository in Cloudflare Pages and choosing the root directory with no build command. The `kothanarena.pages.dev` hostname can only be assigned if it is available in the Cloudflare account.

## Important scope note
This is an original, self-contained browser implementation inspired by the requested survival-game format. It is not the original Azeroth Survivors game and does not yet reproduce its exact class ability tables, enemy roster, bosses, damage/health formulas, achievements or progression. Online/LAN multiplayer is not implemented; this version is local couch co-op only. No proprietary WoW/Blizzard assets or MPQ data are included.
