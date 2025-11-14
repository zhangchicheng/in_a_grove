# bevy_game - Minimal Platformer

A small Bevy 0.17 platformer demo with:

- Player movement, gravity, jump and double-jump
- Coyote time (grace period for jumping after leaving a platform)
- Simple AABB collision resolution with static and moving platforms
- Camera smoothing (lerp) and clamp to level bounds
- Console-based HUD that prints controls and jumps left periodically

How to run

- Build and run:

```bash
cargo run
```

Controls

- Left/Right: Arrow keys
- Jump: Space (double jump supported)

Notes & next steps

- The HUD is printed to the console for compatibility across Bevy versions; we can add an on-screen UI if you want (may require enabling UI features).
- Consider using a physics plugin (e.g. Rapier) for more robust collision and slopes.
- I can add level data, tilemap support, animations, and polish next.
