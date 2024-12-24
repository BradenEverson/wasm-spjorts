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

impl Game {
    /// Creates valid renderable HTML for a Game element
    pub fn render_html(&self) -> String {
        format!(
            r#"
            <div class="game-box" onclick="window.location.href='{}'">
                <img src="{}" alt="{}" class="game-thumbnail" />
                <div class="game-name">{}</div>
            </div>
            "#,
            self.name, self.img, self.name, self.name
        )
    }
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
