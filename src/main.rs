use std::{
    fmt::Display,
    process::{exit, Command},
};

use serde::Deserialize;

#[allow(while_true)]
fn main() {
    println!("hyprwarp starting...");
    let engine = HyprlandEngine::new();
    let windows = engine.get_windows();

    let mut last_mouse = engine.get_mouse();

    while true {
        let mouse = engine.get_mouse();

        if last_mouse == mouse {
            sleep();
            continue;
        }

        last_mouse.update(mouse.x, mouse.y);
        let contains = windows.contains_mouse(&mouse);

        match contains {
            Some(window) => {
                let dir = window.mouse_on_border(&mouse);
                windows.warp_to_adjacent_output(window, &mouse, dir, &engine)
            }
            None => {
                println!("Mouse is not in any window");
            }
        }

        sleep();
    }
    println!("hyprwarp finished!");
}

fn sleep() {
    std::thread::sleep(std::time::Duration::from_millis(100));
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
struct Mouse {
    x: i32,
    y: i32,
}

impl Mouse {
    fn new(x: i32, y: i32) -> Mouse {
        Mouse { x, y }
    }

    fn update(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
}

impl Display for Mouse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mouse: x={}, y={}", self.x, self.y)
    }
}

#[derive(Default, Debug, Clone)]
struct Window {
    name: String,
    width: i32,
    height: i32,
    x: i32,
    y: i32,
}

// const
const DIRECTION_NONE: i32 = 0;
const DIRECTION_LEFT: i32 = 1;
const DIRECTION_RIGHT: i32 = 2;
const DIRECTION_UP: i32 = 4;
const DIRECTION_DOWN: i32 = 8;

fn dir_to_string(dir: i32) -> String {
    let mut s = String::new();
    if dir & DIRECTION_UP != 0 {
        s.push_str("Up ");
    }
    if dir & DIRECTION_DOWN != 0 {
        s.push_str("Down ");
    }
    if dir & DIRECTION_LEFT != 0 {
        s.push_str("Left ");
    }
    if dir & DIRECTION_RIGHT != 0 {
        s.push_str("Right ");
    }

    s
}

impl Window {
    fn new(name: &str, width: i32, height: i32, x: i32, y: i32) -> Window {
        Window {
            name: name.to_string(),
            width,
            height,
            x,
            y,
        }
    }

    fn update(&mut self, width: i32, height: i32, x: i32, y: i32) {
        self.width = width;
        self.height = height;
        self.x = x;
        self.y = y;
    }

    fn border_x(&self) -> i32 {
        self.x + self.width
    }

    fn border_y(&self) -> i32 {
        self.y + self.height
    }

    fn contains_mouse(&self, mouse: &Mouse) -> bool {
        self.contains(mouse.x, mouse.y)
    }

    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x <= self.border_x() && y >= self.y && y <= self.border_y()
    }

    fn mouse_on_border(&self, mouse: &Mouse) -> i32 {
        let mut dir: i32 = DIRECTION_NONE;

        if mouse.y == self.y && mouse.x >= self.x && mouse.x <= self.border_x() {
            dir |= DIRECTION_UP;
        }
        if mouse.y == self.border_y() && mouse.x >= self.x && mouse.x <= self.border_x() {
            dir |= DIRECTION_DOWN;
        }
        if mouse.x == self.x && mouse.y >= self.y && mouse.y <= self.border_y() {
            dir |= DIRECTION_LEFT;
        }
        if mouse.x == self.border_x() && mouse.y >= self.y && mouse.y <= self.border_y() {
            dir |= DIRECTION_RIGHT;
        }

        dir
    }
}

impl Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Window {}: width={}, height={}, x={}, y={}",
            self.name, self.width, self.height, self.x, self.y
        )
    }
}

#[derive(Default, Debug, Clone)]
struct Windows {
    windows: Vec<Window>,
}

impl Windows {
    fn new() -> Windows {
        Windows {
            windows: Vec::new(),
        }
    }

    fn add(&mut self, window: Window) {
        self.windows.push(window);
    }

    fn remove(&mut self, index: usize) {
        self.windows.remove(index);
    }

    fn contains_mouse(&self, mouse: &Mouse) -> Option<&Window> {
        self.contains(mouse.x, mouse.y)
    }

    fn contains(&self, x: i32, y: i32) -> Option<&Window> {
        for window in &self.windows {
            if window.contains(x, y) {
                return Some(window);
            }
        }
        None
    }

