![image](camaleon.png)


# 🦎 Projet C.A.M.A.L.E.O.N.

**Cybernetic Adaptive Morphing Agent for Layered Environment Observation & Neutralization**

Un bijou défensif adaptatif, bio-inspiré, capable de changer de forme, de signature, de surface d'exposition en temps réel pour déstabiliser les attaquants, les berner, ou les forcer à exposer leurs outils.

## 🧬 Bio-inspiration → Cyber-conversion

| Comportement naturel du caméléon | Traduction en cybersécurité |
|----------------------------------|----------------------------|
| Camouflage adaptatif | Mutation des bannières, ports, OS fingerprint |
| Vision stéréoscopique 360° | Double moteur de détection : réseau + système |
| Langue rapide et ciblée | Réaction rapide, ciblée, silencieuse |
| Déplacement lent, contrôlé | Ne perturbe pas l'environnement sauf si attaqué |
| Changements de couleur contextuels | Change de posture défensive selon l'intensité/menace |

## 🔧 Composants principaux

```
/chameleon
├── chame_core/     ← Cœur adaptatif (profilage, changement à chaud)
├── skinshift/      ← Fausse bannière, OS fingerprint morphing
├── eye360/         ← Détection système (eBPF, syscalls, logs)
├── nettongue/      ← Détection réseau passive (PCAP + latency fuzz)
├── lurefield/      ← Micro-honeypots adaptatifs injectés au besoin
├── pigment_api/    ← API locale pour pilotage en live (mode couleurs)
└── posture_engine/ ← Décide de l'attitude : furtif, défensif, leurre, escalade
```

## 🧠 Modes adaptatifs (aka *postures comportementales*)

| Mode | Apparence simulée | Comportement défensif |
|------|-------------------|----------------------|
| 🟢 **Silencieux** | Aucun service visible, ports aléatoires, OS inconnu | Observe, ne répond jamais, logue uniquement |
| 🔵 **Neutre** | Serveur Linux standard, bannières réalistes | Réponse normale mais monitorée |
| 🟠 **Mimétique** | Faux service vulnérable ciblé | Déclenche un honeypot personnalisé |
| 🔴 **Fulgurance** | Comportement erratique ou piégeur | Freeze de connexion, perturbation réseau ciblée |
| 🟣 **Instable** | Comportement changeant pour semer la confusion | Simule une instabilité système ou un crash |

## 🎯 Objectifs opérationnels

1. **Cloaking dynamique** : Empêcher fingerprint OS via `nmap`, `p0f`, `TTL`, `SYN`, etc.
2. **Port Morphing** : Changement régulier des services (SSH devient HTTP, etc.)
3. **Mimétisme ciblé** : Simule **le type de système que l'attaquant semble chercher** (Windows vulnérable, routeur chinois, etc.)
4. **Mini-honeypots injectés à la volée** : service jetable mimant la vulnérabilité appât
5. **Auto-adaptation** : change de posture à partir de l'intensité + nature des paquets
6. **Mode paranoïde** : bloque activement les scans et fait croire à un blocage matériel

## 🧠 Stack technologique

| Domaine | Tech utilisée |
|---------|--------------|
| Core adaptatif | `Rust` + `Tokio` (pour les performances et la sûreté) |
| Faux OS fingerprint | `p0f`, `netfilter`, `iptables`, `tcpdump`, `faked-banner` |
| Détection réseau | `libpcap`, `nfqueue`, `suricata-lite` |
| Vision système | `eBPF` (tracepoints, syscall monitor), `procfs`, `auditd` |
| UI / API | API REST locale (mode dashboard ou CLI), affichage via WebAssembly |
| Honeypots | Services générés à la volée via container/firejail ou WASM |

## 🧪 Exemples de commandes

```bash
# Lancer en mode "furtif total"
chameleon start --mode silent

# Mimer un serveur Windows vulnérable
chameleon skinshift --preset "win2008_smb1"

# Détecter les tentatives d'analyse TCP
chameleon eye360 --track-syn

# Inverser les services exposés
chameleon posture --rotate-services

# Générer un honeypot ciblé MongoDB
chameleon lurefield --generate mongodb --fake-auth --log-keystroke
```

## 📈 Feuille de route (MVP)

| Sprint | Livrables | Durée |
|--------|-----------|-------|
| S0 | Scaffolding CLI + profilage réseau local | 1 semaine |
| S1 | Skinshift : changement fingerprint (OS, bannière, TTL) | 2 semaines |
| S2 | Lurefield : création de faux services dynamiques | 2 semaines |
| S3 | Eye360 + NetTongue : détection passive réseau + système | 3 semaines |
| S4 | Posture Engine (moteur d'adaptation) + UI/API | 2 semaines |
| S5 | Intégration live Kali ou ISO bootable pour blue team terrain | - |

## ⚙️ Installation

Instructions d'installation à venir...

## 📜 Licence

Ce projet est sous licence MIT - voir le fichier [LICENSE](LICENSE) pour plus de détails.
