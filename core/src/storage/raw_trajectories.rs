/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// raw_trajectories.rs — angle-bucketed trajectory index for neighbor lookup

use super::super::geometry::trajectory::Trajectory;
use crate::utils::data_types::angle_u16::{AngleU16, FULL_CIRCLE};

const BUCKET_SIZE: f64 = 1.0; // degrees, must evenly divide 360.0

pub struct Bucket {
    pub angle_start: AngleU16, // inclusive
    pub angle_end: AngleU16,   // exclusive
    pub trajectories: Vec<Trajectory>,
}

pub struct RawTrajectories {
    pub bucket_size: AngleU16,
    pub max_angle: AngleU16,
    pub traj_buckets: Vec<Bucket>,
}

impl RawTrajectories {
    pub fn new(max_angle: AngleU16) -> Self {
        let bucket_size: AngleU16 = AngleU16::from_degrees(BUCKET_SIZE);
        let buckets: Vec<Bucket> = Self::create_buckets(bucket_size);
        Self {
            bucket_size,
            max_angle,
            traj_buckets: buckets,
        }
    }

    fn create_buckets(bucket_size: AngleU16) -> Vec<Bucket> {
        let bucket_raw: u16 = bucket_size.raw();

        assert!(
            FULL_CIRCLE.is_multiple_of(bucket_raw),
            "Bucket size must evenly divide 360°"
        );

        let num_buckets: usize = (FULL_CIRCLE / bucket_raw) as usize;
        let mut buckets: Vec<Bucket> = Vec::with_capacity(num_buckets);

        for i in 0..num_buckets {
            let start_raw: u16 = i as u16 * bucket_raw;
            let mut end_raw: u16 = start_raw + bucket_raw;

            end_raw = end_raw.min(FULL_CIRCLE);

            buckets.push(Bucket {
                angle_start: AngleU16::from_u16(start_raw),
                angle_end: AngleU16::from_u16(end_raw),
                trajectories: Vec::new(),
            });
        }

        buckets
    }

    #[inline]
    fn angle_to_bucket(&self, angle: AngleU16) -> usize {
        (angle.raw() / self.bucket_size.raw()) as usize
    }

    pub fn add_trajectory(&mut self, traj: Trajectory) {
        let bucket_idx: usize = self.angle_to_bucket(traj.angle);

        if let Some(bucket) = self.traj_buckets.get_mut(bucket_idx) {
            bucket.trajectories.push(traj);
        } else {
            panic!("Bucket index {bucket_idx} does not exist");
        }
    }

    // Returns a copy of trajectories from all buckets within max_angle
    pub fn vec_nearby_angle(&self, angle: AngleU16) -> Vec<Trajectory> {
        let idx: usize = self.angle_to_bucket(angle);

        let u_len: usize = self.traj_buckets.len();
        let i_len: isize = u_len as isize;
        let wrap = |i: isize| -> usize { ((i % i_len) + i_len) as usize % u_len };

        // Number of neighboring buckets required on each side to fully cover max_angle
        let bucket_radius: isize = self.max_angle.raw().div_ceil(self.bucket_size.raw()) as isize;

        // estimate size to avoid reallocations
        let mut total: usize = 0;
        for offset in -bucket_radius..=bucket_radius {
            let i: usize = wrap(idx as isize + offset);
            total += self.traj_buckets[i].trajectories.len();
        }

        let mut result: Vec<Trajectory> = Vec::with_capacity(total);

        for offset in -bucket_radius..=bucket_radius {
            let i: usize = wrap(idx as isize + offset);
            result.extend(self.traj_buckets[i].trajectories.iter().cloned());
        }

        result
    }

    pub fn get_num_trajectories(&self) -> usize {
        self.traj_buckets.iter().map(|b| b.trajectories.len()).sum()
    }

    #[allow(unused)]
    pub fn print_info(&self) {
        for (i, bucket) in self.traj_buckets.iter().enumerate() {
            println!(
                "Bucket {}: Angle [{:.2}°, {:.2}°[ - {} trajectories",
                i,
                bucket.angle_start,
                bucket.angle_end,
                bucket.trajectories.len()
            );

            for traj in &bucket.trajectories {
                println!("  {}", traj.print_info());
            }
        }
    }
}
