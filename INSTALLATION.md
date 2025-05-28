# Guide d'installation et de déploiement - Projet CAMALEON

## Introduction

Ce document fournit les instructions détaillées pour installer, configurer et déployer le projet CAMALEON, un outil de cybersécurité adaptatif bio-inspiré.

## Prérequis

- Système d'exploitation Linux (Ubuntu 20.04+ recommandé)
- Rust 1.70+ et Cargo
- libpcap-dev (pour la capture de paquets)
- Privilèges root pour certaines fonctionnalités (eBPF, capture réseau)

## Installation

### 1. Installation des dépendances

```bash
# Mise à jour du système
sudo apt update && sudo apt upgrade -y

# Installation des dépendances requises
sudo apt install -y build-essential libpcap-dev curl git

# Installation de Rust (si non installé)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Clonage du dépôt

```bash
git clone https://github.com/servais1983/CAMALEON.git
cd CAMALEON
```

### 3. Compilation du projet

```bash
# Compilation en mode debug
cargo build

# Compilation en mode release (recommandé pour la production)
cargo build --release
```

### 4. Installation des binaires

```bash
# Installation locale
cargo install --path .

# Ou copie manuelle des binaires
sudo cp target/release/camaleon /usr/local/bin/
```

## Configuration

### 1. Fichier de configuration

Créez un fichier de configuration dans `/etc/camaleon/config.toml` :

```bash
sudo mkdir -p /etc/camaleon
sudo nano /etc/camaleon/config.toml
```

Contenu de base pour le fichier de configuration :

```toml
[general]
log_level = "info"
adaptive_mode = true
default_posture = "neutral"

[skinshift]
enabled = true
presets_dir = "/etc/camaleon/presets"
rotation_interval = 3600  # secondes, 0 = désactivé

[eye360]
enabled = true
syscall_monitoring = true
log_suspicious = true
ebpf_enabled = false  # Nécessite des privilèges root

[nettongue]
enabled = true
pcap_enabled = true
interface = "eth0"  # Interface par défaut, à remplacer dans la config personnalisée
latency_fuzz_enabled = false
latency_fuzz_min_ms = 50
latency_fuzz_max_ms = 200

[lurefield]
enabled = true
honeypot_dir = "/etc/camaleon/honeypots"
max_honeypots = 5
auto_deploy = false

[posture]
change_threshold = 0.75  # Niveau de confiance pour déclencher un changement de posture
service_rotation_enabled = false
service_rotation_interval = 7200  # secondes
postures = [
    "silent",
    "neutral",
    "mimetic",
    "fulgurant",
    "unstable"
]

[pigment_api]
enabled = true
bind_address = "127.0.0.1:8080"
enable_cors = true
```

### 2. Création des répertoires nécessaires

```bash
sudo mkdir -p /etc/camaleon/presets
sudo mkdir -p /etc/camaleon/honeypots
sudo mkdir -p /var/log/camaleon
```

## Déploiement

### 1. Démarrage manuel

```bash
# Démarrage avec le fichier de configuration par défaut
camaleon start

# Démarrage avec un fichier de configuration personnalisé
camaleon -c /chemin/vers/config.toml start

# Démarrage en mode silencieux
camaleon start --mode silent
```

### 2. Déploiement comme service systemd

Créez un fichier de service systemd :

```bash
sudo nano /etc/systemd/system/camaleon.service
```

Contenu du fichier de service :

```
[Unit]
Description=CAMALEON Adaptive Security Service
After=network.target

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/camaleon -c /etc/camaleon/config.toml start
Restart=on-failure
RestartSec=5
StandardOutput=syslog
StandardError=syslog
SyslogIdentifier=camaleon

[Install]
WantedBy=multi-user.target
```

Activation et démarrage du service :

```bash
sudo systemctl daemon-reload
sudo systemctl enable camaleon
sudo systemctl start camaleon
```

### 3. Vérification du statut

```bash
# Vérification du statut du service
sudo systemctl status camaleon

# Vérification des logs
sudo journalctl -u camaleon -f
```

## Utilisation de l'API

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

## Utilisation de l'interface CLI

Exemples de commandes CLI :

```bash
# Afficher le statut du système
camaleon status

# Configurer la détection système
camaleon eye360 --track-syn --syscalls execve,fork,clone

# Déployer un honeypot
camaleon lurefield --generate ssh --fake-auth --log-keystroke

# Changer la posture défensive
camaleon posture --set mimetic
```

## Support multi-formats

CAMALEON prend en charge l'analyse de différents formats de fichiers :

### 1. Analyse de fichiers CSV

```bash
# Analyse d'un fichier CSV pour détecter des menaces
camaleon analyze --format csv --file /chemin/vers/fichier.csv
```

### 2. Analyse de fichiers de logs

```bash
# Analyse d'un fichier de logs pour détecter des activités suspectes
camaleon analyze --format log --file /chemin/vers/fichier.log
```

## Dépannage

### Problèmes courants et solutions

1. **Erreur de permission pour la capture réseau**
   - Solution : Exécuter avec des privilèges root ou configurer les capabilities
   ```bash
   sudo setcap cap_net_raw,cap_net_admin=eip /usr/local/bin/camaleon
   ```

2. **Module eBPF non fonctionnel**
   - Solution : Vérifier que le noyau Linux est récent (5.4+) et que les headers sont installés
   ```bash
   sudo apt install linux-headers-$(uname -r)
   ```

3. **API non accessible**
   - Solution : Vérifier que le service est en cours d'exécution et que le port n'est pas bloqué
   ```bash
   sudo netstat -tulpn | grep 8080
   ```

## Conclusion

CAMALEON est maintenant installé et configuré sur votre système. Pour plus d'informations sur les fonctionnalités avancées, consultez la documentation complète dans le dossier `docs/` du projet.
