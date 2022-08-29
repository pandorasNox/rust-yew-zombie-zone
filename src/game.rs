use std::collections::VecDeque;

use yewdux::prelude::*;

// #[derive(Default, Clone, PartialEq, Eq, Store)]
#[derive(Clone, PartialEq, Store)]
pub struct State {
    tick_interval_ms: u16,
    pub grid: Grid,
}

#[derive(Default, Clone, PartialEq)]
pub struct Grid([Option<Lane>; 5]);

#[derive(Default, Clone, PartialEq)]
pub struct Lane([Field; 9]);

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Field(VecDeque<Entity>);

#[derive(Debug, Clone, PartialEq)]
pub enum Entity {
    Zombie,
    Turret,
    Bullet,
}

impl Entity {
    //todo: maybe should be moved to Lane? (as Entity should probabbly not know about the lane logic and fields...)
    fn move_zombie(mut current_field: Field, mut opt_prev_field: Option<Field>) -> (Field, Option<Field>) {
        match opt_prev_field {
            Some(Field(mut prev_field)) => {
                let field_is_empty = prev_field.len() == 0;
                if field_is_empty {
                    current_field.pop_front();
                    prev_field.push_back(Entity::Zombie);

                    return (current_field, Some(Field(prev_field)));
                } else {
                    //todo:
                    return (current_field, Some(Field(prev_field)));
                }
            },
            None => {
                //todo:
                //end of lane, hit player
                current_field.pop_front();
                return (current_field, None);
            }
        }
    }
}

// inital state
impl std::default::Default for State {
    fn default() -> Self {
        State::new()
    }
}

impl State {
    fn new() -> State {
        State {
            tick_interval_ms: 700,
            // lane: Lane(Default::default()), //needs #[derive(Default)] on Field
            // lane: Lane(std::array::from_fn(|_| Field(VecDeque::new()))), //creates an array by repeatedly calling the closure
 
            // grid: Grid(Default::default()),
            // grid: Grid(std::array::from_fn(|_| { Some(Lane(Default::default())) } )),
            grid: Grid([
                None,
                None,
                Some(Lane(Default::default())),
                None,
                None,
            ]),
        }
    }

    pub fn next(&mut self) {
        self.grid = State::grid_next(self.grid.clone());
    }

    fn grid_next(mut grid: Grid) -> Grid {
        for i in 0..grid.len() {
            let opt_lane = grid[i].clone();
            match opt_lane {
                Some(lane) => {
                    grid[i] = Some(State::lane_next(lane));
                },
                None => {},
            }
        }

        grid
    }

    fn lane_next(mut lane: Lane) -> Lane {
        for i in 0..lane.len() {
            let Field(mut current_field) = lane[i].clone();
            let opt_prev_field = if i == 0 {None} else {Some(lane[i-1].clone())};
            let opt_next_field = if i+1 > lane.len() -1 {None} else {Some(lane[i+1].clone())};

            if current_field.len() == 0 {
                continue;
            }

            match current_field[0] {
                Entity::Zombie => {
                    let (z_current, z_prev) = Entity::move_zombie(Field(current_field), opt_prev_field);

                    lane[i] = z_current;
                    if i > 0 && z_prev != None {
                        lane[i-1] = z_prev.unwrap();
                    }
                }
                _ => {}
            }
        }

        lane
    }
}

// impl Grid {
//     pub fn len(&self) -> usize {
//         self.0.len()
//     }
// }
impl core::ops::Deref for Grid {
    type Target = [Option<Lane>; 5];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
// // DerefMut is optional
impl core::ops::DerefMut for Grid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl core::ops::Deref for Lane {
    type Target = [Field; 9];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::ops::DerefMut for Lane {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl core::ops::Deref for Field {
    type Target = VecDeque<Entity>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::ops::DerefMut for Field {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_zombie_moves_left() {
        let mut state = State {
            grid: Grid([
                None,
                None,
                Some(Lane([
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::from([Entity::Zombie])),
                ])),
                None,
                None,
            ]),
            ..Default::default()
        };

        let mut grid = state.clone().grid;
        let mut third_lane = grid[2].as_ref().unwrap();
        // let lane_field = &lane.0[8];

        assert_eq!(&Field(VecDeque::from([Entity::Zombie])), &third_lane.0[8]);
        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();
        assert_eq!( &Field(VecDeque::new()).len(), &third_lane.0[8].len() );
        assert_eq!(&Field(VecDeque::from([Entity::Zombie])), &third_lane.0[7]);

    }

    #[test]
    fn two_zombies_move_left() {
        let mut state = State {
            grid: Grid([
                None,
                None,
                Some(Lane([
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::new()),
                    Field(VecDeque::from([Entity::Zombie])),
                    Field(VecDeque::from([Entity::Zombie])),
                ])),
                None,
                None,
            ]),
            ..Default::default()
        };

        let mut grid = state.clone().grid;
        let mut third_lane = grid[2].as_ref().unwrap();
        // let lane_field = &lane.0[8];

        assert_eq!(&Field(VecDeque::from([Entity::Zombie])), &third_lane.0[8]);
        state.next();
        state.next();
        grid = state.grid.clone();
        third_lane = grid[2].as_ref().unwrap();
        assert_eq!( &Field(VecDeque::new()).len(), &third_lane.0[8].len() );
        assert_eq!(&Field(VecDeque::from([Entity::Zombie])), &third_lane.0[6]);
        assert_eq!(&Field(VecDeque::from([Entity::Zombie])), &third_lane.0[5]);

    }

}
