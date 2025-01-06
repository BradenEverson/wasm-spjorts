//! Game struct registration

use serde::Serialize;

/// Render valid html for an ID being connected to
pub fn render_id_connection(id: u64) -> String {
    format!(
        r#"
            <div class="id-box" hx-get="/connect?id={}" hx-swap="none" value="{}">
                <div class="name">#{}</div>
            </div>
            "#,
        id, id, id
    )
}

/// Game information for rendering
#[derive(Serialize)]
pub struct Game {
    /// Path to the WASM runtime
    pub wasm_path: &'static str,
    /// Path to thumbnail image
    pub img: &'static str,
    /// Description
    pub name: &'static str,
    /// If a game is multiplayer or not
    pub multiplayer: bool,
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
                        import init, {{ Runner }} from '{}'

                        const socket = new WebSocket("/");
                        socket.binaryType = "arraybuffer";

                        socket.addEventListener("open", () => {{
                            console.log("WebSocket connection opened");
                            let id = parseInt(localStorage.getItem("ID"));
                            console.log(`ID: ${{id}}`);
                            const buffer = createWsMessage(1, id);

                            socket.send(buffer);
                            console.log("ArrayBuffer sent:", buffer);
                        }});

                        function createWsMessage(id, payload) {{
                            const buffer = new ArrayBuffer(9);
                            const dataView = new DataView(buffer);

                            dataView.setUint8(0, id);

                            const bigIntPayload = BigInt(payload);
                            dataView.setBigUint64(1, bigIntPayload, true); 

                            return buffer;
                        }}

                        init().then(() => {{
                            let runner = new Runner();
                            let send = runner.get_send();

                            if ({}) {{
                                let players = parseInt(prompt("How many players:"));
                                send.set_players(players);
                            }}

                            socket.addEventListener("message", (event) => {{
                                const buffer = event.data;
                                const dataView = new DataView(buffer);
                                const id = dataView.getUint8(0);

                                switch (id) {{
                                    case 2:
                                        // Button A
                                        send.press_a();
                                        break;
                                    case 3:
                                        // Button B
                                        send.press_b();
                                        break;
                                    case 4:
                                        // Angle data
                                        const pitch = dataView.getFloat32(1, true);
                                        const yaw = dataView.getFloat32(5, true);
                                        const roll = dataView.getFloat32(9, true);
                                        send.rotate(pitch, yaw, roll);
                                        break;
                                    default:
                                        console.log("Unknown ID found: ", id);
                                }}

                            }});

                            socket.addEventListener("error", (error) => {{
                                console.error("WebSocket error:", error);
                            }});

                            socket.addEventListener("close", () => {{
                                console.log("WebSocket connection closed");
                            }});
                            
                            console.log("Run has begun");
                            runner.run();
                        }});

                    </script>
                </body>
            </html>
            "#,
            self.name, self.wasm_path, self.multiplayer
        )
    }
}

macro_rules! game {
    ($wasm:expr_2021, $img:expr_2021, $descr:expr_2021, $mult:expr_2021) => {
        Game {
            wasm_path: $wasm,
            img: $img,
            name: $descr,
            multiplayer: $mult,
        }
    };
}

/// All registered games
pub const GAMES: &'static [Game] = &[
    game!(
        "/wasm/cube/out/cube.js",
        "/frontend/bg/cube.png",
        "THE_CUBE",
        false
    ),
    game!(
        "/wasm/bowling/out/bowling.js",
        "/frontend/bg/bowling.jpg",
        "Bowling",
        true
    ),
];
