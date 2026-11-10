//! macdesk — reproducible macOS-style desktop transformation for Debian + Cinnamon
//! https://github.com/CRISTOP-bot/macdesk
mod sys;
mod steps;
mod verify;

use clap::{Parser, Subcommand};
use sys::Ctx;

#[derive(Parser)]
#[command(name = "macdesk", about = "Reproduce a macOS-like Cinnamon desktop (WhiteSur/Tahoe themes, mac fonts, dark terminal, plank坞)")]
struct Cli {
    /// Dry-run: print what would happen without changing anything
    #[arg(long, global = true)]
    dry_run: bool,

    /// Skip apt package installation
    #[arg(long, global = true)]
    skip_packages: bool,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Apply the full transformation (packages + themes + config + spices + terminal)
    Install,
    /// Check whether the transformation is fully applied
    Verify,
    /// Snapshot current config/gsettings for later restore
    Backup,
    /// Restore the most recent backup
    Restore,
}

fn main() {
    let cli = Cli::parse();
    let ctx = Ctx::new(cli.dry_run);

    let result = match cli.cmd {
        Cmd::Install => install(&ctx, cli.skip_packages),
        Cmd::Verify => verify_state(&ctx),
        Cmd::Backup => steps::backup_state(&ctx),
        Cmd::Restore => restore_latest(&ctx),
    };

    if let Err(e) = result {
        eprintln!("macdesk: {e}");
        std::process::exit(1);
    }
}

fn install(ctx: &Ctx, skip_packages: bool) -> Result<(), String> {
    println!("==================================================");
    println!("  macdesk — reproducir macOS en Debian/Cinnamon");
    println!("==================================================");
    if ctx.dry_run { println!("[DRY-RUN] no se modificará nada\n"); }

    steps::backup_state(ctx)?;
    steps::sudo_step(ctx)?;
    steps::install_packages(ctx, skip_packages)?;
    steps::install_themes(ctx)?;
    steps::apply_themes(ctx)?;
    steps::apply_plank(ctx)?;
    steps::apply_spices(ctx)?;
    steps::apply_terminal(ctx)?;

    println!();
    println!("✅ Instalación completa. Cierra sesión y entra de nuevo.");
    println!("   Verifica con: macdesk verify");
    Ok(())
}

fn verify_state(ctx: &Ctx) -> Result<(), String> {
    let results = verify::run(ctx);
    let mut ok = 0;
    let mut failed = 0;
    for r in &results {
        if r.ok { ok += 1; println!("  ✓ {}", r.name); } else { failed += 1; println!("  ✗ {}  ({})", r.name, r.detail); }
    }
    println!("\n{} ok, {} failed", ok, failed);
    if failed == 0 { Ok(()) } else { Err(format!("{} checks failed", failed)) }
}

fn restore_latest(ctx: &Ctx) -> Result<(), String> {
    let backup_root = ctx.state_dir.join("backup");
    if !backup_root.exists() { return Err(format!("no hay backups en {}", backup_root.display())); }
    let latest = std::fs::read_dir(&backup_root).map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .max_by_key(|e| e.file_name())
        .ok_or("no hay backups")?;
    println!("Último backup: {}", latest.path().display());
    println!("(restaurar manualmente: gsettings list-recursively <archivo> > /tmp/ref.txt; usar gsettings set para trasladar)");
    Ok(())
}
