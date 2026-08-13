/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// base_traclusdl.rs — shared TraClus trait with default DBSCAN steps

use crate::geometry::{segment::Segment, trajectory::Trajectory};
use crate::objects::corridor::Corridor;
use crate::objects::{
    cluster::Cluster,
    cluster_member::{ClusterMember, ClusterSeed},
};
use crate::storage::{
    clustered_trajectories::ClusteredTrajectories, raw_trajectories::RawTrajectories,
};
use crate::ui::args::TraclusArgs;
use crate::utils::events::app_events::{AppEvent, ComputationType};
use crate::utils::events::event_singleton::emit;
use crate::utils::parallel_runner::StopFlag;
use std::sync::atomic::Ordering;

// Trait for serial and parallel TraClus implementations
pub trait TraclusAlgorithm {
    // ============================================================
    // Shared Data Accessors
    // ============================================================
    fn args(&self) -> &TraclusArgs;

    fn stop_flag(&self) -> &Option<StopFlag>;
    fn set_stop_flag(&mut self, stop_flag: StopFlag);

    // ============================================================
    // Required Methods (Must Be Implemented by Implementations)
    // ============================================================

    // DBSCAN over trajectory segments; returns false if stopped early
    fn db_scan_clustering(
        &self,
        raw_trajectories: &RawTrajectories,
        clustered_trajectories: &mut ClusteredTrajectories,
    ) -> bool;

    // ============================================================
    // Default Methods (Can Be Overridden If Needed)
    // ============================================================

    // Finds density-reachable segments using angle, distance, and min_density constraints
    fn cluster_reachable_segs(
        &self,
        seed: ClusterSeed,
        nearby_trajs: &[Trajectory],
    ) -> Option<Cluster> {
        let mut cluster: Cluster = Cluster::new(seed, Vec::new());
        let seed_ref: &ClusterSeed = &cluster.seed;
        let mut local_weight: u32 = seed_ref.cm.weight;

        for nearby_traj in nearby_trajs {
            // Constraint 1: Skip if same trajectory
            if seed_ref.cm.traj_id == nearby_traj.id {
                continue;
            }

            // Constraint 2: Check angle difference
            let angle_diff: u16 = seed_ref.angle.min_diff(nearby_traj.angle);
            if angle_diff > self.args().max_angle.raw() {
                continue;
            }

            // Constraint 3: Check spatial distance
            let (dist, segment_id) = nearby_traj.distance_to_point(&seed_ref.cm.center);
            if dist > self.args().max_dist + 1e-9 {
                continue;
            }

            // Add qualifying segment as a candidate
            let segment: &Segment = nearby_traj.segment(segment_id).unwrap();
            let candidate: ClusterMember = ClusterMember::new(
                nearby_traj.id,
                segment_id,
                nearby_traj.weight,
                segment.middle,
                segment.start,
            );
            local_weight += candidate.weight;
            cluster.candidates.push(candidate);
        }

        // Constraint 4: Check density threshold (including seed weight)
        if local_weight < self.args().min_density {
            return None;
        }

        Some(cluster)
    }

    // BFS expansion: each candidate becomes a seed until no new candidates remain
    fn expand_segment_cluster<'a>(
        &self,
        cluster: &'a mut Cluster,
        nearby_trajs: &[Trajectory],
    ) -> &'a mut Cluster {
        while !cluster.candidates.is_empty() {
            let mut new_clusters: Vec<Cluster> = Vec::new();

            // Process candidates in reverse order for consistency with v1 behavior
            for candidate in cluster.candidates.iter().rev() {
                let seed_member: ClusterSeed = ClusterSeed::new(
                    ClusterMember::new_from_candidate(candidate),
                    cluster.seed.angle,
                );

                if let Some(new_cluster) = self.cluster_reachable_segs(seed_member, nearby_trajs) {
                    new_clusters.push(new_cluster);
                }
            }

            // Promote all candidates to members
            cluster.move_candidates_to_members();

            // Merge newly discovered clusters
            for new_cluster in new_clusters {
                cluster.merge_clusters(new_cluster);
            }
        }

        cluster
    }

    // Single-pass reachable set from seed without expansion
    fn initial_segment_cluster(
        &self,
        seed: (&Segment, &Trajectory),
        nearby_trajs: &[Trajectory],
    ) -> Option<Cluster> {
        let member: ClusterMember = ClusterMember::new_from_traj(seed.1, seed.0);
        let seed_member: ClusterSeed = ClusterSeed::new(member, seed.1.angle);
        self.cluster_reachable_segs(seed_member, nearby_trajs)
    }

    // Seeds non_clustered_segments from every raw trajectory segment
    fn fill_non_clustered_segments(
        &self,
        raw_trajectories: &RawTrajectories,
        clustered_trajectories: &mut ClusteredTrajectories,
    ) {
        for bucket in &raw_trajectories.traj_buckets {
            for traj_seed in &bucket.trajectories {
                clustered_trajectories.fill_non_clustered_segments(traj_seed);
            }
        }
    }

    // Converts priority-queue clusters into corridors, respecting stop signal
    fn create_corridors(&self, clustered_trajectories: &mut ClusteredTrajectories) {
        let mut num_last_elements: usize = clustered_trajectories.get_size_priority_queue();

        while let Some(completed_cluster) = clustered_trajectories.pop_and_clean(self.args()) {
            let index_corridor: usize = clustered_trajectories.corridors.len();
            let corridor: Corridor = Corridor::new(completed_cluster, index_corridor);
            clustered_trajectories.corridors.push(corridor);

            let num_current_elements: usize = clustered_trajectories.get_size_priority_queue();
            self.tick_remove_duplicates(num_last_elements, num_current_elements);
            num_last_elements = num_current_elements;

            // Bail out early on stop signal
            if self.is_stopped() {
                return;
            }
        }
        clustered_trajectories.take_non_clustered_segments();
    }

    // ============================================================
    // Emitter Helpers (For Emitting Progress Events During Clustering)
    // ============================================================

    // Returns true if a stop has been requested
    fn is_stopped(&self) -> bool {
        if let Some(stop_flag) = self.stop_flag() {
            return stop_flag.load(Ordering::Relaxed);
        }
        false
    }

    fn tick_clustering(&self, count: usize) {
        emit(AppEvent::ComputationProgress {
            computation_type: ComputationType::Clustering,
            increment_progress: count,
        });
    }

    fn tick_remove_duplicates(&self, num_last_elements: usize, num_current_elements: usize) {
        emit(AppEvent::ComputationProgress {
            computation_type: ComputationType::RemoveDuplicates,
            increment_progress: num_last_elements - num_current_elements,
        });
    }

    fn emit_start_clustering(&self, raw_trajectories: &RawTrajectories) {
        emit(AppEvent::ComputationStart {
            computation_type: ComputationType::Clustering,
            max_progress: raw_trajectories.get_num_trajectories(),
        });
    }

    fn emit_complete_clustering(&self) {
        emit(AppEvent::ComputationComplete {
            computation_type: ComputationType::Clustering,
        });
    }

    fn emit_start_remove_duplicates(&self, clustered_trajectories: &ClusteredTrajectories) {
        emit(AppEvent::ComputationStart {
            computation_type: ComputationType::RemoveDuplicates,
            max_progress: clustered_trajectories.get_size_priority_queue(),
        });
    }

    fn emit_complete_remove_duplicates(&self) {
        emit(AppEvent::ComputationComplete {
            computation_type: ComputationType::RemoveDuplicates,
        });
    }
}
