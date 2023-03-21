extern crate crossbeam_channel;

use crate::config::DonationConfig;
use crate::worker::worker_pool::WorkerConfig;

use self::crossbeam_channel::{unbounded, Receiver};
use std;
use std::thread;
use std::time::Duration;

pub const DONATION_THRESHOLD: f64 = 1.0 / 10.0;

#[derive(Debug, PartialEq)]
pub enum TickAction {
    ArmChange,
    DonationHashing,
}

pub fn interval_mod_setup(
    autotune_interval: Option<u64>,
    donation_percentage: f64,
) -> (u64, Option<u64>) {
    todo!("Replace the entire control infrastucture")
}

/// clock for bandit arm change and donation
pub fn setup(worker_conf: &WorkerConfig, donation_conf: &DonationConfig) -> Receiver<TickAction> {
    let (clock_sndr, clock_rcvr) = unbounded();

    let (reg_interval, donation_mod) = interval_mod_setup(
        worker_conf.autotune.as_ref().map(|a| a.interval_minutes),
        donation_conf.0,
    );
    let mut interval = reg_interval;

    let donation_percentage = donation_conf.0;
    //if auto_tune is not enabled, never send the clock signal for drawing
    //a new arm, effectively disabling auto tuning
    thread::Builder::new()
        .name("clock signal thread".to_string())
        .spawn(move || {
            let mut arm_changes = 1;
            loop {
                thread::sleep(Duration::from_secs(interval));

                let action = if let Some(d_mod) = donation_mod {
                    if arm_changes % d_mod == 0 {
                        TickAction::DonationHashing
                    } else {
                        TickAction::ArmChange
                    }
                } else {
                    TickAction::ArmChange
                };

                interval = if action == TickAction::DonationHashing {
                    (donation_percentage * 60.0).ceil() as u64
                } else {
                    reg_interval
                };

                clock_sndr.send(action).expect("sending clock signal");
                arm_changes += 1;
            }
        })
        .expect("clock signal thread handle");

    clock_rcvr
}
