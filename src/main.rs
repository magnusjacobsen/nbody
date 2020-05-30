pub mod nbody;
use nbody::Body;
use std::io::{self, BufRead};
use std::str::{FromStr};
use std::fs::File;

use ggez;
use ggez::{timer, graphics};
use ggez::nalgebra as na;
use ggez::{Context, GameResult};
use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler, KeyCode};
use ggez::input::keyboard;
use std::collections::HashMap;

const AU: f64 = 1.4960e+11; // in meters, roughly distance earth -> sun

struct MainState {
    pub bodies: Vec<Body>,
    dt: f64,
    pub trajectories: Vec<Vec<na::Point2<f32>>>,
    pub time_to_save: u8,
    pub draw_trajectory: bool,
    is_running: bool,
    pub desired_fps: u32,
    pub camera: (f32,f32),
    pub scale: f64,
    scale_constant: f64,
    pressed_keys: Vec<(KeyCode,bool)>,
    keys_up: HashMap<KeyCode, bool>,
    keys_down: HashMap<KeyCode, bool>,
}

impl MainState {
    fn new(bodies: Vec<Body>, dt: f64, draw_trajectory: bool) -> GameResult<MainState> {
        let trajectories = vec![vec![]; bodies.len()];

        let mut pressed_keys = vec![];
        let mut keys_up = HashMap::new();
        let mut keys_down = HashMap::new();
        let keys = vec![KeyCode::Space, KeyCode::T, KeyCode::R,];
        for key in keys {
            pressed_keys.push((key, false));
            keys_up.insert(key, false);
            keys_down.insert(key, false);
        }

        let scale_constant = 20.0;
        let scale = scale_constant / nbody::AU;

        let s = MainState {
            bodies: bodies, 
            dt: dt, 
            trajectories: trajectories, 
            time_to_save: 1,
            draw_trajectory: draw_trajectory,
            is_running: true,
            desired_fps: 2000,
            camera: (0.0, 0.0),
            scale: scale,
            scale_constant: scale_constant,
            pressed_keys: pressed_keys,
            keys_up: keys_up,
            keys_down: keys_down,
        };
        Ok(s)
    }

    fn tick(&mut self) {
        nbody::move_all_bodies(&mut self.bodies, self.dt);
        self.time_to_save -= 1;
        if self.time_to_save == 0 {
            for i in 0..self.bodies.len() {
                let (x,y) = scale_pos(&self.bodies[i], self.scale);
                let p = na::Point2::new(x as f32, y as f32);
                self.trajectories[i].push(p);
            }
            self.time_to_save = 10;
        }
    }
}

fn scale_pos(b: &Body, scale: f64) -> (f32, f32) {
    ((b.pos.0 * scale) as f32, (b.pos.1 * scale) as f32)
}

