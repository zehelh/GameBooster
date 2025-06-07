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
- **CPU Limiter:**
    - Changer les priorité CPU des processus et leurs imposer des limitation.
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

## Évolution de la Stratégie de Blocage Réseau

La méthode initiale de blocage réseau, qui reposait sur la librairie `ndisapi`, s'est avérée peu fiable et ne produisait pas les résultats escomptés. Après une nouvelle phase de recherche et de test, une approche différente a été décidée.

### Analyse des Alternatives

Plusieurs technologies ont été évaluées pour remplacer l'approche existante. Voici un tableau comparatif :

| Alternative | Avantages | Inconvénients | Pertinence pour le projet |
| :--- | :--- | :--- | :--- |
| **`ndisapi`** | Permet un filtrage au niveau des paquets. | L'implémentation s'est avérée complexe et non fonctionnelle, difficile à déboguer. La librairie est moins commune que d'autres alternatives. | **Faible**. Solution actuelle, à remplacer. |
| **Windows Firewall API (`winfw-rs`)** | API native, plus propre que de lancer un processus externe. Crée des règles de pare-feu standards. | Les règles sont persistantes, la gestion par PID est complexe, pas idéal pour un contrôle dynamique et temporaire. | **Moyenne**. Une amélioration, mais ne résout pas les problèmes de fond. |
| **Windows Filtering Platform (WFP)** | API bas niveau la plus puissante de Windows. Contrôle total du trafic, pas de pilote tiers nécessaire. | Extrêmement complexe, courbe d'apprentissage très élevée, peu de wrappers Rust de haut niveau. | **Élevée (en théorie)**. Trop complexe pour une intégration rapide et maintenable dans ce projet. |
| **WinDivert (`windivert-rust`)** | API simple et puissante, conçue pour l'interception/modification de paquets en user-mode. Contrôle par processus, filtrage avancé. | **Nécessite un pilote tiers** (fourni et chargé par la librairie, mais doit être distribué avec l'application). | **Très Élevée**. Le meilleur compromis entre puissance et simplicité pour nos besoins. |

### Décision

L'alternative retenue est **WinDivert**, via la caisse `windivert-rust`. Bien qu'elle introduise une dépendance à un pilote, sa simplicité d'utilisation et sa puissance de filtrage en font la solution la plus adaptée pour implémenter un blocage réseau par processus qui soit fiable, dynamique et efficace.

La nouvelle feuille de route pour la fonctionnalité réseau est la suivante :
1.  **Nettoyer** l'ancienne implémentation basée sur `ndisapi`.
2.  **Intégrer** la caisse `windivert` dans le projet.
3.  **Remplacer** la logique de blocage pour utiliser WinDivert, en capturant les paquets sortants et en les bloquant si le PID correspond à un processus ciblé.
4.  **Distribuer** les fichiers `WinDivert.dll` et `WinDivert64.sys` avec l'application.