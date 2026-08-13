/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// clustered_trajectories.rs — clustering results, corridors, and non-clustered segments

use super::super::geometry::trajectory::Trajectory;
use super::super::objects::cluster::Cluster;
use super::super::objects::cluster_member::ClusterMember;
use super::super::objects::corridor::Corridor;
use super::super::storage::priority_queue::PriorityQueueCluster;
use crate::ui::args::TraclusArgs;

pub struct ClusteredTrajectories {
    clusters: PriorityQueueCluster,
    pub corridors: Vec<Corridor>,
    pub non_clustered_segments: Vec<ClusterMember>,
    pub args_snapshot: TraclusArgs,
}

impl ClusteredTrajectories {
    pub fn new(args: &TraclusArgs) -> Self {
        Self {
            clusters: PriorityQueueCluster::new(),
            corridors: Vec::new(),
            non_clustered_segments: Vec::new(),
            args_snapshot: args.clone(),
        }
    }

    pub fn add_cluster(&mut self, cluster: Cluster) {
        self.clusters.push(cluster);
    }

    pub fn add_list_cluster(&mut self, clusters: Vec<Cluster>) {
        for cluster in clusters {
            self.add_cluster(cluster);
        }
    }

    pub fn pop_and_clean(&mut self, args: &TraclusArgs) -> Option<Cluster> {
        self.clusters.pop_and_clean(args)
    }

    pub fn take_non_clustered_segments(&mut self) {
        self.non_clustered_segments = std::mem::take(&mut self.clusters.non_clustered_segments);
        self.clusters = PriorityQueueCluster::new();
    }

    pub fn fill_non_clustered_segments(&mut self, trajectory: &Trajectory) {
        for segment in trajectory.segments_iter() {
            let cluster_member: ClusterMember = ClusterMember::new_from_traj(trajectory, segment);
            self.clusters.non_clustered_segments.push(cluster_member);
        }
    }

    // Yields (corridor_id, member); corridor_id -1 for unclustered
    pub fn get_all_cluster_members_iter(&self) -> impl Iterator<Item = (i32, &ClusterMember)> {
        let clustered = self
            .corridors
            .iter()
            .enumerate()
            .flat_map(|(corridor_idx, corridor)| {
                corridor
                    .cluster
                    .get_all_members_iter()
                    .map(move |cm| (corridor_idx as i32, cm))
            });
        let non_clustered = self.non_clustered_segments.iter().map(|cm| (-1, cm));

        clustered.chain(non_clustered)
    }

    pub fn get_summary(&self) -> Vec<String> {
        let total_clustered_segments: usize = self
            .corridors
            .iter()
            .map(|c| c.cluster.get_all_members_iter().count())
            .sum();
        vec![
            format!("=== Corridor Summary ==="),
            format!("- Total corridors found: {}", self.corridors.len()),
            format!("- Total clustered segments: {}", total_clustered_segments),
            format!(
                "- Total non-clustered segments: {}",
                self.non_clustered_segments.len()
            ),
        ]
    }

    pub fn get_size_priority_queue(&self) -> usize {
        self.clusters.get_size_elements()
    }
}
