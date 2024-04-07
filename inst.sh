sudo mkdir -p /usr/share/oreon/oreon-system-manager && sudo cp src/style.css /usr/share/oreon/oreon-system-manager
cargo build --release && sudo cp target/release/oreon-system-manager /usr/bin
