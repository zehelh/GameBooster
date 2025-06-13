# GameBooster 🚀

## Description
GameBooster est un utilitaire d'optimisation PC pour Windows spécialement conçu pour améliorer les performances de gaming. L'application offre une interface graphique moderne et intuitive pour surveiller, nettoyer et optimiser votre système en temps réel.

## Fonctionnalités
- 📊 Monitoring en temps réel de la RAM (Windows & Linux)
- 📈 Visualisation graphique de l'utilisation mémoire (Windows & Linux)
- 🔄 Nettoyage de la mémoire (Windows: `EmptyWorkingSet`, Linux: `drop_caches` si root)
- ⚙️ Interface utilisateur moderne avec egui
- 🐧 Détection de l'OS avec affichage conditionnel des fonctionnalités (ex: onglets spécifiques Windows marqués "WIP" sous Linux)

## Roadmap
- **Nettoyage de disque avancé (Windows & Linux):**
    - Suppression des fichiers temporaires (système, utilisateur, navigateurs).
    - Vidage des caches (navigateurs, applications).
    - Suppression des miniatures Windows.
- **Scheduler avancé (Windows & Linux):**
    - Planification du nettoyage de RAM.
    - Planification du nettoyage de disque.
    - Options de planification : au démarrage de la session, toutes les X heures, etc.
- **Optimisation des services Windows:**
    - Désactivation (temporaire/permanente avec avertissements) de Windows Defender.
    - Optimisation d'autres services pour le gaming (avec prudence).
- **Network Limiter (Windows & Linux):**
    - Lister les processus non-Windows et leur utilisation réseau.
    - Permettre de limiter ou couper le débit réseau pour des processus spécifiques.
- **CPU Limiter (Windows & Linux):**
    - Changer les priorité CPU des processus et leurs imposer des limitation.
- **Création d'un installeur (Windows & Linux).**
- Portage MAC OS (à plus long terme).

## Prérequis
L'application est développée en Rust. Assurez-vous d'avoir Rust et Cargo installés.

### Pour la compilation croisée Windows vers Linux (et potentiellement d'autres cibles) avec Zig:

Il est recommandé d'utiliser Zig comme linker pour faciliter la compilation croisée, notamment pour la glibc.

1.  **Installer Zig:**
    Téléchargez la dernière version de Zig pour votre système depuis [ziglang.org/download/](https://ziglang.org/download/).
    Extrayez l'archive et ajoutez le répertoire de Zig à votre `PATH`.
    *Alternative (non recommandée pour ce projet à cause de problèmes de version/path):* `sudo snap install zig --classic --beta`

2.  **Configurer Cargo pour utiliser Zig:**
    Créez (ou modifiez) le fichier `.cargo/config.toml` dans votre répertoire projet ou global Cargo avec le contenu suivant :

    ```toml
    [target.x86_64-unknown-linux-gnu]
    linker = "zig"
    rustflags = ["-C", "linker-flavor=ld.lld", "-C", "link-arg=-fuse-ld=lld"]

    # Si vous utilisez un wrapper script (voir ci-dessous)
    # linker = "/chemin/vers/votre/zig_cc_wrapper.sh"
    # rustflags = [] # Les flags sont dans le wrapper
    ```

3.  **(Optionnel mais recommandé) Script Wrapper pour Zig:**
    Pour plus de flexibilité et pour gérer les cas où `zig cc` a besoin d'arguments spécifiques ou si `zig` n'est pas directement dans le PATH du processus de build de Cargo, vous pouvez utiliser un script wrapper.
    Créez un fichier `zig_cc_wrapper.sh` (ou un nom similaire) quelque part dans votre système (par exemple, `/usr/local/bin/` ou dans le répertoire de votre projet) :

    ```bash
    #!/bin/bash
    # Wrapper pour utiliser zig comme linker avec Cargo
    # Assurez-vous que ce script est exécutable (chmod +x zig_cc_wrapper.sh)
    
    # Chemin vers votre exécutable zig si non standard
    # ZIG_PATH="/chemin/vers/votre/zig/zig"
    ZIG_PATH="zig" # Si zig est dans le PATH

    # Détecter la cible à partir des arguments
    TARGET=""
    for arg in "$@"; do
        if [[ "$arg" == "--target="* ]]; then
            TARGET="${arg#--target=}"
            break
        fi
    done

    if [ -z "$TARGET" ]; then
        # Essayer de déduire la cible si non fournie explicitement
        # Ceci est une heuristique et pourrait nécessiter des ajustements
        if [[ "$(uname -s)" == "Linux" && "$(uname -m)" == "x86_64" ]]; then
            TARGET="x86_64-linux-gnu"
        elif [[ "$(uname -s)" == "Darwin" ]]; then
            TARGET="$(uname -m)-apple-darwin"
        fi
        # Ajoutez d'autres détections si nécessaire
    fi

    # Exécuter zig cc avec les arguments et les flags nécessaires
    # Pour Linux, spécifier la version de glibc peut être crucial
    if [[ "$TARGET" == *"linux-gnu"* ]]; then
        # Adaptez la version de glibc si nécessaire (ex: 2.17, 2.28, etc.)
        # Utilisez `zig targets` pour voir les options disponibles
        exec $ZIG_PATH cc -target "$TARGET-gnu.2.28" "$@"
    elif [[ "$TARGET" == *"windows-msvc"* ]]; then
        # Pour Windows, zig peut aussi cross-compiler
        exec $ZIG_PATH cc -target "$TARGET" "$@"
    else
        # Pour les autres cibles ou si la cible n'est pas détectée, utiliser le comportement par défaut
        exec $ZIG_PATH cc "$@"
    fi
    ```
    N'oubliez pas de rendre ce script exécutable (`chmod +x zig_cc_wrapper.sh`) et d'ajuster le `linker` dans `.cargo/config.toml` pour pointer vers ce script.

## Installation
1. Clonez le repository :
```bash
git clone https://github.com/votre-username/GameBooster.git
cd GameBooster
```
2. Compilez le projet :

   Pour Windows (natif) :
   ```bash
   cargo build --release
   ```

   Pour Linux (compilation native ou croisée depuis Windows/macOS avec Zig configuré) :
   ```bash
   cargo build --release --target x86_64-unknown-linux-gnu
   ```

## Utilisation
Lancez l'application depuis le dossier `target/release` (pour Windows) ou `target/x86_64-unknown-linux-gnu/release` (pour Linux).

Pour Windows:
```bash
./gamebooster.exe 
```

Pour Linux:
```bash
./gamebooster
```
L'application nécessite des droits administrateur (Windows) ou root (Linux) pour certaines fonctionnalités (nettoyage RAM avancé, gestion des services, etc.).

## Contribution
Les contributions sont les bienvenues ! N'hésitez pas à ouvrir une issue ou une pull request.

## Licence
Ce projet est sous licence MIT. Voir le fichier `LICENSE` pour plus de détails.

## Changelog
Consultez le fichier `CHANGELOG.md` pour suivre l'évolution du projet.
