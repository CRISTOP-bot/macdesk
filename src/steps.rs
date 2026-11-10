//! Install steps — every part of the macOS-on-Cinnamon transformation.
use crate::sys::Ctx;
use std::fs;
use std::process::Command;

pub const WHITE_SUR_DIR: &str = "WhiteSur-gtk-theme";
pub const WHITE_SUR_ICON_DIR: &str = "WhiteSur-icon-theme";
pub const WHITE_SUR_CURSOR_DIR: &str = "WhiteSur-cursors";
pub const WHITE_SUR_WALLPAPERS_DIR: &str = "WhiteSur-wallpapers";
pub const WHITE_SUR_RELEASE_DARK: &str = "WhiteSur-Dark.tar.xz";
pub const TAHOE_GTK_SUBDIR: &str = "src/offline/gtk/MacTahoeLiquidKde-Dark";

pub fn sudo_step(ctx: &Ctx) -> Result<(), String> {
    if ctx.dry_run {
        println!("== sudo: [dry] would refresh the sudo session ==");
        return Ok(());
    }
    println!("== sudo: caching credentials (you may be prompted) ==");
    ctx.sudo_refresh()
}

pub fn install_packages(ctx: &Ctx, skip: bool) -> Result<(), String> {
    if skip { println!("== packages: skipped (--skip-packages) =="); return Ok(()); }
    println!("== packages (apt) ==");
    if !ctx.dry_run && !ctx.sudo_cached() { return Err("sudo session not cached; run sudo -v first or run macdesk as root".into()); }
    ctx.run("apt-get", &["update", "-qq"])?;
    ctx.run("apt-get", &["install", "-y", "-qq",
        "plank", "gnome-themes-extra", "gtk2-engines-murrine",
        "sassc", "libxml2-utils", "fonts-inter", "fonts-hack",
        "fastfetch", "fonts-powerline"])?;
    Ok(())
}

pub fn install_themes(ctx: &Ctx) -> Result<(), String> {
    println!("== themes: downloading + installing ==");
    let tmp = ctx.state_dir.join("tmp");
    fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;

    // WhiteSur GTK (prebuilt dark)
    let ws_gtk = tmp.join(WHITE_SUR_DIR);
    if !ws_gtk.join("release").exists() {
        ctx.git_clone("https://github.com/vinceliuice/WhiteSur-gtk-theme.git", &ws_gtk, true, true)?;
    }
    let tarball = ws_gtk.join("release").join(WHITE_SUR_RELEASE_DARK);
    fs::create_dir_all(ctx.home.join(".themes")).map_err(|e| e.to_string())?;
    ctx.run("tar", &["-xJf", tarball.to_string_lossy().as_ref(), "-C",
        ctx.home.join(".themes").to_string_lossy().as_ref()])?;

    // WhiteSur icon theme
    let ws_icon = tmp.join(WHITE_SUR_ICON_DIR);
    if !ws_icon.join("install.sh").exists() {
        ctx.git_clone("https://github.com/vinceliuice/WhiteSur-icon-theme.git", &ws_icon, true, true)?;
    }
    ctx.run("bash", &[ws_icon.join("install.sh").to_string_lossy().as_ref()])?;

    // WhiteSur cursors
    let ws_cur = tmp.join(WHITE_SUR_CURSOR_DIR);
    if !ws_cur.join("install.sh").exists() {
        ctx.git_clone("https://github.com/vinceliuice/WhiteSur-cursors.git", &ws_cur, true, true)?;
    }
    ctx.run("bash", &[ws_cur.join("install.sh").to_string_lossy().as_ref()])?;

    // Tahoe Liquid GTK/Cinnamon theme
    let tahoe = tmp.join("tahoe-liquid");
    if !tahoe.join("src").exists() {
        ctx.git_clone("https://github.com/lestercorderomurillo/macos-tahoe-liquid-kde.git", &tahoe, true, false)?;
        ctx.run("git", &["sparse-checkout", "set", "src/offline/gtk"]).ok();
    }
    let src = tahoe.join(TAHOE_GTK_SUBDIR);
    let dst = ctx.home.join(".themes").join("MacTahoeLiquidKde-Dark");
    ctx.remove(&dst)?;
    ctx.copy_dir(&src, &dst)?;

    // Wallpapers
    let walls = tmp.join(WHITE_SUR_WALLPAPERS_DIR);
    if !walls.join("4k").exists() {
        ctx.git_clone("https://github.com/vinceliuice/WhiteSur-wallpapers.git", &walls, true, false)?;
    }
    let wall_dir = ctx.home.join("Imágenes/wallpapers");
    for name in ["Monterey-dark.jpg", "Sonoma-dark.jpg"] {
        let src = walls.join("4k").join(name);
        if src.exists() {
            fs::create_dir_all(&wall_dir).map_err(|e| e.to_string())?;
            fs::copy(&src, wall_dir.join(name)).map_err(|e| e.to_string())?;
        }
    }
    // Tahoe wallpaper direct download
    ctx.download(
        "https://raw.githubusercontent.com/lestercorderomurillo/macos-tahoe-liquid-kde/main/src/offline/wallpapers/MacTahoe/contents/images_dark/3840x2160.jpg",
        &wall_dir.join("MacTahoe-dark.jpg"),
    )?;

    Ok(())
}

