# Hyprwarp

Inspired by the [xedgewarp](https://github.com/Airblader/xedgewarp) project. 
This is a simple program written in Rust that warps the mouse cursor to adjacent monitors when it reaches the edge of the screen.

It depends on [Hyprland](https://hyprland.org/) for monitors information and cursor position, and [ydotool](https://github.com/ReimuNotMoe/ydotool) to move the cursor.

## Installation

Build the project with `cargo build --release` and copy the binary to your $PATH.
```bash
./bin/install.sh
```

Add to your `hyprland.conf`
```
exec-once ydotoold
exec-once hyprward
