# Quickstart: M0 Setup – Toolchain Proof

How to go from a fresh checkout to pressing Play (FR-013), then check every M0 acceptance item.
Run it on the target machine (Ubuntu 24.04 LTS, Intel UHD 630). Versions come from
[research.md](research.md).

## Prerequisites (once per machine)

1. **Rust**: `rustup` installed. The toolchain pinned in `rust/rust-toolchain.toml` (1.100.0,
   with rustfmt and clippy) is installed automatically the first time `cargo` runs inside `rust/`.
2. **Godot 4.7.2-stable**: already installed as `~/.local/bin/godot`. Check it with:

   ```sh
   godot --version    # expect: 4.7.2.stable.official.ed1daf0bf
   ```

   The Rust library targets the 4.6 API, so it also loads in any Godot from 4.6 up
   ([research.md R1–R2](research.md)). If the installed version changes, update research.md R1.
3. **Session**: the default Ubuntu (Wayland) session is fine; Godot runs through XWayland
   ([research.md R5](research.md)).

## Build and run

```sh
cd rust && cargo build && cd ..                 # creates rust/target/debug/libbeastrider.so
godot --editor --path godot                     # opens the project in the editor
```

In the editor, press **Play** (F5).

## Validation scenarios

Each item points back to the spec. Tick them off at the end of the milestone.

| # | Spec | Do | Expect |
|---|---|---|---|
| 1 | US1, FR-001–006 | Press Play | A 1280×720 window with a dark background, one coloured square drifting left to right, a "Speed: 160 px/s" label and an FPS counter |
| 2 | US1, FR-003 | Watch the square reach the right edge | It leaves fully, then comes in from the left at the same speed |
| 3 | US1, FR-011, SC-004 | Read the Output panel at startup and after 60 s of play | No errors. The startup lines show the Compatibility renderer (OpenGL) on the Intel/Mesa GPU, not `llvmpipe` |
| 4 | US2, SC-002 | Watch the FPS counter for 60 s by the clock | It shows 60 the whole time (the first second is allowed to be lower) |
| 5 | US3, FR-007, SC-003 | Time one full crossing. Stop. Change `speed` in `godot/data/tuning.json` from 160 to 320. **Do not run cargo.** Press Play | The label shows 320, and a crossing takes about half the time (±10 %) |
| 6 | US3 | Set `speed` to `0`, press Play | The square stays still, and the label shows 0 |
| 7 | Edge cases, FR-009 | Set `speed` to `-5`, then `"fast"`, then delete the key, then rename the file. Press Play after each | Each time: one `DriftingSquare:` error naming the file, field and problem; the square stays still; no crash. **Restore the file afterwards.** |
| 8 | FR-004, FR-005, FR-008 | Read `speed_label.gd`, `fps_label.gd` and the Rust source | The labels hold no values of their own. No literal speed appears in Rust or GDScript |
| 9 | FR-010 | Project Settings → Rendering → Renderer | The rendering method is `gl_compatibility`. V-Sync mode is Enabled |
| 10 | US4, SC-005 | `cd rust && cargo fmt --check && cargo build && cargo clippy --all-targets && cargo test` | All pass, with 0 warnings |
| 11 | US4, SC-005 | In the editor, check the Output and Script panels after opening every `.gd` file | 0 GDScript errors or warnings |
| 12 | US4, FR-012, SC-006 | After building and playing: `git status` | Clean. `rust/target/` and `godot/.godot/` do not appear |

## If something goes wrong

- **"Can't open dynamic library" when the editor opens**: step 1 of *Build and run* was skipped,
  or the build failed. Run `cargo build` in `rust/`.
- **FPS below 60, or stuttering**: first check that the Output panel names the Intel/Mesa GPU.
  Then log in with "Ubuntu on Xorg" and repeat scenario 4 to tell XWayland problems apart from
  others ([research.md R5](research.md)).
- **Rust changes not showing**: the library reloads on rebuild (`reloadable = true`). If it
  doesn't, restart the editor.
