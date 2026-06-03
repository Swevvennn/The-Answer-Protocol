Installer les dépendances	cargo fetch
Compiler	cargo build
Exécuter	cargo run
Linter	cargo clippy
Formater	cargo fmt
Nettoyer	cargo clean

cargo new mon_projet     # créer un projet
cargo run                # compiler et exécuter
cargo build              # compiler
cargo build --release    # compiler optimisé
cargo check              # vérification rapide sans générer le binaire
cargo test               # lancer les tests
cargo fmt                # formater le code
cargo clippy             # analyse de qualité du code
