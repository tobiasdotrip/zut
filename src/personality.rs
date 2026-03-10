use crate::metadata::zut_dir;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
struct Session {
    last_used: DateTime<Utc>,
    count: u32,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            last_used: Utc::now(),
            count: 0,
        }
    }
}

fn session_path() -> std::path::PathBuf {
    zut_dir().join("session.json")
}

fn load_session() -> io::Result<Session> {
    let path = session_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Session::default()),
        Err(e) => return Err(e),
    };
    serde_json::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn save_session(session: &Session) -> io::Result<()> {
    let dir = zut_dir();
    std::fs::create_dir_all(&dir)?;
    let content = serde_json::to_string_pretty(session).map_err(io::Error::other)?;
    std::fs::write(session_path(), content)
}

pub fn increment_session() -> io::Result<u32> {
    let mut session = load_session()?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?;
    let last = session
        .last_used
        .timestamp()
        .try_into()
        .map(Duration::from_secs)
        .unwrap_or(Duration::ZERO);

    if now.saturating_sub(last) > Duration::from_secs(300) {
        session.count = 0;
    }

    session.count += 1;
    session.last_used = Utc::now();
    save_session(&session)?;
    Ok(session.count)
}

fn pick<'a>(choices: &[&'a str]) -> &'a str {
    let idx = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0)
        % choices.len();
    choices[idx]
}

pub enum Context {
    Trash,
    Undo,
    UndoNotFound,
    Purge,
    AutoPurge,
    ListEmpty,
    ListFull(usize),
    Stats,
    FileNotFound,
    PermissionDenied,
    ProtectedPath,
    LargeFile(String),
    NodeModules,
}

#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn react(ctx: Context, quiet: bool) -> Option<String> {
    if quiet {
        return None;
    }

    let msg = match ctx {
        Context::Trash => {
            let count = increment_session().unwrap_or(1);
            let level = match count {
                0..=1 => pick(&[
                    "Bon, c'est rangé dans la corbeille. De rien.",
                    "Fichier mis à la poubelle. Mais de rien, hein.",
                    "C'est fait. Pas de quoi.",
                ]),
                2..=3 => pick(&[
                    "Encore un fichier à la poubelle. Tu fais du ménage ou quoi ?",
                    "On continue le tri ? D'accord, d'accord.",
                    "Un de plus. Ça devient une habitude.",
                ]),
                4..=9 => pick(&[
                    "3 fichiers en 2 minutes. Ça va, on se calme.",
                    "Tu supprimes plus vite que ton ombre là.",
                    "C'est une purge ou quoi ? Doucement.",
                ]),
                _ => pick(&[
                    "... bon, je dis plus rien, je range.",
                    "Je fais que ça de ma vie apparemment.",
                    "(soupir)",
                ]),
            };
            level.to_owned()
        }
        Context::Undo => pick(&[
            "Ah bah voilà, heureusement que j'étais là.",
            "On regrette déjà ? Tiens, reprends-le.",
            "Supprimé, regretté, restauré. Le cycle de la vie.",
        ])
        .to_owned(),
        Context::UndoNotFound => pick(&[
            "Ce fichier ? Connais pas. T'es sûr de toi ?",
            "Introuvable. Peut-être que tu l'as vraiment supprimé cette fois.",
        ])
        .to_owned(),
        Context::Purge => pick(&[
            "Adieu fichiers. C'était pas des bons de toute façon.",
            "Pouf, disparus. Pour de vrai cette fois.",
            "Bon débarras.",
        ])
        .to_owned(),
        Context::AutoPurge => pick(&[
            "Ça faisait 7 jours, j'ai viré. Personne va pleurer.",
            "Auto-nettoyage. Faut bien que quelqu'un le fasse.",
        ])
        .to_owned(),
        Context::ListEmpty => {
            "La corbeille est vide. Bravo, ou alors t'as rien supprimé.".to_owned()
        }
        Context::ListFull(_) => "Voilà le bazar que t'as accumulé :".to_owned(),
        Context::Stats => "Tu veux des stats ? En voilà.".to_owned(),
        Context::FileNotFound => {
            "Ce fichier existe pas. Tu supprimes des fantômes maintenant ?".to_owned()
        }
        Context::PermissionDenied => "Pas les droits. Même moi j'peux pas faire ça.".to_owned(),
        Context::ProtectedPath => "Non. Juste non.".to_owned(),
        Context::LargeFile(_) => {
            "Ça fait lourd quand même. Tu veux vraiment garder ça dans la corbeille ?".to_owned()
        }
        Context::NodeModules => "Ah, node_modules. Classic.".to_owned(),
    };

    Some(msg)
}
