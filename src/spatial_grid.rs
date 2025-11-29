use crate::*;
use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_platform::collections::{HashMap, HashSet};
use bevy_reflect::prelude::*;

#[derive(Resource)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Resource))]
pub struct SpatialHashGrid {
    pub(crate) cell_size: f32,
    pub(crate) grid_to_ent: HashMap<IVec2, HashSet<Entity>>,
    pub(crate) ent_to_grid: HashMap<Entity, HashSet<IVec2>>,
}

impl Default for SpatialHashGrid {
    fn default() -> Self {
        Self {
            cell_size: Self::DEFAULT_CELL_SIZE,
            grid_to_ent: Default::default(),
            ent_to_grid: Default::default(),
        }
    }
}

impl SpatialHashGrid {
    pub const DEFAULT_CELL_SIZE: f32 = 20.0;

    pub fn insert_or_update(&mut self, ent: Entity, pos: &Position, coll: &Collider) {
        let cells = self.find_cells(pos, coll);

        let existing_cells = self.ent_to_grid.get(&ent).cloned().unwrap_or_default();
        if existing_cells != cells {
            for cell in &existing_cells {
                if let Some(set) = self.grid_to_ent.get_mut(cell) {
                    set.remove(&ent);
                }
            }

            self.ent_to_grid.insert(ent, cells.clone());
            for cell in cells {
                self.grid_to_ent.entry(cell).or_default().insert(ent);
            }
        }
    }

    pub fn remove(&mut self, ent: Entity) {
        if let Some(grid_set) = self.ent_to_grid.remove(&ent) {
            for grid in grid_set {
                if let Some(ent_set) = self.grid_to_ent.get_mut(&grid) {
                    ent_set.remove(&ent);
                }
            }
        }
    }

    fn find_cells(&self, pos: &Position, coll: &Collider) -> HashSet<IVec2> {
        let half_size = coll.size * 0.5;
        let max_bounds = pos.0 + half_size;
        let min_bounds = pos.0 - half_size;
        let min_cell = (min_bounds / self.cell_size).floor().as_ivec2();
        let max_cell = (max_bounds / self.cell_size).floor().as_ivec2();

        let mut cells = HashSet::new();

        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                cells.insert(IVec2::new(x, y));
            }
        }

        cells
    }

    pub fn iter(&self, ent: Entity) -> Option<HashSet<Entity>> {
        match self.ent_to_grid.get(&ent) {
            Some(grid_set) => {
                let mut entities = HashSet::new();

                for grid in grid_set {
                    match self.grid_to_ent.get(grid) {
                        Some(ent_set) => {
                            entities.extend(ent_set);
                        },

                        None => {
                            return None;
                        },
                    }
                }

                Some(entities)
            },

            None => None,
        }
    }
}
