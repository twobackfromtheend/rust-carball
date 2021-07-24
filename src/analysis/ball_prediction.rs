use crate::actor_handlers::TimeSeriesBallData;
use chip::Ball;
use lazy_static::lazy_static;
use nalgebra::{Point3, Vector3};
use std::sync::Mutex;

static PHYSICS_TICK_DELTA: f32 = 1.0 / 120.0;

lazy_static! {
    static ref BALL: Mutex<Ball> = Mutex::new(Ball::new());
}

pub fn predict_ball_bounce(
    ball_data: &TimeSeriesBallData,
    delta: f32,
) -> Result<bool, BallPredictionError> {
    let mut ball = BALL.lock().map_err(|_| BallPredictionError::LockError)?;
    ball.set_pos(Point3::new(
        ball_data.pos_x.ok_or(BallPredictionError::MissingData)?,
        ball_data.pos_y.ok_or(BallPredictionError::MissingData)?,
        ball_data.pos_z.ok_or(BallPredictionError::MissingData)?,
    ));
    ball.set_vel(Vector3::new(
        ball_data.vel_x.unwrap_or(0.0),
        ball_data.vel_y.unwrap_or(0.0),
        ball_data.vel_z.unwrap_or(0.0),
    ));
    let initial_omega = Vector3::new(
        ball_data.ang_vel_x.unwrap_or(0.0),
        ball_data.ang_vel_y.unwrap_or(0.0),
        ball_data.ang_vel_z.unwrap_or(0.0),
    );
    ball.set_omega(initial_omega);
    let mut simulation_elapsed: f32 = 0.0;
    loop {
        if delta - simulation_elapsed > PHYSICS_TICK_DELTA {
            ball.step(PHYSICS_TICK_DELTA);
            simulation_elapsed += PHYSICS_TICK_DELTA;
        } else {
            // More than half the duration of a physics tick remaining, it's likely there was an additional physics tick performed.
            // I have not observed this being in effect, as the replay's frame delta tends to hover slightly above 30Hz.
            if delta - simulation_elapsed > PHYSICS_TICK_DELTA / 2.0 {
                ball.step(PHYSICS_TICK_DELTA);
            }
            break;
        }
    }
    Ok(ball.omega() != initial_omega)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BallPredictionError {
    LockError,
    MissingData,
}
