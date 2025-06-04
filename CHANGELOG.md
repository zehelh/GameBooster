# Changelog

Toutes les modifications notables de ce projet seront documentées dans ce fichier.

Le format est basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/),
et ce projet adhère au [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-03-19

### Ajouté
- Interface graphique initiale avec eframe/egui
- Monitoring de la RAM système
- Support des cartes graphiques NVIDIA via NVML
- Support des cartes graphiques AMD/Intel via WMI
- Système de logging avec env_logger
- Documentation initiale (README, CHANGELOG)
- Gestion des dépendances avec Cargo
- Configuration du projet et structure de base

### Technique
- Mise en place de l'architecture du projet
- Intégration des bibliothèques principales
- Configuration du système de build
- Gestion des erreurs avec anyhow

## [0.2.0] - 2023-12-20

### Ajouté
- Support amélioré pour les cartes graphiques AMD
- Visualisation graphique de l'utilisation de la mémoire VRAM
- Option de planification des nettoyages

### Amélioré
- Performances générales du programme
- Précision de la détection des GPU AMD

## [0.3.0] - 2024-MM-DD (En développement)

### Ajouté
- Amélioration significative de la fonction `get_amd_gpu_memory`
  - Utilisation des compteurs de performance pour une mesure plus précise
  - Support de plusieurs méthodes de récupération des données selon les disponibilités
  - Meilleure compatibilité avec les systèmes Linux

### En cours
- Optimisation de la fonction `get_amd_gpu_temperature`
- Amélioration de la documentation
- Tests supplémentaires pour différentes configurations matérielles