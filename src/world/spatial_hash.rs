use super::types::*;

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct SpatialHash {
    pub cell_size: f32,
    pub cells: HashMap<(i32, i32), Vec<usize>>,
}

impl SpatialHash {
    pub fn new(cell_size: f32) -> Self {
        SpatialHash {
            cell_size,
            cells: HashMap::new(),
        }
    }

    pub fn insert(&mut self, index: usize, world_pos: Vec2, size: Vec2) {
        let left = world_pos.x;
        let right = world_pos.x + size.x;
        let bottom = world_pos.y - size.y; // y-up world space: bottom is the smaller y
        let top = world_pos.y;

        let cx_min = (left / self.cell_size).floor() as i32;
        let cx_max = (right / self.cell_size).floor() as i32;
        let cy_min = (bottom / self.cell_size).floor() as i32;
        let cy_max = (top / self.cell_size).floor() as i32;

        for cx in cx_min..=cx_max {
            for cy in cy_min..=cy_max {
                self.cells.entry((cx, cy)).or_default().push(index);
            }
        }
    }

    pub fn query(&self, world_pos: Vec2, size: Vec2) -> impl Iterator<Item = usize> {
        let left = world_pos.x;
        let right = world_pos.x + size.x;
        let bottom = world_pos.y - size.y; // y-up world space: bottom is the smaller y
        let top = world_pos.y;

        let cx_min = (left / self.cell_size).floor() as i32;
        let cx_max = (right / self.cell_size).floor() as i32;
        let cy_min = (bottom / self.cell_size).floor() as i32;
        let cy_max = (top / self.cell_size).floor() as i32;

        let mut cells = HashSet::new();
        for cx in cx_min..=cx_max {
            for cy in cy_min..=cy_max {
                if let Some(indices) = self.cells.get(&(cx, cy)) {
                    cells.extend(indices.iter().copied());
                }
            }
        }
        cells.into_iter()
    }
}
