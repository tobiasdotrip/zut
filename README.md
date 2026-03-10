<div align="center">

<img src="zut-logo.png" alt="zut" width="200">

# zut

> Un `rm` qui fait pas de bêtises.

![CI](https://github.com/tobiasdotrip/zut/actions/workflows/ci.yml/badge.svg) ![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg) ![Made in France](https://img.shields.io/badge/Made%20in-France-blue)

</div>

---

## C'est quoi ce truc

`zut` remplace `rm` en déplaçant tes fichiers vers `~/.zut/trash/` au lieu de les supprimer pour de vrai. Parce que t'as déjà supprimé `prod.env` par erreur, et c'était pas drôle.

Livrée avec une personnalité passive-agressive incluse dans le binaire. Fait partie de la même famille que [putain](https://github.com/tobiasdotrip/putain) — des outils qui ont l'air inutiles jusqu'au jour où ils te sauvent la mise.

---

## Ce que ça donne

```
$ zut old-api.rs
→ /home/user/projets/old-api.rs
Fichier mis à la poubelle. Mais de rien, hein.

$ zut debug.log tmp.txt
→ /home/user/debug.log
→ /home/user/tmp.txt
Encore un fichier à la poubelle. Tu fais du ménage ou quoi ?

$ zut --undo
← /home/user/tmp.txt
Ah bah voilà, heureusement que j'étais là.

$ zut --list
+----------+---------------------------+------------------+--------+
| ID       | Chemin original           | Supprimé le      | Taille |
+----------+---------------------------+------------------+--------+
| a1b2c3d4 | ~/projets/old-api.rs      | 10/03/2026 14:32 | 4.1 Ko |
| e5f6a7b8 | ~/debug.log               | 10/03/2026 14:30 | 12 Ko  |
+----------+---------------------------+------------------+--------+
2 fichiers, 16.1 Ko au total

$ zut --purge -f
2 fichiers supprimés (16.1 Ko)
Bon débarras.
```

---

## Installation

```bash
cargo install --git https://github.com/tobiasdotrip/zut
```

Optionnel — remplacer `rm` une bonne fois pour toutes :

```bash
# ~/.zshrc ou ~/.bashrc
alias rm='zut'

# Pour le vrai rm quand t'as vraiment besoin :
\rm fichier.txt
```

---

## Usage

| Commande | Description |
|---|---|
| `zut fichier.txt` | Déplace vers la corbeille |
| `zut -r dossier/` | Idem (le `-r` est accepté par compatibilité) |
| `zut -f missing.txt` | Mode force, pas d'erreur si absent |
| `zut --undo` | Restaure le dernier fichier |
| `zut --undo nom.txt` | Restaure par nom |
| `zut --undo a1b2` | Restaure par préfixe d'ID |
| `zut --list` | Affiche le contenu de la corbeille |
| `zut --purge` | Vide la corbeille |
| `zut --purge --older 3d` | Purge les fichiers de plus de 3 jours |
| `zut --stats` | Statistiques de la corbeille |
| `zut --quiet` | Mode silencieux (pour les gens sans humour) |

---

## Personnalité

`zut` a quatre niveaux d'escalade selon à quel point tu l'exaspères.

| Niveau | Déclencheur | Exemple |
|---|---|---|
| **Neutre** | Utilisation normale | `Fichier mis à la poubelle.` |
| **Passif-agressif** | Plusieurs fichiers d'un coup | `Tu fais du ménage ou quoi ?` |
| **Soulagé** | `--undo` après une suppression | `Ah bah voilà, heureusement que j'étais là.` |
| **Définitif** | `--purge` | `Bon débarras.` |

Désactivable avec `--quiet` ou `ZUT_PERSONALITY=false`, si t'es vraiment comme ça.

---

## Configuration

```toml
# ~/.config/zut/config.toml
auto_purge_after = "7d"
sarcasm_level = "normal"    # discret | normal | maximal
personality = true
confirm_purge = true
```

Variables d'environnement disponibles : `ZUT_SARCASM`, `ZUT_PERSONALITY`, `ZUT_TRASH_DIR`.

---

## Du même créateur

[putain](https://github.com/tobiasdotrip/putain) — parce que `cd ..` c'est pas expressif.

---

## License

MIT
