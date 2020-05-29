use ggez::graphics;

const AU: f64 = 1.4960e+11; // in meters, roughly distance earth -> sun
const SCALE: f64 = 20. / AU;
const GRAVITY: f64 = 6.67428e-11;

type Vel = (f64,f64);
type Pos = (f64,f64);
type Mass = f64;
type Accel = (f64,f64);
type Dist = (f64,f64);
type Force = (f64,f64);

#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    pos: Pos,
    vel: Vel,
    mass: Mass,
    pub color: graphics::Color,
    pub name: String,
    pub scale_pos: Pos,
    pub radius: f32,
}

impl Body {
    pub fn new(p: Pos, v: Vel, m: Mass, c: graphics::Color, n: String, r: f32) -> Body {
        Body {pos: p, vel: v, mass: m, color: c, name: n, radius: r, scale_pos: (p.0 * SCALE, p.1 * SCALE),}
    }

    fn get_distance(&self, other: &Pos) -> Dist {
        let dx = other.0 - self.pos.0;
        let dy = other.1 - self.pos.1;
        (dx,dy)
    }

    fn get_force(&self, other: &Body) -> Force {
        let dist = self.get_distance(&other.pos);
        if dist.0 == 0.0 && dist.1 == 0.0 {
            return (0.0, 0.0);
        }
        let r = (dist.0.powi(2) + dist.1.powi(2)).sqrt();
        let cos = dist.0 / r;
        let sin = dist.1 / r;
        let f = (GRAVITY * self.mass * other.mass) / r.powi(2);
        let fx = f * cos;
        let fy = f * sin;
        (fx, fy)
    }

    fn move_body(&mut self, a: Accel, dt: f64) {
        self.vel = (self.vel.0 + dt * a.0, self.vel.1 + dt * a.1);
        self.pos = (self.pos.0 + dt * self.vel.0, self.pos.1 + dt * self.vel.1);
        self.scale_pos = (self.pos.0 * SCALE, self.pos.1 * SCALE);
    }
}

fn calculate_forces(bodies: &Vec<Body>) -> Vec<Force> {
    let mut forces = vec![(0.,0.); bodies.len()];
    for i in 0..bodies.len() - 1 {
        for j in i + 1..bodies.len() {
            let f = bodies[i].get_force(&bodies[j]);
            forces[i].0 += f.0;
            forces[i].1 += f.1;
            forces[j].0 += -f.0;
            forces[j].1 += -f.1;
        }
    }
    forces
}

fn calculate_accels(bodies: &Vec<Body>, forces: Vec<Force>) -> Vec<Accel> {
    let mut accels = vec![(0.,0.); bodies.len()];
    for i in 0..bodies.len() {
        let ax = forces[i].0 / bodies[i].mass;
        let ay = forces[i].1 / bodies[i].mass;
        accels[i] = (ax,ay);
    }
    accels
}

// uses leapfrog scheme
pub fn move_all_bodies(bodies: &mut Vec<Body>, dt: f64) {
    let forces = calculate_forces(bodies);
    let accels = calculate_accels(bodies, forces);
    for i in 1..bodies.len() {
        bodies[i].move_body(accels[i], dt);
    }
}