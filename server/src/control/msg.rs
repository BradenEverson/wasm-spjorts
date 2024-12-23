//! Controller message protocol

/// Messages a controller can send through
pub enum ControllerMessage {
    /// Keep-alive signal
    Heartbeat,
    /// Connect to a new user
    Connect,
    /// Press A button
    ButtonPressA,
    /// Press B button
    ButtonPressB,
    /// Update current angle (pitch, roll, yaw)
    AngleInfo(f32, f32, f32),
}
