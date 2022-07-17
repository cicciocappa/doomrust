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

const ZOOM: f64 = 4.0;

mod math;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    keys: Keys,
    tick: u32,
    player: Player,
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
        Self {
            keys,
            tick: 0,
            player,
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
    fn draw(&self, frame: &mut [u8]) {
        self.clear(frame);
        let cs = math::COS[self.player.angle as usize];
        let sn = math::SIN[self.player.angle as usize];

        let x1 = 40 - self.player.x;
        let y1 = 10 - self.player.y;

        let x2 = 40 - self.player.x;
        let y2 = 290 - self.player.y;

        let wx0 = x1 as f64 * cs - y1 as f64 * sn;
        let wx1 = x2 as f64 * cs - y2 as f64 * sn;

        let wx2 = wx0;
        let wx3 = wx1;

        let wy0 = y1 as f64 * cs + x1 as f64 * sn;
        let wy1 = y2 as f64 * cs + x2 as f64 * sn;

        let wy2 = wy0;
        let wy3 = wy1;

        let wz0 = 0.0 - self.player.z as f64 + (self.player.look as f64 * wy0 / 32.0);
        let wz1 = 0.0 - self.player.z as f64 + (self.player.look as f64 * wy1 / 32.0);

        let wz2 = wz0 + 40.0;
        let wz3 = wz1 + 40.0;

        let sx0 = (wx0 * 200.0 / wy0) as i32 + SW2;
        let sy0 = (wz0 * 200.0 / wy0) as i32 + SH2;

        let sx1 = (wx1 * 200.0 / wy1) as i32 + SW2;
        let sy1 = (wz1 * 200.0 / wy1) as i32 + SH2;

        let sx2 = (wx2 * 200.0 / wy2) as i32 + SW2;
        let sy2 = (wz2 * 200.0 / wy2) as i32 + SH2;

        let sx3 = (wx3 * 200.0 / wy3) as i32 + SW2;
        let sy3 = (wz3 * 200.0 / wy3) as i32 + SH2;
        self.draw_wall(frame, sx0, sx1, sy0, sy1, sy2, sy3);
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
    ) {
        let dyb = b2 - b1;
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
                self.pixel(frame, x as u32, y as u32, 0);
            }
        }
    }

    fn clip_behind_player(
        x1: &mut i32,
        y1: &mut i32,
        z1: &mut i32,
        x2: &mut i32,
        y2: &mut i32,
        z2: &mut i32,
    ) {
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
