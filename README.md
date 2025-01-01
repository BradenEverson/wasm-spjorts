# Spjörts 🏂🎾⛳
## Server Architecture for hosting WASM-built games over the web, supporting communication through physical controller hardware 💃

This project exists because I really wanted to make [Wii Sports](https://en.wikipedia.org/wiki/Wii_Sports), while also learning more about the Bevy game engine, WASM and optimized embedded systems that need to continuously stream timely data. 

Using the Deku binary format, optimized websocket streams are able to communicate button presses and gyroscope angle readings over from a Raspberry Pi controller to the server itself, which then streams from another websocket to all frontend "listener" streams, which finally are then fed into the WASM games via a standardized `Sender` struct that comes along when a game is initialized.

Long story short there's a large chain of communication that essentially goes from 

`[ Controller ] -> [ Web Server ] -> [ Js Frontend ] -> [ WASM Game ]`

# Implemented Games
- [x] THE CUBE 🧊
  * "Game" meant for early debugging purposes.
  * Controls a cube floating in the void, pitch roll and yaw updates determine the cube's rotation and pressing the two controller buttons moves it along the X axis in different directions.
- [ ] Bowling 🎳
  * Implementation of a standard bowling game, currently supports moving along the x axis for aiming.
  * "Throwing" the ball via aiming with the physical controller's angle data and hitting the A button for release propels the ball forward at that angle with velocity according to the rate at which the ball is thrown
  * Rapier3d is used for all physics simulation, which allows for (somewhat) accurate bowling!
  - [ ] **Still in progress**: Implementing proper score handling and external game state info, basically everything that isn't aiming and throwing a ball
