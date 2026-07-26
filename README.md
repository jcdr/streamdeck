# Stream Deck Starship

Linux controller for the Elgato Stream Deck Original V2 (`0fd9:006d`) with
SpaceX Starship themed key art. Actions mirror the custom XFCE keyboard
shortcuts on this machine.

## Why `elgato-streamdeck` (Rust)

| Option | Role | Fit for this project |
| --- | --- | --- |
| **elgato-streamdeck** (Rust) | Low-level HID library | Best: direct key control, images, no GUI stack, matches preferred language |
| python-elgato-streamdeck | Low-level HID library | Excellent alternative if Python is preferred |
| OpenDeck | Full desktop app + plugin host | Great general Linux daily driver; heavier than needed for fixed keymaps |
| StreamController | Linux-first GUI controller | Polished UI; overkill for a fixed 7-key map |
| streamdeck-ui / deckmaster | Older controllers | Limited / less maintained |

This project needs a fixed map of shell actions and custom images, so a thin
library client is the right layer.

## Key layout (Original V2, 5×3)

| Index | Action | Same as XFCE |
| --- | --- | --- |
| 0 | Mute audio | `Ctrl+End` → `wpctl set-mute @DEFAULT_AUDIO_SINK@ 1` |
| 1 | Unmute audio | `Ctrl+Home` → `wpctl set-mute @DEFAULT_AUDIO_SINK@ 0` |
| 2 | Volume down 5% | `Ctrl+Page_Down` → `wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-` |
| 3 | Volume up 5% | `Ctrl+Page_Up` → `wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+` |
| 5 | Full screenshot | `Print` → `xfce4-screenshooter` |
| 6 | Window screenshot | `Shift+Print` → `/usr/local/bin/screenshoot-window` |
| 7 | Region screenshot | `Ctrl+Print` → `/usr/local/bin/screenshoot-region` |

## Traps avoided

1. **Do not fake key presses** (`xdotool` / `ydotool`). Region capture and focus
   races break easily; call the same commands XFCE uses.
2. **Do not use `pactl` here.** Custom XFCE binds use WirePlumber `wpctl`.
3. **Do not use stock XFCE Print bindings.** This machine overrides them:
   window/region use local scripts that auto-save under `~/Pictures`.
4. **Full screenshot is interactive on purpose.** `Print` runs bare
   `xfce4-screenshooter` (dialog), not `-f`. Matching that avoids surprising
   auto-save behavior.
5. **HID access.** Only one process can own the device. Install the udev rule
   (or keep an ACL on the hidraw node) so the app does not need root.
6. **Always `flush()` after writing key images**, or nothing appears on the deck.

## Build

System package `libhidapi-hidraw0` must be installed. Headers for `hidapi` and
`libudev` can live under `.deps/` (see that directory) when `-dev` packages are
not available system-wide.

```bash
cargo build --release
```

## Run

```bash
cargo run --release
```

Ctrl+C resets the deck and exits.

## Udev (optional, recommended)

```bash
sudo cp udev/70-streamdeck.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Unplug and replug the Stream Deck after installing the rule.