pub fn apply_themes(ctx: &Ctx) -> Result<(), String> {
    println!("== theme: applying WhiteSur-Dark / Tahoe + icons + cursors ==");
    let t = "MacTahoeLiquidKde-Dark";
    ctx.gsettings_set("org.cinnamon.theme", "name", t)?;
    ctx.gsettings_set("org.cinnamon.desktop.interface", "gtk-theme", t)?;
    ctx.gsettings_set("org.cinnamon.desktop.wm.preferences", "theme", t)?;
    ctx.gsettings_set("org.cinnamon.desktop.interface", "icon-theme", "WhiteSur-dark")?;
    ctx.gsettings_set("org.gnome.desktop.interface", "icon-theme", "WhiteSur-dark")?;
    ctx.gsettings_set("org.cinnamon.desktop.interface", "cursor-theme", "WhiteSur-cursors")?;
    ctx.gsettings_set("org.gnome.desktop.interface", "cursor-theme", "WhiteSur-cursors")?;
    ctx.gsettings_set("org.cinnamon.desktop.interface", "cursor-size", "32")?;
    ctx.gsettings_set("org.gnome.desktop.interface", "cursor-size", "32")?;
    ctx.gsettings_set("org.cinnamon.desktop.wm.preferences", "button-layout", "close,minimize,maximize:")?;
    ctx.gsettings_set("org.gnome.desktop.wm.preferences", "button-layout", "close,minimize,maximize:")?;
    // wallpaper
    let wall = ctx.home.join("Imágenes/wallpapers/MacTahoe-dark.jpg");
    ctx.gsettings_set("org.cinnamon.desktop.background", "picture-uri",
        &format!("file://{}", wall.display()))?;
    // fonts
    ctx.gsettings_set("org.gnome.desktop.interface", "font-name", "Inter 11")?;
    ctx.gsettings_set("org.gnome.desktop.interface", "document-font-name", "Inter 11")?;
    ctx.gsettings_set("org.gnome.desktop.interface", "monospace-font-name", "Hack 11")?;
    ctx.gsettings_set("org.cinnamon.desktop.interface", "font-name", "Inter 11")?;
    ctx.gsettings_set("org.cinnamon.desktop.wm.preferences", "titlebar-font", "Inter Bold 11")?;
    ctx.gsettings_set("org.gnome.desktop.wm.preferences", "titlebar-font", "Inter Bold 11")?;
    ctx.gsettings_set("org.gnome.desktop.interface", "font-antialiasing", "rgba")?;
    ctx.gsettings_set("org.cinnamon", "panels-height", "['1:24']")?;
    Ok(())
}

