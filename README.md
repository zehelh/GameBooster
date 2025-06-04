# GameBooster 🚀

## Description
GameBooster est un utilitaire d'optimisation PC pour Windows spécialement conçu pour améliorer les performances de gaming. L'application offre une interface graphique moderne et intuitive pour surveiller, nettoyer et optimiser votre système en temps réel.

## Fonctionnalités
- 📊 Monitoring en temps réel de la RAM
- 📈 Visualisation graphique de l'utilisation mémoire
- 🔄 Nettoyage automatique de la mémoire
- ⚙️ Interface utilisateur moderne avec egui

## Roadmap
- **Nettoyage de disque avancé:**
    - Suppression des fichiers temporaires (système, utilisateur, navigateurs).
    - Vidage des caches (navigateurs, applications).
    - Suppression des miniatures Windows.
- **Scheduler avancé:**
    - Planification du nettoyage de RAM.
    - Planification du nettoyage de disque.
    - Options de planification : au démarrage de la session, toutes les X heures, etc.
- **Optimisation des services Windows:**
    - Désactivation (temporaire/permanente avec avertissements) de Windows Defender.
    - Optimisation d'autres services pour le gaming (avec prudence).
- **Network Limiter:**
    - Lister les processus non-Windows et leur utilisation réseau.
    - Permettre de limiter ou couper le débit réseau pour des processus spécifiques.
- **Création d'un installeur.**
- Portage Linux/MAC OS (à plus long terme).

## Prérequis
L'application est développée en Rust. Assurez-vous d'avoir Rust et Cargo installés.

## Installation
1. Clonez le repository :
```bash
git clone https://github.com/votre-username/GameBooster.git
cd GameBooster
```
2. Compilez le projet :
```bash
cargo build --release
```

## Utilisation
Lancez l'application depuis le dossier `target/release` :
```bash
./gamebooster.exe 
```
L'application nécessite des droits administrateur pour certaines fonctionnalités de nettoyage.

## Contribution
Les contributions sont les bienvenues ! N'hésitez pas à ouvrir une issue ou une pull request.

## Licence
Ce projet est sous licence MIT. Voir le fichier `LICENSE` pour plus de détails.

## Changelog
Consultez le fichier `CHANGELOG.md` pour suivre l'évolution du projet.