fn update_key_activity(ctx: &mut Context, state: &mut MainState) {
    let mut next = vec![];
    for (key, pressed) in &state.pressed_keys {
        let current_val = keyboard::is_key_pressed(ctx, *key);
        if current_val && !*pressed {
            *state.keys_down.get_mut(key).unwrap() = true;
        } else if !current_val && *pressed {
            *state.keys_up.get_mut(key).unwrap() = true;
        } else {
            if state.keys_down[key] {
                *state.keys_down.get_mut(key).unwrap() = false;
            }
            if state.keys_up[key] {
                *state.keys_up.get_mut(key).unwrap() = false;
            }
        }
        next.push((*key, current_val));
    }
    state.pressed_keys = next;
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {

        update_key_activity(ctx, self);

        if self.keys_down[&KeyCode::R] {
            self.trajectories = vec![vec![]; self.bodies.len()];
        }
        if self.keys_down[&KeyCode::Space] {
            self.is_running ^= true;
        }
        if self.keys_down[&KeyCode::T] {
            self.draw_trajectory ^= true;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Down) {
            if self.desired_fps > 10 {
                self.desired_fps -= 10;
            }
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Up) {
            self.desired_fps += 10;
        }        
        if keyboard::is_key_pressed(ctx, KeyCode::W) {
            self.camera.1 += 1.0;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::A) {
            self.camera.0 += 1.0;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::S) {
            self.camera.1 -= 1.0;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::D) {
            self.camera.0 -= 1.0;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::PageUp) {
            self.scale_constant += 1.0;
            self.scale = self.scale_constant / nbody::AU;
            self.trajectories = vec![vec![]; self.bodies.len()];
        }
        if keyboard::is_key_pressed(ctx, KeyCode::PageDown) {
            if self.scale_constant > 2.0 {
                self.scale_constant -= 1.0;
                self.scale = self.scale_constant / nbody::AU;
                self.trajectories = vec![vec![]; self.bodies.len()];
            }
        }

        while timer::check_update_time(ctx, self.desired_fps) { 
            if self.is_running {
                self.tick();
            }
        }

        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 0.0].into());
        let (width, height) = graphics::drawable_size(&ctx);
        let (sx, sy) = scale_pos(&self.bodies[0], self.scale); // the suns position is the center, when the camera is not moved
        let (offsetx, offsety) = ((width / 2.0) - sx as f32 + self.camera.0, 
                                  (height / 2.0) - sy as f32 + self.camera.1);

        if self.draw_trajectory && self.trajectories[0].len() > 2 {
            for i in 1..self.trajectories.len() {
                let trajectory = graphics::Mesh::new_line(
                    ctx, 
                    &self.trajectories[i],
                    1.0, 
                    self.bodies[i].color)?;
                graphics::draw(ctx, &trajectory, (na::Point2::new(offsetx, offsety),))?;
            }

        }

        for i in 0..self.bodies.len() {
            let g_body = graphics::Mesh::new_circle(
                ctx, 
                graphics::DrawMode::fill(), 
                na::Point2::new(0.0,0.0), 
                self.bodies[i].radius, 
                0.02, 
                self.bodies[i].color)?;
            let (x, y) = scale_pos(&self.bodies[i], self.scale);
            let p_body = na::Point2::new(x as f32 + offsetx, y as f32 + offsety);
            
            graphics::draw(ctx, &g_body, (p_body,))?;
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    // get input
    let file = File::open("data/planets.txt")?;
    let mut lines = io::BufReader::new(file).lines();
    let n_bodies = usize::from_str(&lines.next().unwrap().unwrap()).unwrap();
    let _world_radius = f64::from_str(&lines.next().unwrap().unwrap()).unwrap();
    let mut bodies: Vec<Body> = vec![];
    
    for _ in 0..n_bodies {
        let tmp = lines.next().unwrap().unwrap();
        let splitted: Vec<&str> = tmp.split_whitespace().collect();
        let pos = (f64::from_str(&splitted[0]).unwrap(), f64::from_str(&splitted[1]).unwrap());
        let vel = (f64::from_str(&splitted[2]).unwrap(), f64::from_str(&splitted[3]).unwrap());
        let m = f64::from_str(&splitted[4]).unwrap();
        let n = String::from(splitted[5]);
        let color: Vec<u8> = splitted[6].split(",")
                                    .map(|x| u8::from_str(x).unwrap())
                                    .collect();
        let c = graphics::Color::from_rgb(color[0], color[1], color[2]);
        let r = f32::from_str(&splitted[7]).unwrap();
        let body = Body::new(pos, vel, m, c, n, r);
        bodies.push(body);
    }

    // run simulation
    let draw_trajectory = true;
    let (width, height) = (1500.0, 820.0);
    let cb = ggez::ContextBuilder::new("super_simple", "yaya")
                .window_mode(WindowMode {
                    width: width,
                    height: height,
                    resizable: false,
                    ..WindowMode::default()
                });
    let (ctx, event_loop) = &mut cb.build()?;
    graphics::set_window_title(ctx, "nbody");
    let state = &mut MainState::new(bodies, 25000., draw_trajectory)?;
    event::run(ctx, event_loop, state)

}
