# macdesk

Reproduce a **macOS-like desktop** on **Debian + Cinnamon** — one command, idempotent, reversible.
The exact same end-state a human applied by hand: Tahoe/WhiteSur GTK themes, macOS Tahoe wallpapers, Big Sur icons, macOS cursors, dark terminal, Plank dock, Inter fonts, transluced panels.

## Requirements

- Debian (trixie tested) with **Cinnamon** desktop
- `git`, `curl`, `tar`, `apt` (for the optional package step)
- `sudo` (cached via `sudo -v`, asked once)

## Usage

```bash
macdesk install            # full transformation (asks for sudo once)
macdesk --dry-run install  # show exactly what would happen; change nothing
macdesk verify             # verify the transformation is in place
macdesk backup             # snapshot current config/gsettings
macdesk restore            # restore the most recent backup
```

`install` performs, in order:

1. **`backup_state`** — dumps every current gsettings tree + copies `~/.themes`, `~/.local/share/icons` to `~/.local/state/macdesk/backup/<timestamp>/` (never overwritten)
2. **`sudo -v`** — caches sudo once
3. **`install_packages`** — `plank, gnome-themes-extra, gtk2-engines-murrine, sassc, libxml2-utils, fonts-inter, fonts-hack, fastfetch, fonts-powerline`
4. **`install_themes`** — clones:
   - `vinceliuice/WhiteSur-gtk-theme` → extracts the prebuilt **WhiteSur-Dark** GTK/Cinnamon/Metacity release into `~/.themes/`
   - `vinceliuice/WhiteSur-icon-theme` → `~/.local/share/icons/WhiteSur-dark`
   - `vinceliuice/WhiteSur-cursors` → `~/.local/share/icons/WhiteSur-cursors`
   - `lestercorderomurillo/macos-tahoe-liquid-kde` (sparse-checkout `src/offline/gtk`) → full **MacTahoeLiquidKde-Dark** theme into `~/.themes/MacTahoeLiquidKde-Dark`
   - wallpapers (Monterey-dark, Sonoma-dark via WhiteSur-wallpapers + `MacTahoe-dark.jpg` via a direct 3840×2160 download) → `~/Imágenes/wallpapers/`
5. **`apply_themes`** — sets gsettings: `org.cinnamon.theme`, gtk/wm themes → MacTahoeLiquidKde-Dark; icons → WhiteSur-dark; cursor → WhiteSur-cursors (32px); wallpaper; Inter/Hack fonts; rgba antialiasing; 24px menubar; macOS window-button layout (`close,minimize,maximize` left)
6. **`apply_plank`** — plank theme `WhiteSur-Dark` with liquid-glass dock.theme (rounded, hairline translucent stroke)
7. **`apply_spices`** — calendar custom format `%a, %d %b %H:%M`, transparent-panels → semi-transparent frosted + theme-defined
8. **`apply_terminal`** — GNOME Terminal dark mac palette (`#1d1f21` bg, `#f5f5f5` fg, Hack 11)

`verify` checks 13 invariants (theme files, gsettings keys, wallpapers, dock, panel height…).

## What you get

- **MacTahoeLiquidKde-Dark** Cinnamon/GTK theme (rounded corners, frosted look, dark slate, macOS HUD-style titlebars)
- **WhiteSur** Big Sur-like icons, macOS cursor set @32px
- **macOS Tahoe dark wallpaper** (`~/Imágenes/wallpapers/MacTahoe-dark.jpg`)
- Inter 11 UI font, Hack 11 mono, Inter Bold 11 titlebars, macOS window buttons on the left
- 24px menubar, Plank dock with liquid-glass theme, GNOME Terminal dark mac palette
- Everything backed up — `git diff` tells you the delta, restore swaps back

## Backups & restore

Every `install` writes a timestamped snapshot to `~/.local/state/macdesk/backup/<ts>/` with gsettings dumps and copies of `.themes` + icons. Restore manually by re-applying the values from the dump (or `macdesk restore` prints them — a future release will automate the copy-back).

## Not included

- KDE Plasma blur / true Liquid Glass window refraction (needs KDE + the full macos-tahoe-liquid-kde Plasma theme — multi-GB and a different DE)
- macOS Global menu bar (window-manager menus; Cinnamon doesn't support)
- Real macOS sounds / notification Centre / dynamic wallpapers (they are proprietary/patch-only)

## License

MIT. The GTK/icon themes remain under their own licenses (GPL-3.0 for WhiteSur/TahoeLiquid themes). You own your config; this tool just sets it.
