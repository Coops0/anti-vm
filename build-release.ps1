cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --release

# cargo bloat --release -n 10
# cargo bloat --release -n 10 --crates