//! Game struct registration

use serde::Serialize;

/// Game information for rendering
#[derive(Serialize)]
pub struct Game {
    /// Path to the WASM runtime
    pub wasm_path: &'static str,
    /// Path to thumbnail image
    pub img: &'static str,
    /// Description
    pub name: &'static str,
}

macro_rules! game {
    ($wasm:expr_2021, $img:expr_2021, $descr:expr_2021) => {
        Game {
            wasm_path: $wasm,
            img: $img,
            name: $descr,
        }
    };
}

/// All registered games
pub const GAMES: &'static [Game] = &[game!(
    "wasm/cube/out/cube.wasm",
    "/frontend/bg/cube.png",
    "THE_CUBE"
)];
