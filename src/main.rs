use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

const SH2: i32 = HEIGHT as i32 / 2;
const SW2: i32 = WIDTH as i32 / 2;

const ZOOM: f64 = 3.0;

mod math;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    keys: Keys,
    tick: u32,
    player: Player,
    sectors: Vec<Sector>,
    walls: Vec<Wall>,
}

struct Keys {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    strafe_left: bool,
    strafe_right: bool,
    look: bool,
}

struct Player {
    x: i32,
    y: i32,
    z: i32,
    angle: i32,
    look: i32,
}

struct Wall {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    color: u8,
}

struct Sector {
    wall_start: usize,
    wall_end: usize,
    z1: i32,
    z2: i32,
    x: i32,
    y: i32,
    distance: i32,
}

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(ZOOM * WIDTH as f64, ZOOM * HEIGHT as f64);
        WindowBuilder::new()
            .with_title("DoomRust")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| println!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            world.keys.down = input.key_held(VirtualKeyCode::S);
            world.keys.up = input.key_held(VirtualKeyCode::W);
            world.keys.left = input.key_held(VirtualKeyCode::A);
            world.keys.right = input.key_held(VirtualKeyCode::D);
            world.keys.strafe_left = input.key_held(VirtualKeyCode::Comma);
            world.keys.strafe_right = input.key_held(VirtualKeyCode::Period);
            world.keys.look = input.key_held(VirtualKeyCode::M);
            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
    });
}

impl World {
    /// Create a new `World` instance that can draw the level.
    fn new() -> Self {
        let keys = Keys {
            up: false,
            down: false,
            left: false,
            right: false,
            strafe_left: false,
            strafe_right: false,
            look: false,
        };
        let player = Player {
            x: 70,
            y: -110,
            z: 20,
            angle: 0,
            look: 0,
        };
        let mut walls = Vec::new();
        let mut sectors = Vec::new();
        let init_sectors = [0, 4, 0, 40, 4, 8, 0, 40, 8, 12, 0, 40, 12, 16, 0, 40];

        let init_walls = [
            0, 0, 32, 0, 0, 32, 0, 32, 32, 1, 32, 32, 0, 32, 0, 0, 32, 0, 0, 1, 64, 0, 96, 0, 2,
            96, 0, 96, 32, 3, 96, 32, 64, 32, 2, 64, 32, 64, 0, 3, 64, 64, 96, 64, 4, 96, 64, 96,
            96, 5, 96, 96, 64, 96, 4, 64, 96, 64, 64, 5, 0, 64, 32, 64, 6, 32, 64, 32, 96, 7, 32,
            96, 0, 96, 6, 0, 96, 0, 64, 7,
        ];

        for n in 0..4 {
            sectors.push(Sector {
                wall_start: init_sectors[n * 4],
                wall_end: init_sectors[n * 4 + 1],
                x: 0,
                y: 0,
                distance: 0,
                z1: init_sectors[n * 4 + 2] as i32,
                z2: init_sectors[n * 4 + 3] as i32,
            });
        }

        for n in 0..16 {
            walls.push(Wall {
                x1: init_walls[n * 5],
                y1: init_walls[n * 5 + 1],
                x2: init_walls[n * 5 + 2],
                y2: init_walls[n * 5 + 3],
                color: init_walls[n * 5 + 4] as u8,
            });
        }

        Self {
            keys,
            tick: 0,
            player,
            sectors,
            walls,
        }
    }

