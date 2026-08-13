/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// parallel_rayon_traclusdl.rs — Rayon-parallel TraClus over trajectories and segments

use std::slice;

use super::base_traclusdl::TraclusAlgorithm;
use crate::geometry::segment::Segment;
use crate::geometry::trajectory::Trajectory;
use crate::objects::cluster::Cluster;
use crate::storage::{
    clustered_trajectories::ClusteredTrajectories,
    raw_trajectories::{Bucket, RawTrajectories},
};
use crate::ui::args::TraclusArgs;
use crate::utils::parallel_runner::StopFlag;

use rayon::prelude::*;
use rayon::slice::Iter;

pub struct ParallelRayonTraclusDL {
    args: TraclusArgs,
    stop_flag: Option<StopFlag>,
}

impl ParallelRayonTraclusDL {
    pub fn new(args: TraclusArgs) -> Self {
        Self {
            args,
            stop_flag: None,
        }
    }

    // Serial over buckets; parallel over trajectories within each bucket
    fn complete_parallel_clustering(
        &self,
        raw_trajectories: &RawTrajectories,
    ) -> Vec<Vec<Cluster>> {
        // Serial iterator over angle buckets
        let bucket_serial_iter: slice::Iter<'_, Bucket> = raw_trajectories.traj_buckets.iter();
        let mut results: Vec<Vec<Cluster>> = Vec::new();

        for bucket in bucket_serial_iter {
            // Shared read-only nearby copy per bucket
            let nearby_trajs: Vec<Trajectory> =
                raw_trajectories.vec_nearby_angle(bucket.angle_start);

            // Parallelize over trajectories in this bucket using Rayon
            let traj_parallel_iter: Iter<'_, Trajectory> = bucket.trajectories.par_iter();
            let bucket_results: Vec<Vec<Cluster>> = traj_parallel_iter
                .map(|traj_seed: &Trajectory| {
                    // Each thread checks the stop flag before proceeding with clustering
                    if self.is_stopped() {
                        return Vec::new();
                    }

                    let clusters: Vec<Cluster> =
                        self.individual_trajectory_clustering(traj_seed, &nearby_trajs);

                    clusters
                })
                .collect::<Vec<_>>();

            // Stop early if requested
            if self.is_stopped() {
                break;
            }

            // Commit the results for this bucket
            results.extend(bucket_results);

            // Tick after every bucket (count = actual number of trajectories in this bucket)
            self.tick_clustering(bucket.trajectories.len());
        }
        results
    }

    // Clusters one trajectory against nearby trajectories by expanding each reachable segment
    fn individual_trajectory_clustering(
        &self,
        traj_seed: &Trajectory,
        nearby_trajs: &[Trajectory],
    ) -> Vec<Cluster> {
        // Parallelize over segments from this Trajectory using Rayon
        let traj_parallel_iter = traj_seed.segments_par_iter();

        let traj_results: Vec<Cluster> = traj_parallel_iter
            .filter_map(|seed_segment: &Segment| {
                // Try to form an initial cluster from this seed segment
                let cluster: Option<Cluster> =
                    self.initial_segment_cluster((seed_segment, traj_seed), nearby_trajs);

                if let Some(mut cluster) = cluster {
                    // Expand the cluster to include all density-reachable segments
                    self.expand_segment_cluster(&mut cluster, nearby_trajs);
                    Some(cluster)
                } else {
                    // If no cluster forms, continue to next segment (not dense enough)
                    None
                }
            })
            .collect();

        traj_results
    }
}

impl TraclusAlgorithm for ParallelRayonTraclusDL {
    // ============================================================
    // Shared Data Accessors
    // ============================================================
    fn args(&self) -> &TraclusArgs {
        &self.args
    }

    fn stop_flag(&self) -> &Option<StopFlag> {
        &self.stop_flag
    }

    fn set_stop_flag(&mut self, stop_flag: StopFlag) {
        self.stop_flag = Some(stop_flag);
    }

    // ============================================================
    // Required Method
    // ============================================================

    // Runs the parallel DBSCAN pipeline over angle buckets using Rayon
    fn db_scan_clustering(
        &self,
        raw_trajectories: &RawTrajectories,
        clustered_trajectories: &mut ClusteredTrajectories,
    ) -> bool {
        // Phase 1: parallel discovery
        self.emit_start_clustering(raw_trajectories);
        let results: Vec<Vec<Cluster>> = self.complete_parallel_clustering(raw_trajectories);

        if self.is_stopped() {
            return false;
        }
        self.emit_complete_clustering();

        // Phase 2: serial fill in non-clustered segments
        self.fill_non_clustered_segments(raw_trajectories, clustered_trajectories);

        // Phase 3: serial commit (regroup clusters)
        for clusters in results {
            clustered_trajectories.add_list_cluster(clusters);
        }

        // Phase 4: create corridors from clusters and finalize non-clustered segments
        self.emit_start_remove_duplicates(clustered_trajectories);
        self.create_corridors(clustered_trajectories);

        if self.is_stopped() {
            return false;
        }
        self.emit_complete_remove_duplicates();

        true
    }
}
