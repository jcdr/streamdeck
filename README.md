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
| StreamController | Linux-first GUI controller | Polished UI; overkill for a fixed key map |
| streamdeck-ui / deckmaster | Older controllers | Limited / less maintained |

This project needs a fixed map of shell actions and custom images, so a thin
library client is the right layer.

## Key layout (Original V2, 5×3)

```
Row 1: full shot | window shot | region shot | (empty) | (empty)
Row 2: mute      | unmute      | vol −5%     | vol +5% | (empty)
Row 3: 20%       | 40%         | 60%         | 80%     | 100%
```

| Index | Action | Command / XFCE equivalent |
| --- | --- | --- |
| 0 | Full screenshot | `Print` → `xfce4-screenshooter` |
| 1 | Window screenshot | `Shift+Print` → `/usr/local/bin/screenshoot-window` |
| 2 | Region screenshot | `Ctrl+Print` → `/usr/local/bin/screenshoot-region` |
| 5 | Mute audio | `Ctrl+End` → `wpctl set-mute @DEFAULT_AUDIO_SINK@ 1` |
| 6 | Unmute audio | `Ctrl+Home` → `wpctl set-mute @DEFAULT_AUDIO_SINK@ 0` |
| 7 | Volume down 5% | `Ctrl+Page_Down` → `wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-` |
| 8 | Volume up 5% | `Ctrl+Page_Up` → `wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+` |
| 10 | Volume 20% | `wpctl set-volume @DEFAULT_AUDIO_SINK@ 20%` |
| 11 | Volume 40% | `wpctl set-volume @DEFAULT_AUDIO_SINK@ 40%` |
| 12 | Volume 60% | `wpctl set-volume @DEFAULT_AUDIO_SINK@ 60%` |
| 13 | Volume 80% | `wpctl set-volume @DEFAULT_AUDIO_SINK@ 80%` |
| 14 | Volume 100% | `wpctl set-volume @DEFAULT_AUDIO_SINK@ 100%` |

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
7. **Window/region scripts need a shell.** `/usr/local/bin/screenshoot-*` are
   one-line shell snippets without a shebang. XFCE runs them via a shell;
   direct `exec` fails with `ENOEXEC`. This app runs them as `sh <script>`.
8. **Screenshots need the graphical session.** `wpctl` works without `DISPLAY`;
   `xfce4-screenshooter` does not. If the controller is started outside XFCE
   (service, IDE, agent), it inherits `DISPLAY` / `XAUTHORITY` /
   `DBUS_SESSION_BUS_ADDRESS` from `xfce4-session` via `/proc`.

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
