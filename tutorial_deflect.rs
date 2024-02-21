// Tutorial: Deflection
// Destroy the enemy ship. Its position is given by the "target" function and velocity by the
// "target_velocity" function.
//
// Hint: p = p₀ + v₀t + ½at² (the third equation of kinematics)
//
// p.s. You can change your username by clicking on it at the top of the page.
use oort_api::prelude::*;

const BULLET_SPEED: f64 = 1000.0; // m/s

pub struct Ship {
    prev_target_velocity: Vec2,
    prev_target_accel: Vec2,
    pred_target_accel: Vec2,
    prev_angle: f64,
    prev_angle_diff: f64
}

impl Ship {
    pub fn new() -> Ship {
        Ship{
            prev_target_velocity: Vec2::new(0.0, 0.0),
            prev_target_accel: Vec2::new(0.0, 0.0),
            pred_target_accel: Vec2::new(0.0, 0.0),
            prev_angle: 0.0,
            prev_angle_diff: 0.0
        }
    }

    pub fn predict_time_of_flight(&mut self, t_vel: Vec2) -> f64 {

        let dp = target() - position();
        let x = dp[0];
        let y = dp[1];
        let dv = target_velocity() - velocity();
        let h = dv[0];
        let v = dv[1];
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

        let d_err = angle - self.prev_angle_diff;
        self.prev_angle_diff = angle;

        (k_p * angle) + (k_d * d_err * TICK_LENGTH)
    }

    pub fn tick(&mut self) {

        // Movement
        let seek = target_velocity() - velocity();
        accelerate(seek);

        let approach = target() - position();
        accelerate(approach / 25.0);

        // Calculate accel, jerk and predict next accel
        let norm_target_velocity = target_velocity() - velocity();
        let d_vel = (norm_target_velocity - self.prev_target_velocity);
        self.prev_target_velocity = norm_target_velocity;

        let jerk = (self.prev_target_accel - d_vel) * TICK_LENGTH;

        let error = d_vel - self.pred_target_accel;
        debug!("Enemy Accel: {}", d_vel);
        debug!("Predicted Accel: {}", self.pred_target_accel);
        debug!("Error %: {}", error.length() / d_vel.length());

        self.pred_target_accel = d_vel + jerk;

        // predict time of flight naively
        let flight_time = self.predict_time_of_flight(norm_target_velocity);

        debug!("Intercept Time: {}", flight_time);

        // Get lead target position
        let lead_target = target() + (norm_target_velocity * flight_time);

        draw_line(position(), lead_target, 0xffffff);

        // Adjust for acceleration once
        let accel_corrected = lead_target + 0.5 * (self.pred_target_accel / TICK_LENGTH) * flight_time * flight_time;

        draw_line(position(), accel_corrected, 0xffff44);

        // Calculate desired angle, and angular diff
        let desired_angle = (accel_corrected - position()).angle();
        let angle = angle_diff(heading(), desired_angle);

        //Add angular velocity to shoot ahead
        let angular_vel = (desired_angle - self.prev_angle) * TICK_LENGTH;
        self.prev_angle = desired_angle;

        debug!("Angular velocity: {}", angular_vel);

        torque(self.pid(angle + angular_vel));

        if angle.abs() < 0.015
        {
            fire(0);
        }

    }
}