    /// Update the `World` internal state;
    fn update(&mut self) {
        self.tick += 1;
        if self.tick < 4 {
            return;
        }
        self.tick = 0;
        if self.keys.left && !self.keys.look {
            self.player.angle -= 4;
            if self.player.angle < 0 {
                self.player.angle += 360;
            }
        }
        if self.keys.right && !self.keys.look {
            self.player.angle += 4;
            if self.player.angle > 359 {
                self.player.angle -= 360;
            }
        }

        let dx = math::SIN[self.player.angle as usize] * 10.0;
        let dy = math::COS[self.player.angle as usize] * 10.0;
        if self.keys.up && !self.keys.look {
            self.player.x += dx as i32;
            self.player.y += dy as i32;
        }
        if self.keys.down && !self.keys.look {
            self.player.x -= dx as i32;
            self.player.y -= dy as i32;
        }
        if self.keys.strafe_left {
            self.player.x -= dy as i32;
            self.player.y += dx as i32;
        }
        if self.keys.strafe_right {
            self.player.x += dy as i32;
            self.player.y -= dx as i32;
        }

        if self.keys.left && self.keys.look {
            self.player.look -= 1;
        }
        if self.keys.right && self.keys.look {
            self.player.look += 1;
        }
        if self.keys.up && self.keys.look {
            self.player.z -= 4;
        }
        if self.keys.down && self.keys.look {
            self.player.z += 4;
        }
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&mut self, frame: &mut [u8]) {
        self.clear(frame);
        let cs = math::COS[self.player.angle as usize];
        let sn = math::SIN[self.player.angle as usize];

        self.sectors.sort_by(|a, b| b.distance.cmp(&a.distance));

        for s in 0..self.sectors.len() {
            self.sectors[s].distance = 0;
            for w in self.sectors[s].wall_start..self.sectors[s].wall_end {
                let x1 = self.walls[w].x1 - self.player.x;
                let y1 = self.walls[w].y1 - self.player.y;

                let x2 = self.walls[w].x2 - self.player.x;
                let y2 = self.walls[w].y2 - self.player.y;

                let mut wx0 = x1 as f64 * cs - y1 as f64 * sn;
                let mut wx1 = x2 as f64 * cs - y2 as f64 * sn;
                let mut wx2 = wx0;
                let mut wx3 = wx1;

                let mut wy0 = y1 as f64 * cs + x1 as f64 * sn;
                let mut wy1 = y2 as f64 * cs + x2 as f64 * sn;
                let mut wy2 = wy0;
                let mut wy3 = wy1;

                self.sectors[s].distance +=
                    World::distance(0, 0, (wx0 + wx1) as i32 / 2, (wy0 + wy1) as i32 / 2);

                let mut wz0 = self.sectors[s].z1 as f64 - self.player.z as f64
                    + (self.player.look as f64 * wy0 / 32.0);
                let mut wz1 = self.sectors[s].z1 as f64 - self.player.z as f64
                    + (self.player.look as f64 * wy1 / 32.0);
                let mut wz2 = wz0 + self.sectors[s].z2 as f64;
                let mut wz3 = wz1 + self.sectors[s].z2 as f64;

                if wy0 < 1.0 && wy1 < 1.0 {
                    continue;
                }
                if wy0 < 1.0 {
                    World::clip_behind_player(&mut wx0, &mut wy0, &mut wz0, wx1, wy1, wz1);
                    World::clip_behind_player(&mut wx2, &mut wy2, &mut wz2, wx3, wy3, wz3);
                }
                if wy1 < 1.0 {
                    World::clip_behind_player(&mut wx1, &mut wy1, &mut wz1, wx0, wy0, wz0);
                    World::clip_behind_player(&mut wx3, &mut wy3, &mut wz3, wx2, wy2, wz2);
                }
                /*
                let sx0 = (wx0 * 200.0 / wy0) as i32 + SW2;
                let sy0 = (wz0 * 200.0 / wy0) as i32 + SH2;

                let sx1 = (wx1 * 200.0 / wy1) as i32 + SW2;
                let sy1 = (wz1 * 200.0 / wy1) as i32 + SH2;

                let sx2 = (wx2 * 200.0 / wy2) as i32 + SW2;
                let sy2 = (wz2 * 200.0 / wy2) as i32 + SH2;

                let sx3 = (wx3 * 200.0 / wy3) as i32 + SW2;
                let sy3 = (wz3 * 200.0 / wy3) as i32 + SH2;
                */

                let sx0 = ((wx0 * 200.0 / wy0) as i32).saturating_add(SW2);
                let sy0 = ((wz0 * 200.0 / wy0) as i32).saturating_add(SH2);

                let sx1 = ((wx1 * 200.0 / wy1) as i32).saturating_add(SW2);
                let sy1 = ((wz1 * 200.0 / wy1) as i32).saturating_add(SH2);

                let sx2 = ((wx2 * 200.0 / wy2) as i32).saturating_add(SW2);
                let sy2 = ((wz2 * 200.0 / wy2) as i32).saturating_add(SH2);

                let sx3 = ((wx3 * 200.0 / wy3) as i32).saturating_add(SW2);
                let sy3 = ((wz3 * 200.0 / wy3) as i32).saturating_add(SH2);

                self.draw_wall(frame, sx0, sx1, sy0, sy1, sy2, sy3, self.walls[w].color);
            }
            let num_wall = (self.sectors[s].wall_end - self.sectors[s].wall_start) as i32;
            self.sectors[s].distance /= num_wall;
        }
    }