    fn warp_to_adjacent_output(
        &self,
        current: &Window,
        mouse: &Mouse,
        dir: i32,
        engine: &impl Engine,
    ) {
        let closest = self.find_closest_output(dir, &mouse);

        if let Some(window) = closest {
            let mut x = mouse.x;
            let mut y = mouse.y;

            if dir & DIRECTION_UP as i32 != 0 {
                y = window.border_y();

                if mouse.x < window.x {
                    x = window.x;
                } else if mouse.x > window.border_x() {
                    x = window.border_x();
                }
            }

            if dir & DIRECTION_DOWN as i32 != 0 {
                y = window.y;

                if mouse.x < window.x {
                    x = window.x;
                } else if mouse.x > window.border_x() {
                    x = window.border_x();
                }
            }

            if dir & DIRECTION_LEFT as i32 != 0 {
                x = window.border_x();

                if mouse.y < window.y {
                    y = window.y;
                } else if mouse.y > window.border_y() {
                    y = window.border_y();
                }
            }

            if dir & DIRECTION_RIGHT as i32 != 0 {
                x = window.x;

                if mouse.y < window.y {
                    y = window.y;
                } else if mouse.y > window.border_y() {
                    y = window.border_y();
                }
            }

            engine.move_mouse(mouse, x, y);
        }
    }

    fn find_closest_output(&self, dir: i32, mouse: &Mouse) -> Option<&Window> {
        let mut best: Option<&Window> = None;

        for window in &self.windows {
            match dir {
                DIRECTION_UP => {
                    if window.y <= mouse.y {
                        if let Some(best_window) = best {
                            if window.y > best_window.y {
                                best = Some(window);
                            }
                        } else {
                            best = Some(window);
                        }
                    }
                }
                DIRECTION_DOWN => {
                    if window.border_y() >= mouse.y {
                        if let Some(best_window) = best {
                            if window.border_y() < best_window.border_y() {
                                best = Some(window);
                            }
                        } else {
                            best = Some(window);
                        }
                    }
                }
                DIRECTION_LEFT => {
                    if window.border_x() <= mouse.x {
                        if let Some(best_window) = best {
                            if window.border_x() > best_window.border_x() {
                                best = Some(window);
                            }
                        } else {
                            best = Some(window);
                        }
                    }
                }
                DIRECTION_RIGHT => {
                    if window.x >= mouse.x {
                        if let Some(best_window) = best {
                            if window.x < best_window.x {
                                best = Some(window);
                            }
                        } else {
                            best = Some(window);
                        }
                    }
                }
                _ => {}
            }
        }

        best
    }
}

impl Display for Windows {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Windows: {:?}", self.windows)
    }
}

trait Engine {
    fn get_mouse(&self) -> Mouse;
    fn get_windows(&self) -> Windows;
    fn move_mouse(&self, mouse: &Mouse, x: i32, y: i32);
}

struct HyprlandEngine {}

impl HyprlandEngine {
    fn new() -> HyprlandEngine {
        HyprlandEngine {}
    }
}

impl Engine for HyprlandEngine {
    fn get_mouse(&self) -> Mouse {
        let cursorpos = Command::new("hyprctl")
            .arg("cursorpos")
            .output()
            .expect("failed to execute `hyprctl cursorpos`");

        // stdout is "123, 456"
        let cursorpos = String::from_utf8(cursorpos.stdout).unwrap();
        // get rid of \n
        let cursorpos = cursorpos.trim();
        // split it to (x,y)
        let mut parts = cursorpos.split(", ");
        let x = parts.next().unwrap().parse::<i32>().unwrap();
        let y = parts.next().unwrap().parse::<i32>().unwrap();

        Mouse::new(x, y)
    }

    fn get_windows(&self) -> Windows {
        let windows = Command::new("hyprctl")
            .arg("-j")
            .arg("monitors")
            .output()
            .expect("failed to execute `hyprctl -j monitors`");

        let windows = String::from_utf8(windows.stdout).unwrap();
        let hypr_windows: Vec<HyprlandMonitor> = serde_json::from_str(&windows).unwrap();

        let mut windows = Windows::new();

        for window in hypr_windows {
            if !window.disabled {
                windows.add(Window::new(
                    &window.name,
                    window.width,
                    window.height,
                    window.x,
                    window.y,
                ));
            }
        }

        windows
    }

    fn move_mouse(&self, mouse: &Mouse, x: i32, y: i32) {
        // calc relative movement
        let x = x - mouse.x;
        let y = y - mouse.y;

        Command::new("ydotool")
            .arg("mousemove")
            .arg("-x")
            .arg(x.to_string())
            .arg("-y")
            .arg(y.to_string())
            .output()
            .expect("failed to execute `ydotool mousemove`");
    }
}

#[derive(Default, Debug, Clone, Deserialize)]
struct HyprlandMonitor {
    name: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    disabled: bool,
}
