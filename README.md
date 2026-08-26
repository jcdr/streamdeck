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
Row 1: full shot | window shot | region shot | open Pictures | time / brightness +10%
Row 2: mute      | unmute      | vol −5%     | vol +5%       | date / brightness −10%
Row 3: 20%       | 40%         | 60%         | 80%           | 100%
```

| Index | Action | Command / XFCE equivalent |
| --- | --- | --- |
| 0 | Full screenshot | `Print` → `/usr/local/bin/screenshoot-full` |
| 1 | Window screenshot | `Shift+Print` → `/usr/local/bin/screenshoot-window` |
| 2 | Region screenshot | `Ctrl+Print` → `/usr/local/bin/screenshoot-region` |
| 3 | Open screenshots folder | `thunar ~/Pictures` (XFCE file manager) |
| 4 | Time display + deck brightness +10% | Live local time (`hh:mm` / `ss` / TZ). Press raises Stream Deck backlight by 10% (clamped 0–100). |
| 5 | Mute audio | `Ctrl+End` → `wpctl set-mute @DEFAULT_AUDIO_SINK@ 1` |
| 6 | Unmute audio | `Ctrl+Home` → `wpctl set-mute @DEFAULT_AUDIO_SINK@ 0` |
| 7 | Volume down 5% | `Ctrl+Page_Down` → `wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-` |
| 8 | Volume up 5% | `Ctrl+Page_Up` → `wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+` |
| 9 | Date display + deck brightness −10% | Live local date (`dd` / `mm` / `yyyy`). Press lowers Stream Deck backlight by 10% (clamped 0–100). |

Date and time keys share the same primary and secondary font sizes so they stay visually matched. Default backlight is 70%. The chosen level is restored after display sleep and after an unplug/replug in the same process.
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
4. **HID access.** Only one process can own the device. Install the udev rule
   (or keep an ACL on the hidraw node) so the app does not need root.
5. **Always `flush()` after writing key images**, or nothing appears on the deck.
6. **Screenshot helpers are installed to `/usr/local/bin`.** Sources live in
   `helpers/` with a proper shebang. The controller still launches them via
   `sh` for safety if an older shebang-less copy is present.
7. **Screenshots need the graphical session.** `wpctl` works without `DISPLAY`;
   `xfce4-screenshooter` does not. If the controller is started outside XFCE
   (service, IDE, agent), it inherits `DISPLAY` / `XAUTHORITY` /
   `DBUS_SESSION_BUS_ADDRESS` from `xfce4-session` via `/proc`.
8. **Do not use the Stream Deck's own idle-sleep timeout.** The Original V2 can
   dim after a host-set idle period, but that would blank the deck while the
   monitor is still on. Sleep follows display DPMS / screensaver / lock instead.

## Build

System package `libhidapi-hidraw0` must be installed. Headers for `hidapi` and
`libudev` can live under `.deps/` (see that directory) when `-dev` packages are
not available system-wide.

```bash
cargo build --release
```

## Screenshot helpers

Repo copies of the XFCE screenshot wrappers:

| File | Action |
| --- | --- |
| `helpers/screenshoot-full` | Full screen → `~/Pictures/Screenshot_….png` |
| `helpers/screenshoot-window` | Active window |
| `helpers/screenshoot-region` | Region select |

Install system-wide (uses `sudo` when not root):

```bash
./scripts/install-screenshot-helpers.sh
```

That installs executable scripts into `/usr/local/bin/`.

## Run

```bash
cargo run --release
```

Ctrl+C resets the deck (when present) and exits.

The process **survives unplug/replug**: it polls about every 2 seconds while the
deck is missing, reconnects when it appears, and repaints all keys. HID read
failures while connected re-enter that wait loop (no process exit).

## Display power saving

The Original V2 has no host-independent power switch. This controller follows
the **display** instead of a Stream Deck idle timer:

- When every connected monitor is in DPMS standby/suspend/off, the screensaver
  is active, or the seated graphical session is locked, the deck backlight is
  set to 0% and every key is painted black.
- When the display is on again, brightness (70%) and key art are restored,
  including the live date/time keys.
- A key press while the deck is sleeping tries to wake the display (`xset dpms
  force on` plus screensaver `SimulateUserActivity`) but does **not** run the
  bound action. Unlock still uses the normal lock screen.

The deck is not blanked while the monitor is still on. Native Stream Deck idle
sleep is unused, so the two devices stay in step.

Sources, any of which can put the deck to sleep:

1. DRM connector DPMS in `/sys/class/drm` (connected + enabled outputs)
2. X11 DPMS via `xset q` (`Monitor is On` vs Off/Standby/Suspend)
3. Screensaver D-Bus `GetActive` (`org.xfce.ScreenSaver`, then freedesktop /
   GNOME / KDE names)
4. logind `LockedHint` on seated user sessions

## Systemd user service

Starts with the **user** `default.target` after login (when your user systemd
instance is up). XFCE under LightDM often **never starts**
`graphical-session.target`, so binding the unit only to that target left the
service enabled-but-dead after reboot.

Do **not** enable linger for this unit unless you know you need it (the app
still expects a desktop for screenshots).

The tracked unit is a **template**
(`systemd/streamdeck-starship.service.in`) with `@PROJECT_ROOT@` placeholders.
The install script expands it into `~/.config/systemd/user/` for your clone path
(no hardcoded username).

```bash
./scripts/install-user-service.sh
```

Useful commands:

```bash
systemctl --user status streamdeck-starship.service
journalctl --user -u streamdeck-starship.service -f
systemctl --user restart streamdeck-starship.service
systemctl --user stop streamdeck-starship.service
systemctl --user disable streamdeck-starship.service
```

After code changes: `cargo build --release` then
`systemctl --user restart streamdeck-starship.service`.

Do not run a second manual instance while the service owns the device (HID is
exclusive).

## Udev (optional, recommended)

```bash
sudo cp udev/70-streamdeck.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Unplug and replug the Stream Deck after installing the rule.

## Publishing notes

- The repo is intended to be public; avoid committing absolute home paths.
- Generated systemd units under `~/.config/systemd/user/` stay local.
- Machine-local `libudev.so` linker stubs under `.deps/lib/` are gitignored.
- Git commit author email is public on GitHub. Prefer GitHub’s private noreply
  address for future commits, for example:

  ```bash
  git config user.email "YOUR_ID+USERNAME@users.noreply.github.com"
  ```

  Existing history still contains earlier author emails unless history is
  rewritten (not done by default).
