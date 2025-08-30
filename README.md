# 🎮 GameBooster - Optimiseur PC pour Gamers

**GameBooster** est une application d'optimisation PC complète pour Windows, spécialement conçue pour améliorer les performances de jeu. L'application fournit une interface graphique moderne et intuitive pour surveiller, nettoyer, optimiser et planifier des tâches de maintenance système en temps réel.

## ✨ Fonctionnalités

### 💾 Gestion Mémoire
- **Surveillance temps réel** de l'utilisation mémoire
- **Nettoyage automatique** de la mémoire système
- **Optimisation intelligente** des ressources
- **Top des processus** utilisant le plus de mémoire

### 💿 Nettoyage Disque
- **Fichiers temporaires** - Nettoyage automatique
- **Cache navigateurs** - Chrome, Firefox, Edge
- **Corbeille** - Vidage automatique
- **Miniatures** - Nettoyage des thumbnails
- **Fichiers Windows Update** - Libération d'espace
- **Logs système** - Nettoyage des logs obsolètes

### ⏰ Planificateur de Tâches
- **CRUD complet** des tâches planifiées
- **Fréquences multiples** - Quotidien, hebdomadaire, mensuel
- **Persistance JSON** des configurations
- **Moteur d'exécution** automatique
- **Interface intuitive** de gestion

### ⚙️ Services Système
- **Gestion des services Windows**
- **Optimisation des services critiques**
- **Surveillance des performances**

### 🌐 Gestion Réseau
- **Limitation de bande passante**
- **QoS avancé**
- **Surveillance des connexions**

## 🏗️ Architecture

L'application est construite avec une architecture modulaire en C# :

```
GameBooster/
├── GameBooster.Common/          # Modèles et interfaces partagés
├── GameBooster.Core/            # Services métier
├── GameBooster.Services/        # Services avancés
├── GameBooster.UI/              # Interface utilisateur Avalonia
└── GameBooster.Tests/           # Tests unitaires
```

### Technologies Utilisées

- **.NET 7.0** - Framework de base
- **Avalonia UI** - Interface graphique cross-platform
- **ReactiveUI** - Pattern MVVM réactif
- **Microsoft.Extensions.DependencyInjection** - Injection de dépendances
- **System.Reactive** - Programmation réactive

## 🚀 Installation et Utilisation

### Prérequis
- Windows 10/11
- .NET 7.0 Runtime
- Droits administrateur (recommandé)

### Compilation
```bash
# Restaurer les dépendances
dotnet restore

# Compiler la solution
dotnet build

# Lancer l'application
dotnet run --project GameBooster.UI
```

### Exécution
1. Lancez l'application en tant qu'administrateur
2. L'interface se charge automatiquement
3. Utilisez les onglets pour naviguer entre les fonctionnalités
4. Cliquez sur "⚡ Optimiser" pour un nettoyage complet

## 📊 Interface Utilisateur

L'interface moderne propose :

- **Thème sombre** adapté aux gamers
- **Vue d'ensemble** en temps réel
- **Onglets organisés** par fonctionnalité
- **Indicateurs visuels** de performance
- **Boutons d'action** intuitifs

## 🔧 Configuration

### Fichiers de Configuration
- `scheduled_tasks.json` - Tâches planifiées
- `app.manifest` - Privilèges administrateur
- Logs dans `%APPDATA%\GameBooster\`

### Personnalisation
- Modifiez les fréquences de nettoyage
- Ajoutez des tâches personnalisées
- Configurez les seuils d'alerte

## 🧪 Tests

```bash
# Lancer les tests unitaires
dotnet test

# Tests avec couverture
dotnet test --collect:"XPlat Code Coverage"
```

## 📈 Performance

L'application est optimisée pour :
- **Démarrage rapide** (< 2 secondes)
- **Utilisation mémoire minimale** (< 50 MB)
- **Surveillance en arrière-plan** efficace
- **Interface fluide** et réactive

## 🔒 Sécurité

- **Privilèges administrateur** requis
- **Validation des données** d'entrée
- **Gestion d'erreurs** robuste
- **Logs détaillés** pour le débogage

## 🤝 Contribution

1. Fork le projet
2. Créez une branche feature (`git checkout -b feature/AmazingFeature`)
3. Committez vos changements (`git commit -m 'Add AmazingFeature'`)
4. Push vers la branche (`git push origin feature/AmazingFeature`)
5. Ouvrez une Pull Request

## 📝 Licence

Ce projet est sous licence MIT. Voir le fichier `LICENSE` pour plus de détails.

## 🆘 Support

- **Issues GitHub** - Signaler les bugs
- **Discussions** - Questions et suggestions
- **Documentation** - Guide d'utilisation détaillé

## 🔄 Roadmap

- [ ] Support des services réseau avancés
- [ ] Intégration avec les jeux populaires
- [ ] API REST pour l'automatisation
- [ ] Plugin système pour extensions
- [ ] Support multi-langues

---

**GameBooster** - Optimisez votre PC pour des performances de jeu maximales ! 🎮⚡
