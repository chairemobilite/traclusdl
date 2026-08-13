/*
 * Copyright 2026, Polytechnique Montreal and contributors
 *
 * This file is licensed under the MIT License.
 * License text available at https://opensource.org/licenses/MIT
 */

// trajectory.rs — OD line split into directed segments for clustering

use crate::utils::data_types::angle_u16::AngleU16;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::f64::consts::PI;

use super::input_od_line::InputODLine;
use super::point::Point;
use super::segment::Segment;

#[derive(Debug, Clone)]
pub struct Trajectory {
    pub id: usize,
    pub start: Point,
    pub end: Point,
    pub weight: u32,
    pub angle: AngleU16,
    segments: Vec<Segment>,
}

impl Trajectory {
    pub fn new(input: InputODLine, seg_size: f64) -> Self {
        let angle: AngleU16 = Self::get_spatial_angle(&input.start, &input.end);

        let mut traj: Trajectory = Self {
            id: input.line_id,
            start: input.start,
            end: input.end,
            weight: input.weight,
            angle,
            segments: Vec::new(),
        };

        traj.make_segments(seg_size);
        traj
    }

    fn get_spatial_angle(start: &Point, end: &Point) -> AngleU16 {
        let delta_y: f64 = end.y - start.y;
        let delta_x: f64 = end.x - start.x;
        let angle_deg: f64 = delta_y.atan2(delta_x).to_degrees();
        AngleU16::from_degrees(angle_deg)
    }

    fn get_spatial_length(&self) -> f64 {
        let dx: f64 = self.end.x - self.start.x;
        let dy: f64 = self.end.y - self.start.y;
        (dx * dx + dy * dy).sqrt()
    }

    // Minimum distance from a point to the trajectory, and index of the closest segment
    pub fn distance_to_point(&self, point: &Point) -> (f64, usize) {
        let px: f64 = point.x;
        let py: f64 = point.y;
        let x1: f64 = self.start.x;
        let y1: f64 = self.start.y;
        let dx: f64 = self.end.x - self.start.x;
        let dy: f64 = self.end.y - self.start.y;

        if dx == 0.0 && dy == 0.0 {
            let min_distance: f64 = ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
            return (min_distance, 0);
        }

        let t: f64 = ((px - x1) * dx + (py - y1) * dy) / (dx * dx + dy * dy);
        let t: f64 = t.clamp(0.0, 1.0);

        let near: Point = Point {
            x: x1 + t * dx,
            y: y1 + t * dy,
        };

        let min_distance: f64 = ((px - near.x).powi(2) + (py - near.y).powi(2)).sqrt();
        let traj_length: f64 = self.get_spatial_length();
        let seg_length: f64 = self.segments.first().unwrap().get_length();

        let index_seg: usize = (t * (traj_length / seg_length)) as usize;
        let index_seg_clamp: usize = index_seg.min(self.segments.len().saturating_sub(1));

        (min_distance, index_seg_clamp)
    }

    // Tiles trajectory into equal-length segments along its bearing
    pub fn make_segments(&mut self, segment_length: f64) {
        self.segments.clear();

        let angle_rad: f64 = self.angle.to_degrees() * PI / 180.0;
        let xstep: f64 = segment_length * angle_rad.cos();
        let ystep: f64 = segment_length * angle_rad.sin();

        let length: f64 = self.get_spatial_length();

        // Ceil avoids an extra partial segment from float error
        let nsegs: usize = ((length / segment_length) - 1e-9).ceil().max(1.0) as usize;
        let base_x: f64 = self.start.x;
        let base_y: f64 = self.start.y;

        for i in 0..nsegs {
            let start_x: f64 = base_x + (i as f64) * xstep;
            let start_y: f64 = base_y + (i as f64) * ystep;

            let end_x: f64 = base_x + ((i + 1) as f64) * xstep;
            let end_y: f64 = base_y + ((i + 1) as f64) * ystep;

            let seg_start: Point = Point {
                x: start_x,
                y: start_y,
            };
            let seg_end: Point = Point { x: end_x, y: end_y };

            let segment: Segment = Segment::new(i, seg_start, seg_end);
            self.segments.push(segment);
        }
    }

    pub fn segments_iter(&self) -> impl Iterator<Item = &Segment> {
        self.segments.iter()
    }

    pub fn segments_par_iter(&self) -> impl ParallelIterator<Item = &Segment> {
        self.segments.par_iter()
    }

    pub fn segment(&self, index: usize) -> Option<&Segment> {
        self.segments.get(index)
    }

    #[allow(unused)]
    pub fn print_info(&self) -> String {
        format!(
            "Trajectory ID: {}, Start: ({}, {}), End: ({}, {}), Weight: {}, Angle: {}",
            self.id, self.start.x, self.start.y, self.end.x, self.end.y, self.weight, self.angle
        )
    }
}
