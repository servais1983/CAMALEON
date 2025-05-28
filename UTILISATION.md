# Guide d'utilisation - Projet CAMALEON

## Introduction

Ce document fournit un guide d'utilisation complet pour le projet CAMALEON, un outil de cybersécurité adaptatif bio-inspiré conçu pour déstabiliser les attaquants en changeant dynamiquement sa signature et sa surface d'exposition.

## Présentation générale

CAMALEON (Cybernetic Adaptive Morphing Agent for Layered Environment Observation & Neutralization) est une solution de cybersécurité qui s'adapte dynamiquement aux menaces détectées. Inspiré par les capacités de camouflage du caméléon, ce système peut modifier sa posture défensive en fonction du niveau de menace perçu.

## Modules principaux

### 1. Modules fondamentaux

#### chame_core
Module central qui gère les événements, les métriques et l'état du système. Il sert de base à tous les autres modules.

#### skinshift
Module responsable de la modification des bannières et des empreintes OS pour tromper les outils de reconnaissance.

### 2. Modules de détection

#### eye360
Module de détection système qui surveille les syscalls et s'intègre avec eBPF pour une détection avancée.

#### nettongue
Module de détection réseau qui capture les paquets et utilise le fuzzing de latence pour perturber les attaques basées sur le timing.

### 3. Modules de décision et réaction

#### posture_engine
Moteur de décision adaptatif qui analyse les menaces détectées et détermine la posture défensive optimale.

#### lurefield
Système de micro-honeypots adaptatifs qui peuvent être déployés à la volée pour piéger et étudier les attaquants.

### 4. Modules d'interface

#### pigment_api
API REST locale pour le pilotage en temps réel du système.

#### cli
Interface en ligne de commande complète pour contrôler toutes les fonctionnalités.

## Fonctionnalités principales

### 1. Postures défensives

CAMALEON peut adopter différentes postures défensives en fonction du niveau de menace :

- **Silent** : Visibilité minimale, idéale pour l'observation discrète
- **Neutral** : Apparence standard d'un serveur normal
- **Mimetic** : Imite un système vulnérable pour attirer les attaquants
- **Fulgurant** : Perturbe activement les scans et les tentatives de reconnaissance
- **Unstable** : Simule un système défectueux pour décourager les attaques

### 2. Analyse multi-formats

CAMALEON peut analyser différents types de fichiers pour détecter des menaces :

- **Fichiers CSV** : Détection de traces de cyberattaques, ransomware, et comportements suspects
- **Fichiers logs** : Identification de tentatives d'intrusion, d'échecs d'authentification et d'activités malveillantes

### 3. Génération de rapports

Le système génère des rapports HTML détaillés qui incluent :

- Un score global de sécurité
- Une liste des détections classées par sévérité
- Des statistiques par type de menace
- Des recommandations d'actions à entreprendre

## Utilisation de base

### Démarrage du système

```bash
# Démarrage avec configuration par défaut
camaleon start

# Démarrage avec une posture spécifique
camaleon start --mode mimetic
```

### Surveillance du système

```bash
# Afficher le statut actuel
camaleon status

# Consulter les événements récents
camaleon events --last 10
```

### Configuration des modules

#### Configuration de eye360 (détection système)

```bash
# Activer la surveillance des syscalls
camaleon eye360 --track-syn --syscalls execve,fork,clone

# Désactiver la surveillance
camaleon eye360 --disable
```

#### Configuration de nettongue (détection réseau)

```bash
# Activer la capture de paquets et le fuzzing de latence
camaleon nettongue --pcap --latency-fuzz

# Spécifier une interface réseau
camaleon nettongue --pcap --interface eth0
```

#### Déploiement de honeypots

```bash
# Déployer un honeypot SSH
camaleon lurefield --generate ssh --fake-auth --log-keystroke

# Déployer un honeypot HTTP
camaleon lurefield --generate http
```

#### Changement de posture

```bash
# Changer manuellement la posture
camaleon posture --set fulgurant

# Activer la rotation des services
camaleon posture --rotate-services
```

### Analyse de fichiers

```bash
# Analyser un fichier CSV
camaleon analyze --format csv --file /chemin/vers/fichier.csv

# Analyser un fichier de logs
camaleon analyze --format log --file /chemin/vers/fichier.log
```

### Utilisation de l'API

L'API est accessible par défaut sur `http://localhost:8080/api/`.

Exemples d'utilisation avec curl :

```bash
# Obtenir le statut du système
curl http://localhost:8080/api/status

# Obtenir les événements récents
curl http://localhost:8080/api/events

# Changer la posture défensive
curl -X POST http://localhost:8080/api/posture -H "Content-Type: application/json" -d '{"posture":"mimetic"}'
```

## Scénarios d'utilisation

### 1. Détection d'une tentative de reconnaissance

1. CAMALEON détecte des scans de ports via le module nettongue
2. Le posture_engine évalue la menace comme moyenne
3. Le système passe en posture mimetic pour attirer l'attaquant
4. Des honeypots sont déployés automatiquement pour observer les techniques de l'attaquant
5. Un rapport est généré avec les détails de l'attaque

### 2. Analyse d'un fichier CSV suspect

1. L'utilisateur soumet un fichier CSV pour analyse
2. Le module formats détecte des indicateurs de ransomware
3. Le système génère un rapport détaillé avec les lignes suspectes
4. Des recommandations d'actions sont fournies

### 3. Protection contre une attaque en cours

1. CAMALEON détecte une tentative d'intrusion active
2. Le posture_engine évalue la menace comme élevée
3. Le système passe en posture fulgurant pour perturber l'attaque
4. Les services exposés sont modifiés dynamiquement
5. Des alertes sont envoyées via l'API et la CLI

## Bonnes pratiques

1. **Surveillance régulière** : Consultez régulièrement le statut et les événements pour détecter les anomalies
2. **Mise à jour des règles** : Ajoutez de nouveaux patterns de détection pour les menaces émergentes
3. **Tests périodiques** : Effectuez des tests de pénétration contrôlés pour vérifier l'efficacité du système
4. **Sauvegarde des rapports** : Conservez les rapports générés pour analyse historique
5. **Isolation réseau** : Déployez CAMALEON sur un segment réseau dédié pour une meilleure efficacité

## Dépannage

### Problèmes courants et solutions

1. **Le système ne détecte pas certaines activités réseau**
   - Vérifiez que l'interface réseau correcte est configurée
   - Assurez-vous que les privilèges sont suffisants pour la capture de paquets

2. **Les honeypots ne se déploient pas**
   - Vérifiez que les ports nécessaires sont disponibles
   - Assurez-vous que le nombre maximum de honeypots n'est pas atteint

3. **L'API n'est pas accessible**
   - Vérifiez que le service est en cours d'exécution
   - Assurez-vous que le port n'est pas bloqué par un pare-feu

4. **Les rapports ne s'affichent pas correctement**
   - Vérifiez que les templates sont correctement installés
   - Assurez-vous que les permissions sont correctes sur le dossier de sortie

## Conclusion

CAMALEON offre une approche innovante de la cybersécurité en s'adaptant dynamiquement aux menaces détectées. En utilisant efficacement ses différentes postures défensives et ses capacités d'analyse, vous pouvez considérablement renforcer la sécurité de votre infrastructure.

Pour plus d'informations sur les fonctionnalités avancées, consultez la documentation technique complète dans le dossier `docs/` du projet.
