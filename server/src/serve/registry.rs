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
            <div class="game-box"  hx-get="sports/{}" hx-target="body">
                <img src="{}" alt="{}" class="game-thumbnail" />
                <div class="game-name">{}</div>
            </div>
            "#,
            self.name, self.img, self.name, self.name
        )
    }

    /// Renders a gameplay page for the Game
    pub fn render_game_scene(&self) -> String {
        format!(
            r#"
            <!DOCTYPE html>
            <html>
                <head>
                    <meta charset="UTF-8" />
                    <link rel="stylesheet" href="frontend/style/game.css">
                </head>
                <body>
                    <title>{}</title>
                    <div class="loader"></div>
                    <script>
                        (function () {{
                            const audioContextList = [];

                            const userInputEventNames = [
                                "click",
                                "contextmenu",
                                "auxclick",
                                "dblclick",
                                "mousedown",
                                "mouseup",
                                "pointerup",
                                "touchend",
                                "keydown",
                                "keyup",
                            ];

                            self.AudioContext = new Proxy(self.AudioContext, {{
                                construct(target, args) {{
                                    const result = new target(...args);
                                    audioContextList.push(result);
                                    return result;
                                }},
                            }});

                            function resumeAllContexts(_event) {{
                                let count = 0;

                                audioContextList.forEach((context) => {{
                                    if (context.state !== "running") {{
                                        context.resume();
                                    }} else {{
                                        count++;
                                    }}
                                }});

                                if (count > 0 && count === audioContextList.length) {{
                                    userInputEventNames.forEach((eventName) => {{
                                        document.removeEventListener(eventName, resumeAllContexts);
                                    }});
                                }}
                            }}

                            userInputEventNames.forEach((eventName) => {{
                                document.addEventListener(eventName, resumeAllContexts);
                            }});
                        }})();
                    </script>
                    <script type="module">
                        import init from '{}'
                        init();
                    </script>
                </body>
            </html>
            "#,
            self.name, self.wasm_path
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
    "/wasm/cube/out/cube.js",
    "/frontend/bg/cube.png",
    "THE_CUBE"
)];
