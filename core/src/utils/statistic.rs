/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// statistic.rs — clustering statistics and directional correlation helpers

use crate::storage::clustered_trajectories::ClusteredTrajectories;
use crate::storage::raw_trajectories::RawTrajectories;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::collections::HashMap;

// Computes a directional correlation factor in [0.0, 1.0]
// Adjacent buckets contribute to the same score so near-concentrated flows still count as directional
pub fn directional_correlation(raw: &RawTrajectories) -> f64 {
    let total: usize = raw.get_num_trajectories();

    // Edge cases
    if total == 0 {
        return 0.0;
    }

    let n: usize = raw.traj_buckets.len();
    if n == 0 {
        return 0.0;
    }

    // Only one bucket → always 1.0
    if n == 1 {
        return 1.0;
    }

    // Step 1: raw share per bucket (fraction of total trajectories)
    let shares: Vec<f64> = raw
        .traj_buckets
        .iter()
        .map(|b| b.trajectories.len() as f64 / total as f64)
        .collect();

    // Step 2: adjacency-smoothed share
    // Each bucket contributes 50% of its share to itself and 25% to each neighbor.
    // This ensures two adjacent full buckets score near 1.0, not 0.5.
    let smoothed: Vec<f64> = (0..n)
        .map(|i| {
            let prev = (i + n - 1) % n;
            let next = (i + 1) % n;
            shares[i] * 0.50 + shares[prev] * 0.25 + shares[next] * 0.25
        })
        .collect();

    // Step 3: HHI = sum of squared shares
    let hhi: f64 = smoothed.iter().map(|s| s * s).sum();

    // Step 4: normalize to [0, 1]
    // HHI_min (uniform) = 1/n,  HHI_max (all in one) = 1.0
    // After smoothing, HHI_max is slightly below 1.0 (0.5^2 + 2*0.25^2 = 0.375 for n>=3)
    // So we normalize against the actual theoretical min/max.
    let hhi_min = 1.0 / n as f64; // perfectly uniform
    let hhi_max = 0.50_f64.powi(2)       // one bucket fully loaded after smoothing
            + 2.0 * 0.25_f64.powi(2); // = 0.375 for n >= 3

    if (hhi_max - hhi_min).abs() < f64::EPSILON {
        return 0.0;
    }

    ((hhi - hhi_min) / (hhi_max - hhi_min)).clamp(0.0, 1.0)
}

// Per-trajectory segment counts: (segments clustered into a corridor, segments left unclustered).
type TrajSegmentCounts = HashMap<usize, (u32, u32)>;

// Histogram of trajectories by number of clustered segments, plus summary stats.
pub struct ClusteringHistogram {
    // `histogram[k]` = number of trajectories with exactly `k` clustered segments.
    pub histogram: Vec<usize>,
    // Trajectories with at least one clustered segment, out of `total_trajectories`.
    pub num_traj_with_clustered_segment: usize,
    // Total number of distinct trajectories observed.
    pub total_trajectories: usize,
    // Highest number of clustered segments found for a single trajectory.
    pub max_clustered_segments: u32,
}

impl ClusteringHistogram {
    #[allow(dead_code)]
    pub fn get_summary(&self) -> Vec<String> {
        let mut output: Vec<String> = Vec::new();
        output.push("=== Clustered Trajectories Summary ===".to_string());
        output.push(format!(
            "- Trajectories with at least one clustered segment: {} ({:.1}%)",
            self.num_traj_with_clustered_segment,
            100.0 * self.num_traj_with_clustered_segment as f64 / self.total_trajectories as f64
        ));
        let mut histogram_str: String = "".to_string();
        histogram_str.push_str("- Segment clustered distribution: \n [nb clustered segments per trajectory: nb of trajectories having that number]\n ");
        for (i, &count) in self.histogram.iter().enumerate() {
            if count > 0 {
                histogram_str.push_str(&format!("[{}:{}] ", i, count));
            }
        }
        output.push(histogram_str + "\n");
        output
    }
}

// Builds a per-trajectory clustered/non-clustered segment count map in parallel
fn count_segments_per_trajectory(clust_storage: &ClusteredTrajectories) -> TrajSegmentCounts {
    clust_storage
        .get_all_cluster_members_iter()
        .par_bridge()
        .fold(
            HashMap::new,
            |mut acc: TrajSegmentCounts, (corridor_idx, cm)| {
                let entry = acc.entry(cm.traj_id).or_insert((0, 0));
                if corridor_idx >= 0 {
                    entry.0 += 1; // clustered segment
                } else {
                    entry.1 += 1; // non-clustered segment
                }
                acc
            },
        )
        .reduce(HashMap::new, |mut a, b| {
            for (traj_id, (clustered, non_clustered)) in b {
                let entry = a.entry(traj_id).or_insert((0, 0));
                entry.0 += clustered;
                entry.1 += non_clustered;
            }
            a
        })
}

// Computes the clustering histogram and summary stats for a clustering result
// Buckets trajectories by how many of their segments ended up in a corridor
pub fn clustering_histogram(clust_storage: &ClusteredTrajectories) -> ClusteringHistogram {
    let counts: TrajSegmentCounts = count_segments_per_trajectory(clust_storage);

    let max_clustered_segments: u32 = counts
        .values()
        .map(|(clustered, _)| *clustered)
        .max()
        .unwrap_or(0);

    let mut histogram: Vec<usize> = vec![0; max_clustered_segments as usize + 1];
    let mut num_traj_with_clustered_segment: usize = 0;

    for (clustered, _) in counts.values() {
        histogram[*clustered as usize] += 1;
        if *clustered > 0 {
            num_traj_with_clustered_segment += 1;
        }
    }

    ClusteringHistogram {
        histogram,
        num_traj_with_clustered_segment,
        total_trajectories: counts.len(),
        max_clustered_segments,
    }
}
