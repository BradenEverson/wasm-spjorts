//! Game Communication Protocol

/// All messages that can be send via a JavaScript web socket
pub enum JsMessage {
    /// Rotate by (pitch, roll, yaw)
    Rotate(f32, f32, f32),
    /// Press A button
    ButtonA,
    /// Press B button
    ButtonB,
    /// Set number of players in a game
    SetPlayers(usize),
}
