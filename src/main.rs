use clap::Parser;
use colored::Colorize;
use humansize::{DECIMAL, format_size};
use tabled::{Table, Tabled};
use zut::cli::Cli;
use zut::metadata;
use zut::personality::{self, Context};

#[derive(Tabled)]
struct Row {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Chemin original")]
    path: String,
    #[tabled(rename = "Supprimé le")]
    deleted_at: String,
    #[tabled(rename = "Taille")]
    size: String,
}

fn say(ctx: Context, quiet: bool) {
    if let Some(msg) = personality::react(ctx, quiet) {
        eprintln!("{}", msg.yellow());
    }
}

fn main() {
    let cli = Cli::parse();
    let config = zut::config::Config::load();
    let quiet = cli.quiet || !config.personality;

    if cli.list {
        let mut entries = match metadata::load_entries() {
            Ok(e) => e,
            Err(e) => {
                eprintln!("zut: {e}");
                std::process::exit(1);
            }
        };

        if entries.is_empty() {
            say(Context::ListEmpty, quiet);
            return;
        }

        say(Context::ListFull(entries.len()), quiet);

        entries.sort_by(|a, b| b.deleted_at.cmp(&a.deleted_at));

        let home = dirs::home_dir();

        let rows: Vec<Row> = entries
            .iter()
            .map(|e| {
                let display_path = match &home {
                    Some(h) => e
                        .original_path
                        .strip_prefix(h)
                        .map(|p| format!("~/{}", p.display()))
                        .unwrap_or_else(|_| e.original_path.display().to_string()),
                    None => e.original_path.display().to_string(),
                };

                Row {
                    id: e.id.to_string()[..8].to_owned(),
                    path: display_path,
                    deleted_at: e.deleted_at.format("%d/%m/%Y %H:%M").to_string(),
                    size: format_size(e.size_bytes, DECIMAL),
                }
            })
            .collect();

        let total_size: u64 = entries.iter().map(|e| e.size_bytes).sum();
        let count = entries.len();

        println!("{}", Table::new(&rows));
        println!(
            "{} {}, {} au total",
            count.to_string().bold(),
            if count == 1 { "fichier" } else { "fichiers" },
            format_size(total_size, DECIMAL).bold()
        );
    } else if cli.stats {
        say(Context::Stats, quiet);
        match zut::stats::compute_stats() {
            Ok(stats) => {
                println!("  Fichiers dans la corbeille : {}", stats.files_count);
                println!(
                    "  Taille totale : {}",
                    format_size(stats.total_size, DECIMAL)
                );
                if let Some(ref oldest) = stats.oldest {
                    println!("  Plus vieux fichier : {oldest}");
                }
                if let Some(ref largest) = stats.largest {
                    println!("  Plus gros fichier : {largest}");
                }
                println!("  Fichiers cette semaine : {}", stats.files_this_week);
                println!("  Fichiers restaurés : {}", stats.restored_count);
            }
            Err(e) => {
                eprintln!("zut: {e}");
                std::process::exit(1);
            }
        }
    } else if cli.purge {
        if config.confirm_purge && !cli.force {
            eprint!("Vider la corbeille ? [o/N] ");
            use std::io::BufRead;
            let mut input = String::new();
            std::io::stdin().lock().read_line(&mut input).ok();
            if !matches!(input.trim(), "o" | "O" | "oui" | "y" | "yes") {
                println!("Annulé.");
                return;
            }
        }

        let result = if let Some(ref older) = cli.older {
            let duration = match zut::trash::parse_duration(older) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("zut: {e}");
                    std::process::exit(2);
                }
            };
            zut::trash::purge_older_than(duration)
        } else {
            zut::trash::purge_all()
        };
        match result {
            Ok(stats) if stats.count > 0 => {
                println!(
                    "{} {} supprimés ({})",
                    stats.count,
                    if stats.count == 1 {
                        "fichier"
                    } else {
                        "fichiers"
                    },
                    format_size(stats.total_size, DECIMAL)
                );
                say(Context::Purge, quiet);
            }
            Ok(_) => println!("Rien à purger."),
            Err(e) => {
                eprintln!("zut: {e}");
                std::process::exit(1);
            }
        }
    } else if let Some(ref target) = cli.undo {
        let result = match target {
            Some(name) => zut::trash::undo_by_name(name),
            None => zut::trash::undo_last(),
        };
        match result {
            Ok(entry) => {
                println!("← {}", entry.original_path.display());
                say(Context::Undo, quiet);
                zut::stats::increment_restored();
            }
            Err(e) => {
                say(Context::UndoNotFound, quiet);
                eprintln!("zut: {e}");
                std::process::exit(1);
            }
        }
    } else if !cli.files.is_empty() {
        match zut::trash::trash_files(&cli.files, cli.force, cli.verbose) {
            Ok(entries) => {
                for entry in &entries {
                    println!("→ {}", entry.original_path.display());
                }
                say(Context::Trash, quiet);
                if let Some(stats) = zut::autopurge::run_autopurge(&config.auto_purge_after) {
                    say(Context::AutoPurge, quiet);
                    eprintln!(
                        "  {} vieux {} supprimés",
                        stats.count,
                        if stats.count == 1 {
                            "fichier"
                        } else {
                            "fichiers"
                        }
                    );
                }
            }
            Err(e) => {
                eprintln!("zut: {e}");
                std::process::exit(1);
            }
        }
    } else {
        println!("Rien à faire. Essaie 'zut --help' pour voir les options.");
    }
}
