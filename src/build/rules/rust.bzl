HCTL2_RUST_EDITION = "2024"
HCTL2_RUSTC_FLAGS = ["-Dwarnings"]
HCTL2_VERSION = "0.0.0"

def hctl2_cargo_env() -> dict[str, str]:
    return {
        "CARGO_PKG_VERSION": HCTL2_VERSION,
    }
