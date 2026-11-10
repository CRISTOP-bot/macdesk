//! Verification: checks that each applied piece is where it should be.
use crate::sys::Ctx;

pub struct CheckResult { pub name: &'static str, pub ok: bool, pub detail: String }

fn exists(p: &std::path::Path) -> bool { p.exists() }

pub fn run(ctx: &Ctx) -> Vec<CheckResult> {
    let home = &ctx.home;
    let mut checks = vec![];

    checks.push(CheckResult {
        name: "GTK theme dir",
        ok: exists(&home.join(".themes/MacTahoeLiquidKde-Dark/gtk-3.0/gtk.css")),
        detail: home.join(".themes/MacTahoeLiquidKde-Dark/gtk-3.0/gtk.css").display().to_string(),
    });
    checks.push(CheckResult {
        name: "Cinnamon theme assets",
        ok: exists(&home.join(".themes/MacTahoeLiquidKde-Dark/cinnamon/cinnamon.css")),
        detail: "cinnamon.css present".into(),
    });
    checks.push(CheckResult {
        name: "WhiteSur icons",
        ok: exists(&home.join(".local/share/icons/WhiteSur-dark")),
        detail: home.join(".local/share/icons/WhiteSur-dark").display().to_string(),
    });
    checks.push(CheckResult {
        name: "WhiteSur cursors",
        ok: exists(&home.join(".local/share/icons/WhiteSur-cursors")),
        detail: home.join(".local/share/icons/WhiteSur-cursors").display().to_string(),
    });
    checks.push(CheckResult {
        name: "Monterey/Sonoma wallpaper dir",
        ok: exists(&home.join("Imágenes/wallpapers/MacTahoe-dark.jpg")),
        detail: home.join("Imágenes/wallpapers/MacTahoe-dark.jpg").display().to_string(),
    });
    checks.push(CheckResult {
        name: "Plank theme + config",
        ok: exists(&home.join(".local/share/plank/themes/WhiteSur-Dark/dock.theme")),
        detail: "dock.theme (WhiteSur-Dark)".into(),
    });
    let th = ctx.gsettings_get("org.cinnamon.theme", "name");
    checks.push(CheckResult { name: "cinnamon theme set", ok: th.as_deref() == Some("'MacTahoeLiquidKde-Dark'"), detail: format!("{:?}", th) });
    let gtk = ctx.gsettings_get("org.cinnamon.desktop.interface", "gtk-theme");
    checks.push(CheckResult { name: "gtk theme set", ok: gtk.as_deref() == Some("'MacTahoeLiquidKde-Dark'"), detail: format!("{:?}", gtk) });
    let ic = ctx.gsettings_get("org.cinnamon.desktop.interface", "icon-theme");
    checks.push(CheckResult { name: "icon theme set", ok: ic.as_deref() == Some("'WhiteSur-dark'"), detail: format!("{:?}", ic) });
    let cu = ctx.gsettings_get("org.cinnamon.desktop.interface", "cursor-theme");
    checks.push(CheckResult { name: "cursor theme set", ok: cu.as_deref() == Some("'WhiteSur-cursors'"), detail: format!("{:?}", cu) });
    let font = ctx.gsettings_get("org.gnome.desktop.interface", "font-name");
    checks.push(CheckResult { name: "font Inter", ok: font.as_deref() == Some("'Inter 11'"), detail: format!("{:?}", font) });
    let pan = ctx.gsettings_get("org.cinnamon", "panels-height");
    checks.push(CheckResult { name: "panel 24px", ok: pan.as_deref().map(|s| s.contains("24")).unwrap_or(false), detail: format!("{:?}", pan) });
    let buttons = ctx.gsettings_get("org.cinnamon.desktop.wm.preferences", "button-layout");
    checks.push(CheckResult { name: "window buttons left", ok: buttons.as_deref().map(|s| s.starts_with("'close,")).unwrap_or(false), detail: format!("{:?}", buttons) });
    checks
}