    fn draw_wall(
        &self,
        frame: &mut [u8],
        mut x1: i32,
        mut x2: i32,
        b1: i32,
        b2: i32,
        t1: i32,
        t2: i32,
        color: u8,
    ) {
        //let dyb = b2 - b1;
        let dyb = b2.saturating_sub(b1);
        let dyt = t2 - t1;
        let mut dx = x2 - x1;
        if dx == 0 {
            dx = 1;
        }
        let xs = x1;
        if x1 < 1 {
            x1 = 1;
        }
        if x2 < 1 {
            x2 = 1;
        }
        if x1 > WIDTH as i32 - 1 {
            x1 = WIDTH as i32 - 1;
        }
        if x2 > WIDTH as i32 - 1 {
            x2 = WIDTH as i32 - 1;
        }
        for x in x1..x2 {
            let mut y1 = (dyb as f64 * ((x - xs) as f64 + 0.5) / (dx as f64)) as i32 + b1;
            let mut y2 = (dyt as f64 * ((x - xs) as f64 + 0.5) / (dx as f64)) as i32 + t1;
            if y1 < 1 {
                y1 = 1;
            }
            if y2 < 1 {
                y2 = 1;
            }
            if y1 > HEIGHT as i32 - 1 {
                y1 = HEIGHT as i32 - 1;
            }
            if y2 > HEIGHT as i32 - 1 {
                y2 = HEIGHT as i32 - 1;
            }
            for y in y1..y2 {
                self.pixel(frame, x as u32, y as u32, color);
            }
        }
    }

    fn clip_behind_player(x1: &mut f64, y1: &mut f64, z1: &mut f64, x2: f64, y2: f64, z2: f64) {
        let da = *y1;
        let db = y2;
        let mut d = da - db;
        if d == 0.0 {
            d = 1.0;
        }
        let s = da / d;
        *x1 = *x1 + (s * (x2 - *x1));
        *y1 = *y1 + (s * (y2 - *y1));
        if *y1 == 0.0 {
            *y1 = 1.0;
        }
        *z1 = *z1 + (s * (z2 - *z1));
    }

    fn distance(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
        (x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1)
    }

    fn clear(&self, frame: &mut [u8]) {
        for pixel in frame.chunks_exact_mut(4) {
            pixel[0] = 0; // R
            pixel[1] = 60; // G
            pixel[2] = 130; // B
            pixel[3] = 0xff; // A
        }
    }
    fn pixel(&self, frame: &mut [u8], x: u32, y: u32, c: u8) {
        let rgb = match c {
            0 => [255, 255, 0],
            1 => [160, 160, 0],
            2 => [0, 255, 0],
            3 => [0, 160, 0],
            4 => [0, 255, 255],
            5 => [0, 160, 160],
            6 => [160, 100, 0],
            7 => [110, 50, 160],
            _ => [0, 60, 130],
        };
        let i = ((y * WIDTH + x) * 4) as usize;
        frame[i] = rgb[0];
        frame[i + 1] = rgb[1];
        frame[i + 2] = rgb[2];
    }
}
