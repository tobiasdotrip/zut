use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Un rm qui fait pas de bêtises. Déplace vers la corbeille au lieu de supprimer."
)]
pub struct Cli {
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Restaurer (optionnel : nom ou préfixe d'ID)
    #[arg(long, value_name = "NAME", conflicts_with_all = ["list", "purge", "stats"])]
    pub undo: Option<Option<String>>,

    /// Contenu de la corbeille
    #[arg(long, conflicts_with_all = ["undo", "purge", "stats"])]
    pub list: bool,

    /// Vider la corbeille (avec --older : sélectif)
    #[arg(long, conflicts_with_all = ["undo", "list", "stats"])]
    pub purge: bool,

    /// Purge sélective : "30m", "1h", "3d", "2w"
    #[arg(long, value_name = "DURATION", requires = "purge")]
    pub older: Option<String>,

    /// Statistiques de la corbeille
    #[arg(long, conflicts_with_all = ["undo", "list", "purge"])]
    pub stats: bool,

    #[arg(short, long = "recursive", visible_short_alias = 'R', hide = true)]
    pub recursive: bool,

    /// Pas de confirmation, ignore les fichiers manquants
    #[arg(short, long)]
    pub force: bool,

    /// Confirmation avant chaque fichier
    #[arg(short, long, conflicts_with = "force")]
    pub interactive: bool,

    #[arg(short, long, conflicts_with = "quiet")]
    pub verbose: bool,

    /// Désactive les messages de personnalité
    #[arg(long, conflicts_with = "verbose")]
    pub quiet: bool,
}
