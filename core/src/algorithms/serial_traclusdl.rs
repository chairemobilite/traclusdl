/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// serial_traclusdl.rs — single-threaded TraClus implementation

use super::super::geometry::trajectory::Trajectory;
use super::super::objects::cluster::Cluster;
use super::super::storage::{
    clustered_trajectories::ClusteredTrajectories, raw_trajectories::RawTrajectories,
};
use super::base_traclusdl::TraclusAlgorithm;

use crate::ui::args::TraclusArgs;
use crate::utils::parallel_runner::StopFlag;

pub struct SerialTraclusDL {
    args: TraclusArgs,
    stop_flag: Option<StopFlag>,
}

impl SerialTraclusDL {
    pub fn new(args: TraclusArgs) -> Self {
        Self {
            args,
            stop_flag: None,
        }
    }

    // Iterates angle buckets serially, clustering each trajectory against nearby copies
    fn complete_serial_clustering(
        &self,
        raw_trajectories: &RawTrajectories,
        clustered_trajectories: &mut ClusteredTrajectories,
    ) {
        for bucket in &raw_trajectories.traj_buckets {
            // Nearby trajectories copied once per bucket
            let nearby_trajs: Vec<Trajectory> =
                raw_trajectories.vec_nearby_angle(bucket.angle_start);

            // Iterate over trajectories in this bucket serially
            for traj_seed in &bucket.trajectories {
                let clusters: Vec<Cluster> =
                    self.individual_trajectory_clustering(traj_seed, &nearby_trajs);
                clustered_trajectories.add_list_cluster(clusters);

                // Tick after every trajectory (count = 1)
                self.tick_clustering(1);

                // Check for stop signal to bail out early
                if self.is_stopped() {
                    return;
                }
            }
        }
    }

    // Clusters each segment of traj_seed; expands and collects valid clusters
    fn individual_trajectory_clustering(
        &self,
        traj_seed: &Trajectory,
        nearby_trajs: &[Trajectory],
    ) -> Vec<Cluster> {
        let mut cluster_group: Vec<Cluster> = Vec::new();

        for seed_segment in traj_seed.segments_iter() {
            // Try to form an initial cluster from this seed segment
            let cluster: Option<Cluster> =
                self.initial_segment_cluster((seed_segment, traj_seed), nearby_trajs);

            if let Some(mut cluster) = cluster {
                // Expand the cluster to include all density-reachable segments
                self.expand_segment_cluster(&mut cluster, nearby_trajs);
                cluster_group.push(cluster);
            }
            // If no cluster forms, continue to next segment (not dense enough)
        }

        cluster_group
    }
}

impl TraclusAlgorithm for SerialTraclusDL {
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

    // Three-phase pipeline: discover clusters, fill leftovers, build corridors
    fn db_scan_clustering(
        &self,
        raw_trajectories: &RawTrajectories,
        clustered_trajectories: &mut ClusteredTrajectories,
    ) -> bool {
        // Phase 1: serial discovery
        self.emit_start_clustering(raw_trajectories);
        self.complete_serial_clustering(raw_trajectories, clustered_trajectories);

        if self.is_stopped() {
            return false;
        }
        self.emit_complete_clustering();

        // Phase 2: serial fill in non-clustered segments
        self.fill_non_clustered_segments(raw_trajectories, clustered_trajectories);

        // Phase 3: create corridors from clusters and finalize non-clustered segments
        self.emit_start_remove_duplicates(clustered_trajectories);
        self.create_corridors(clustered_trajectories);

        if self.is_stopped() {
            return false;
        }
        self.emit_complete_remove_duplicates();

        true
    }
}
