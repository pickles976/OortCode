// Tutorial: Lead
// Destroy the enemy ship. Its position is given by the "target" function and velocity by the
// "target_velocity" function. Your ship is not able to accelerate in this scenario.
//
// This is where the game becomes challenging! You'll need to lead the target
// by firing towards where the target will be by the time the bullet gets there.
//
// Hint: target() + target_velocity() * t gives the position of the target after t seconds.
//
// You can scale a vector by a number: vec2(a, b) * c == vec2(a * c, b * c)
//
// p.s. You can change your username by clicking on it at the top of the page.
use oort_api::prelude::*;

const BULLET_SPEED: f64 = 1000.0; // m/s

// TODO: 

pub struct Ship {
    prev_angle: f64
}

impl Ship {
    pub fn new() -> Ship {
        Ship{
            prev_angle: 0.0
        }
    }

    pub fn flight_time(&mut self) -> f64 {
        // Values: 
        // t = travel time (what we are trying to find)
        // T0 = target @ time 0
        // P0 = player @ time 0
        // V = target velocity @ time 0
        // P = BS = Bullet Speed
        // 
        // dp = T0 - P0
        // x = dp.x
        // y = dp.y
        // h = V.x
        // v = V.y
        //
        // t = d / P, t*P = d
        // d = magnitude(P0 - T0)
        // t*P = magnitude(P0 - T0)
        // t*P = sqrt((x+ht)**2 + (y+vt)**2)
        // t**2 * P**2 = (x+ht)**2 + (y+vt)**2
        // Expand right side with quadratic formula
        // Solving quadtratic returns two zeros. One zero will be positive,
        // a viable intercept.

        let dp = target() - position();
        let x = dp[0];
        let y = dp[1];
        let h = target_velocity()[0];
        let v = target_velocity()[1];
        let bs2 = BULLET_SPEED * BULLET_SPEED;

        let a = h*h + v*v - bs2;
        let b = 2.0 * (h*x + v*y);
        let c = y*y + x*x;

        let mut sqr = b*b - (4.0 * a * c);
        sqr = sqr.sqrt();

        let plus = (-b + sqr) / (2.0 * a);
        let minus = (-b - sqr) / (2.0 * a);

        // debug!("Quadratic zeros: {} {}", plus, minus);

        let mut flight_time = 0.0;
        if plus > 0.0 {
            flight_time = plus;
        } else if minus > 0.0 {
            flight_time = minus;
        }

        flight_time
    }

    pub fn pid(&mut self, angle: f64) -> f64 {
        let k_p = 50.0;
        let k_d = 50000.0;

        let d_err = angle - self.prev_angle;
        self.prev_angle = angle;

        debug!("Change in error: {}", d_err);

        (k_p * angle) + (k_d * d_err * TICK_LENGTH)
    }

    pub fn tick(&mut self) {

        draw_line(position(), position() + Vec2::new(1000.0 * heading().cos(), 1000.0 * heading().sin()), 0x0000ff);

        // Calculate the travel time of the bullet
        let flight_time = self.flight_time();

        // debug!("Time to target: {}", flight_time);
        let lead_target = target() + (target_velocity() * flight_time);
        draw_line(position(), lead_target, 0x00ff00);

        // Calculate desired angle, and angular diff
        let desired_angle = (lead_target - position()).angle();
        let angle = angle_diff(heading(), desired_angle);
        debug!("Delta Angle: {}", angle);

        // turn(angle*50.0);
        torque(self.pid(angle));

        if angle.abs() < 0.025 
        {
            fire(0);
        }

    }
}
