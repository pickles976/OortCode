// Tutorial: Radar
// Destroy the enemy ships. Use your radar to find them.
// Hint: Press 'g' in-game to show where your radar is looking.
// Hint: Press 'n' to single-step.
// Hint: Use the set_radar_heading() function to keep your radar pointed at a
// target, or to search for a new one.
//
// Join the Discord at https://discord.gg/vYyu9EhkKH for Oort discussion and
// tournament results.
use oort_api::prelude::*;

const BULLET_SPEED: f64 = 1000.0; // m/s



pub struct Ship {
    tracking: bool,
    current_heading: f64,
    prev_target_pos: Vec2,
    prev_target_vel: Vec2,
    prev_target_vel_norm: Vec2,
    prev_target_accel: Vec2,
    prev_target_jerk: Vec2,
    prev_angle: f64,
    prev_angle_diff: f64,
    pred_target_accel: Vec2,
    pred_target_vel: Vec2
}

impl Ship {
    pub fn new() -> Ship {
        Ship {
            tracking: false,
            current_heading: 0.0,
            prev_target_pos: Vec2::new(0.0, 0.0),
            prev_target_vel: Vec2::new(0.0, 0.0),
            prev_target_vel_norm: Vec2::new(0.0, 0.0),
            prev_target_accel: Vec2::new(0.0, 0.0),
            prev_target_jerk: Vec2::new(0.0, 0.0),
            prev_angle: 0.0,
            prev_angle_diff: 0.0,
            pred_target_accel: Vec2::new(0.0, 0.0),
            pred_target_vel: Vec2::new(0.0, 0.0)
        }
    }

    pub fn predict_time_of_flight(&mut self, dist: Vec2, t_vel: Vec2) -> f64 {
        // Predict time of flight of a bullet from a player

        let x = dist[0];
        let y = dist[1];
        let dv = t_vel;
        let h = dv[0];
        let v = dv[1];
        let bs2 = BULLET_SPEED * BULLET_SPEED;

        let a = h*h + v*v - bs2;
        let b = 2.0 * (h*x + v*y);
        let c = y*y + x*x;

        let mut root_part = b*b - (4.0 * a * c);
        root_part = root_part.sqrt();

        let plus = (-b + root_part) / (2.0 * a);
        let minus = (-b - root_part) / (2.0 * a);

        let mut flight_time = 0.0;
        if plus > 0.0 {
            flight_time = plus;
        } else if minus > 0.0 {
            flight_time = minus;
        }

        flight_time
    }

    pub fn pid(&mut self, angle: f64) -> f64 {
        let k_p = 60.0;
        let k_d = 50000.0;

        let d_err = angle - self.prev_angle_diff;
        self.prev_angle_diff = angle;

        (k_p * angle) + (k_d * d_err * TICK_LENGTH)
    }

    pub fn process_contact(&mut self, contact: ScanResult) {

        let vel = contact.position - self.prev_target_pos;
        let norm_vel = vel - velocity();
        let accel = vel - self.prev_target_vel;
        let jerk = accel - self.prev_target_accel;

        self.prev_target_pos = contact.position;
        self.prev_target_vel = vel;
        self.prev_target_vel_norm = norm_vel;
        self.prev_target_accel = accel;
        self.prev_target_jerk = jerk;

        // debug!("Contact! {:?}", contact.position);
        // debug!("Velocity: {}", vel);

        // let error = self.prev_target_accel - self.pred_target_accel;
        // debug!("Enemy Accel: {}", self.prev_target_accel);
        // debug!("Predicted Accel: {}", self.pred_target_accel);
        // debug!("Error %: {}", error.length() / self.prev_target_accel.length());

        self.pred_target_accel = accel + jerk * TICK_LENGTH;

        // let error = self.prev_target_accel - self.pred_target_accel;
        // debug!("Enemy Vel: {}", self.prev_target_vel);
        // debug!("Predicted Vel: {}", self.pred_target_vel);
        // debug!("Error %: {}", error.length() / self.prev_target_vel.length());

        self.pred_target_vel = vel + self.pred_target_accel * TICK_LENGTH;

        let flight_time = self.predict_time_of_flight(contact.position - position(), self.pred_target_vel - velocity());

        let lead_target = contact.position + (self.pred_target_vel / TICK_LENGTH) * flight_time;
        draw_line(position(), lead_target, 0x0000ff);

        // Adjust for acceleration once
        let accel_corrected = lead_target + 0.5 * (self.pred_target_accel / TICK_LENGTH) * flight_time * flight_time;
        draw_line(position(), accel_corrected, 0xffff44);

        // Calculate desired angle, and angular diff
        let desired_angle = (accel_corrected - position()).angle();
        let angle = angle_diff(heading(), desired_angle);

        //Add angular velocity to shoot ahead
        let angular_vel = (desired_angle - self.prev_angle) * TICK_LENGTH;
        self.prev_angle = desired_angle;

        torque(self.pid(angle + angular_vel));

        if angle.abs() < 0.075
        {
            fire(0);
        }

    }

    pub fn scan_mode(&mut self) {
        self.current_heading -= PI / 2.0;
        set_radar_width(PI / 2.0);
    }

    pub fn track_mode(&mut self) {
        let predicted_pos = self.prev_target_pos;
        let desired_angle = (predicted_pos - position()).angle();
        self.current_heading = desired_angle;
        set_radar_width(PI / 20.0);
    }

    pub fn tick(&mut self) {        

        // On contact
        if let Some(contact) = scan() { 
            
            self.tracking = true;
            self.process_contact(contact);

        } else {
            self.tracking = false;
        }

        // Needs to be set before the next turn
        if self.tracking {
            self.track_mode();
        } else {
            self.scan_mode();
        }

        debug!("Tracking: {}", self.tracking);

        set_radar_heading(self.current_heading);

        // predicted track
        draw_line(position(), self.prev_target_pos + self.prev_target_vel + self.prev_target_accel * TICK_LENGTH, 0xffffff);
    }

}
