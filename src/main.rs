pub mod nbody;
use nbody::Body;
use std::io::{self, BufRead};
use std::str::{FromStr};
use std::fs::File;

use ggez;
use ggez::timer;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra as na;
use ggez::{Context, GameResult};
use ggez::conf::WindowMode;

struct MainState {
    pub bodies: Vec<Body>,
    dt: f64,
    pub trajectories: Vec<Vec<na::Point2<f32>>>,
    pub time_to_save: u8,
    pub draw_trajectory: bool,
}

impl MainState {
    fn new(bodies: Vec<Body>, dt: f64, draw_trajectory: bool) -> GameResult<MainState> {
        let traj = vec![vec![]; bodies.len()];
        let s = MainState {
            bodies: bodies, 
            dt: dt, 
            trajectories: traj, 
            time_to_save: 1,
            draw_trajectory: draw_trajectory
        };
        Ok(s)
    }

    fn tick(&mut self) {
        nbody::move_all_bodies(&mut self.bodies, self.dt);
        if self.draw_trajectory {
            self.time_to_save -= 1;
            if self.time_to_save == 0 {
                for i in 0..self.bodies.len() {
                    let (x,y) = self.bodies[i].scale_pos;
                    let p = na::Point2::new(x as f32, y as f32);
                    self.trajectories[i].push(p);
                }
                self.time_to_save = 10;
            }
        }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 900;

        while timer::check_update_time(ctx, DESIRED_FPS) { 
            self.tick();
        }

        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 0.0].into());
        let (width, height) = graphics::drawable_size(&ctx);
        let (offsetx, offsety) = ((width / 2.0) - self.bodies[0].scale_pos.0 as f32, 
                                  (height / 2.0) - self.bodies[0].scale_pos.1 as f32);

        if self.draw_trajectory && self.trajectories[0].len() > 2 {
            for i in 0..self.trajectories.len() {
                if i == 20 {
                    let g_body = graphics::Mesh::new_circle(
                        ctx, 
                        graphics::DrawMode::fill(), 
                        na::Point2::new(0.0,0.0), 
                        self.bodies[i].radius, 
                        0.02, 
                        self.bodies[i].color)?;
                    let (x, y) = self.bodies[i].scale_pos;
                    let p_body = na::Point2::new(x as f32 + offsetx, y as f32 + offsety);
                    graphics::draw(ctx, &g_body, (p_body,))?;
                } else {
                    let trajectory = graphics::Mesh::new_line(
                        ctx, 
                        &self.trajectories[i],
                        1.0, 
                        self.bodies[i].color)?;
                    graphics::draw(ctx, &trajectory, (na::Point2::new(offsetx, offsety),))?;
                }
            }

        }

        if true { // !self.draw_trajectory {
            for i in 0..self.bodies.len() {
                let g_body = graphics::Mesh::new_circle(
                    ctx, 
                    graphics::DrawMode::fill(), 
                    na::Point2::new(0.0,0.0), 
                    self.bodies[i].radius, 
                    0.02, 
                    self.bodies[i].color)?;
                let (x, y) = self.bodies[i].scale_pos;
                let p_body = na::Point2::new(x as f32 + offsetx, y as f32 + offsety);
                
                graphics::draw(ctx, &g_body, (p_body,))?;
            }
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
