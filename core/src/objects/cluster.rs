/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// cluster.rs — DBSCAN cluster with seed, candidates, and members

use super::cluster_member::{ClusterMember, ClusterSeed};

pub struct Cluster {
    pub seed: ClusterSeed,
    pub total_weight: u32,
    pub candidates: Vec<ClusterMember>,
    pub members: Vec<ClusterMember>,
    pub sum_distance: f64,
}

impl Cluster {
    pub fn new(seed: ClusterSeed, candidates: Vec<ClusterMember>) -> Self {
        let weight: u32 = seed.cm.weight;
        Self {
            seed,
            total_weight: weight,
            candidates,
            members: Vec::new(),
            sum_distance: 0.0,
        }
    }

    pub fn move_candidates_to_members(&mut self) {
        while let Some(candidate) = self.candidates.pop() {
            self.total_weight += candidate.weight;
            self.sum_distance += self.distance_to_members(&candidate);
            self.members.push(candidate);
        }
    }

    // Appends unique candidates from another cluster (one segment per trajectory)
    pub fn merge_clusters(&mut self, other: Cluster) {
        for candidate in other.candidates {
            if !self.contains_traj(candidate.traj_id) {
                self.candidates.push(candidate);
            }
        }
    }

    pub fn contains_traj(&self, traj_id: usize) -> bool {
        for member in &self.members {
            if member.traj_id == traj_id {
                return true;
            }
        }
        for candidate in &self.candidates {
            if candidate.traj_id == traj_id {
                return true;
            }
        }
        if self.seed.cm.traj_id == traj_id {
            return true;
        }
        false
    }

    // Sum of distances to existing members — used for cluster ordering ties
    fn distance_to_members(&self, new_member: &ClusterMember) -> f64 {
        let mut total_dist: f64 = 0.0;
        for member in &self.members {
            let dx: f64 = member.start.x - new_member.start.x;
            let dy: f64 = member.start.y - new_member.start.y;
            total_dist += (dx * dx + dy * dy).sqrt();
        }
        total_dist
    }

    pub fn get_all_members_iter(&self) -> impl Iterator<Item = &ClusterMember> {
        std::iter::once(&self.seed.cm).chain(self.members.iter())
    }

    #[allow(unused)]
    pub fn print_info(&self) {
        println!(
            "Cluster Seed ID: ({}, {}), Total Weight: {}, Num members {}, Num Candidates: {}",
            self.seed.cm.traj_id,
            self.seed.cm.segment_id,
            self.total_weight,
            self.members.len(),
            self.candidates.len(),
        );
        for member in &self.members {
            println!(
                "  Member - Traj ID: {}, Segment ID: {}, Weight: {}",
                member.traj_id, member.segment_id, member.weight
            );
        }
    }
}
