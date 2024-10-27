# Simple Rust Raycaster
A simple untextured raycaster built with the help of [Trigonometry Class Notes (lamar.edu)](https://tutorial.math.lamar.edu/pdf/trig_cheat_sheet.pdf) using Rust and [rust-sdl2: SDL2 bindings for Rust (github.com)](https://github.com/Rust-SDL2/rust-sdl2) as part of project-based learning.

![2024-10-2410-10-50-Trim-ezgif com-video-to-gif-converter](https://github.com/user-attachments/assets/fa057ed6-26a1-4c63-9cb4-99b6d445e1e2)

## Building and Running
 Follow the instructions at [this link (github.com)](https://github.com/Rust-SDL2/rust-sdl2?tab=readme-ov-file#sdl20-development-libraries) to download the necessary SDL libraries and add them to the correct folders for your rust toolchain.
 Make sure to add SDL.dll to the project directory and anywhere you may run the executable, as the build will be dynamically linked to SDL.
Then simply,

    cargo build
and the built executable will be in your target directory.
