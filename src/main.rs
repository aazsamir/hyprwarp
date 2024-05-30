use std::{
    fmt::Display,
    process::{exit, Command},
};

use serde::Deserialize;

fn main() {
    println!("hyprwarp starting...");
    let mut engine = HyprlandEngine::new();
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

#[derive(Debug)]
enum Direction {
    None = 0,
    Left = 1,
    Right = 2,
    Up = 4,
    Down = 8,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::None
    }
}

fn dir_to_string(dir: i32) -> String {
    let mut s = String::new();
    if dir & Direction::Up as i32 != 0 {
        s.push_str("Up ");
    }
    if dir & Direction::Down as i32 != 0 {
        s.push_str("Down ");
    }
    if dir & Direction::Left as i32 != 0 {
        s.push_str("Left ");
    }
    if dir & Direction::Right as i32 != 0 {
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
        let mut dir: i32 = Direction::None as i32;

        if mouse.y == self.y && mouse.x >= self.x && mouse.x <= self.border_x() {
            dir |= Direction::Up as i32;
        }
        if mouse.y == self.border_y() && mouse.x >= self.x && mouse.x <= self.border_x() {
            dir |= Direction::Down as i32;
        }
        if mouse.x == self.x && mouse.y >= self.y && mouse.y <= self.border_y() {
            dir |= Direction::Left as i32;
        }
        if mouse.x == self.border_x() && mouse.y >= self.y && mouse.y <= self.border_y() {
            dir |= Direction::Right as i32;
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

            if dir & Direction::Up as i32 != 0 {
                y = window.border_y();
            }

            if dir & Direction::Down as i32 != 0 {
                y = window.y;
            }

            if dir & Direction::Left as i32 != 0 {
                x = window.border_x();

                if mouse.y < window.y {
                    y = window.y;
                } else if mouse.y > window.border_y() {
                    y = window.border_y();
                }
            }

            if dir & Direction::Right as i32 != 0 {
                x = window.x;
            }

            engine.move_mouse(mouse, x, y);
        }
    }

    fn find_closest_output(&self, dir: i32, mouse: &Mouse) -> Option<&Window> {
        let mut best: Option<&Window> = None;

        for window in &self.windows {
            match dir {
                4 => {
                    // up
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
                8 => {
                    // down
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
                1 => {
                    // left
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
                2 => {
                    // right
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
        // split it to (x,y)
        let cursorpos = String::from_utf8(cursorpos.stdout).unwrap();
        // get rid of \n
        let cursorpos = cursorpos.trim();
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