pub fn apply_plank(ctx: &Ctx) -> Result<(), String> {
    println!("== plank: dock theme + glass liquid ==");
    let src = ctx.home.join(".local/share/plank/themes/WhiteSur-Dark");
    let dst = src.clone();
    if !dst.exists() {
        ctx.run("echo", &["  (missing WhiteSur plank theme; skipping dock.theme)"]).ok();
        return Ok(());
    }
    let dock = dst.join("dock.theme");
    let content = fs::read_to_string(&dock).unwrap_or_default();
    let patched = content
        .replace("FillStartColor=25;;25;;25;;175", "FillStartColor=28;;28;;33;;125")
        .replace("FillEndColor=25;;25;;25;;200",   "FillEndColor=20;;20;;25;;165")
        .replace("OuterStrokeColor=0;;0;;0;;255",  "OuterStrokeColor=255;;255;;255;;32")
        .replace("TopRoundness=23",               "TopRoundness=26")
        .replace("BottomRoundness=23",            "BottomRoundness=26")
        .replace("LineWidth=0",                   "LineWidth=1");
    if !ctx.dry_run { fs::write(&dock, patched).map_err(|e| e.to_string())?; }
    ctx.dconf_write("/net/launchpad/plank/docks/dock1/icon-size", "48")?;
    ctx.dconf_write("/net/launchpad/plank/docks/dock1/theme", "'WhiteSur-Dark'")?;
    Ok(())
}

pub fn apply_spices(ctx: &Ctx) -> Result<(), String> {
    println!("== cinnamon spices (calendar, panel transparency, stark menu apple icon) ==");
    // calendar custom format → mac-style
    let cal = ctx.home.join(".config/cinnamon/spices/calendar@cinnamon.org");
    if let Ok(rd) = fs::read_dir(&cal) {
        for e in rd.flatten() {
            if e.path().extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(s) = fs::read_to_string(e.path()) {
                    if let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&s) {
                        if let Some(cf) = v.get_mut("custom-format") {
                            cf.as_object_mut().map(|o| o.entry("value").or_insert(serde_json::json!("%a, %d %b %H:%M")));
                        }
                        if !ctx.dry_run { let _ = fs::write(e.path(), serde_json::to_string_pretty(&v).unwrap_or_default()); }
                    }
                }
            }
        }
    }
    // transparent-panels semi-transparent
    let tp = ctx.home.join(".config/cinnamon/spices/transparent-panels@germanfr/transparent-panels@germanfr.json");
    if tp.exists() {
        if let Ok(s) = fs::read_to_string(&tp) {
            if let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&s) {
                if let Some(tt) = v.get_mut("transparency-type") {
                    if let Some(o) = tt.as_object_mut() { o.insert("value".into(), serde_json::json!("panel-semi-transparent")); }
                }
                if !ctx.dry_run { let _ = fs::write(&tp, serde_json::to_string_pretty(&v).unwrap_or_default()); }
            }
        }
    }
    Ok(())
}

pub fn apply_terminal(ctx: &Ctx) -> Result<(), String> {
    println!("== gnome-terminal: dark mac palette ==");
    let out = Command::new("gsettings").args(["get", "org.gnome.Terminal.ProfilesList", "default"]).output()
        .map_err(|e| e.to_string())?;
    let profile = String::from_utf8_lossy(&out.stdout).trim().trim_matches('\'').to_string();
    let uri = format!("org.gnome.Terminal.Legacy.Profile:/org/gnome/terminal/legacy/profiles:/:{profile}/");
    for (k, v) in [
        ("background-color", "'#1d1f21'"),
        ("foreground-color", "'#f5f5f5'"),
        ("use-system-font", "false"),
        ("font", "'Hack 11'"),
    ] {
        ctx.run("gsettings", &["set", &uri, k, v])?;
    }
    Ok(())
}

pub fn backup_state(ctx: &Ctx) -> Result<(), String> {
    println!("== backup ==");
    ctx.backup_gsettings("org.cinnamon", "gsettings-cinnamon.txt")?;
    ctx.backup_gsettings("org.cinnamon.desktop.interface", "gsettings-cinnamon-iface.txt")?;
    ctx.backup_gsettings("org.cinnamon.desktop.wm.preferences", "gsettings-wm.txt")?;
    ctx.backup_gsettings("org.gnome.desktop.interface", "gsettings-gnome-iface.txt")?;
    ctx.backup_path(&ctx.home.join(".themes"), "dot-themes")?;
    ctx.backup_path(&ctx.home.join(".local/share/icons"), "icons")?;
    Ok(())
}
