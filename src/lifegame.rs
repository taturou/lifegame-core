extern crate rand;

use std::fmt;
use self::rand::Rng;

pub struct LifeGame {
    generation: usize,
    world :Vec<u8>,
    width: usize,
    height: usize,
    callback: Box<FnMut(CallbackInfo)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CellInfo {
    pub x: usize,
    pub y: usize,
    pub live: bool
}

#[derive(Clone, Debug, PartialEq)]
pub enum CallbackEvent {
    Reset,
    Set,
    Evolution
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallbackInfo {
    pub event: CallbackEvent,
    pub generation: usize,
    pub width: usize,
    pub height: usize,
    pub num_cells: usize,
    pub cell: Option<CellInfo>
}

pub struct LifeGameIterBool<'a> {
    pos: usize,
    max: usize,
    live: Option<bool>,
    game: &'a LifeGame
}

pub struct LifeGameIterU8<'a> {
    pos: usize,
    max: usize,
    live: Option<bool>,
    game: &'a LifeGame
}

impl LifeGame {
    pub fn new(width: usize, height: usize) -> LifeGame {
        if (width == 0) || (height == 0) {
            panic!("Width or height must be not 0.");
        }

        let len = width * height;
        let world = vec![0; len];

        LifeGame {
            generation: 0,
            world,
            width,
            height,
            callback: Box::new(|_| {}),
        }
    }

    fn xy2i(&self, x: usize, y: usize) -> usize {
        (self.width * y) + x
    }

    fn get_as_u8(&self, x: usize, y: usize) -> u8 {
        let i = self.xy2i(x, y);
        self.world[i]
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        let live = self.get_as_u8(x, y);
        live > 0
    }

    fn set_u8(&mut self, x: usize, y: usize, live: u8) {
        let i = self.xy2i(x, y);
        self.world[i] = live;
    }

    pub fn set(&mut self, x: usize, y: usize, live: bool) -> &Self {
        let live = if live { 1 } else { 0 };
        self.set_u8(x, y, live);
        self.on_set(x, y, live);
        self
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    fn coordinate_normalize(n: isize, max: usize) -> usize {
        if n < 0 {
            ((max as isize) + n) as usize
        } else {
            let n = n as usize;
            if n >= max {
                n - max
            } else {
                n
            }
        }
    }

    fn neighbors_lives(&self, x: usize, y: usize) -> u8 {
        let x = x as isize;
        let y = y as isize;
        let width = self.width();
        let height = self.height();

        let mut count: u8 = 0;
        for j in (y-1)..(y+2) {
            for i in (x-1)..(x+2) {
                let i = LifeGame::coordinate_normalize(i, width);
                let j = LifeGame::coordinate_normalize(j, height);
                if self.get(i, j) {
                    count += 1;
                }
            }
        }
        if self.get(x as usize, y as usize) {
            count -= 1;
        }
        count
    }

    fn cell_evolution(&self, x: usize, y: usize) -> u8 {
        let live = self.get(x, y);
        let count = self.neighbors_lives(x, y);

        if live {
            match count {
                2 | 3 => 1,
                0 | 1 => 0,
                _     => 0
            }
        } else {
            match count {
                3 => 1,
                _ => 0
            }
        }
    }

    pub fn evolution(&mut self) -> &Self {
        let mut new = LifeGame::new(self.width, self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                let live = self.cell_evolution(x, y);
                new.set_u8(x, y, live);
            }
        }
        self.world = new.world;
        self.generation = self.generation() + 1;
        self.on_evolution();
        self
    }

    fn update_to_neighbors_lives(&mut self) -> &Self {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get(x, y) {
                    let lives = self.neighbors_lives(x, y);
                    self.set_u8(x, y, lives);
                }
            }
        }
        self
    }

    pub fn reset(&mut self) -> &Self {
        let len = self.width * self.height;
        self.world = vec![0; len];
        self.generation = 0;
        self.on_reset();
        self
    }

    pub fn reset_by_rand(&mut self) -> &Self {
        for y in 0..self.height {
            for x in 0..self.width {
                let live =
                    if rand::thread_rng().gen_range(0, 100) > 50 {
                        1
                    } else {
                        0
                    };
                self.set_u8(x, y, live);
            }
        }
        self.generation = 0;
        self.on_reset();
        self
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    pub fn set_callback<F>(mut self, callback: F) -> Self
        where F: FnMut(CallbackInfo) + 'static {
        self.callback = Box::new(callback);
        self
    }

    fn on_reset(&mut self) {
        let num_cells = self.num_cells();
        (self.callback)(
            CallbackInfo {
                event: CallbackEvent::Reset,
                generation: self.generation,
                width: self.width,
                height: self.height,
                num_cells: num_cells,
                cell: None
            });
    }

    fn on_set(&mut self, x: usize, y: usize, live: u8) {
        let live = if live == 1 { true } else { false };
        let num_cells = self.num_cells();
        (self.callback)(
            CallbackInfo {
                event: CallbackEvent::Set,
                generation: self.generation,
                width: self.width,
                height: self.height,
                num_cells: num_cells,
                cell: Some(CellInfo { x, y, live })
            });
    }

    fn on_evolution(&mut self) {
        let num_cells = self.num_cells();
        (self.callback)(
            CallbackInfo {
                event: CallbackEvent::Evolution,
                generation: self.generation,
                width: self.width,
                height: self.height,
                num_cells: num_cells,
                cell: None
            });
    }

    pub fn num_cells(&self) -> usize {
        self.world.iter().fold(0, |sum, &live| sum + (live as usize))
    }

    pub fn iter(&self, live: Option<bool>) -> LifeGameIterBool {
        let iter = LifeGameIterBool {
                        pos: 0,
                        max: self.width() * self.height(),
                        live: live,
                        game: self
                    };
        iter
    }

    pub fn iter_as_u8(&mut self, live: Option<bool>) -> LifeGameIterU8 {
        self.update_to_neighbors_lives();
        let iter = LifeGameIterU8 {
                        pos: 0,
                        max: self.width() * self.height(),
                        live: live,
                        game: self
                    };
        iter
    }
}

impl fmt::Display for LifeGame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let summary = format!("({}, {})", self.width, self.height);

        let mut world = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                //let live = self.get(x, y);
                //let cell = if live { "o " } else { "x " };
                let cell = &self.get_as_u8(x, y).to_string();
                world.push_str(cell);
            }
            world.push_str("\n");
        }

        write!(f, "{}\n{}", summary, world)
    }
}

impl<'a> Iterator for LifeGameIterBool<'a> {
    type Item = (usize, usize, bool);
    fn next (&mut self) -> Option<(usize, usize, bool)> {
        loop {
            if self.pos >= self.max {
                return None;
            }

            let pos = self.pos;
            self.pos += 1;

            let live = self.game.world[pos] > 0;
            if (self.live == None) || (self.live == Some(live)) {
                let x = pos % self.game.width();
                let y = pos / self.game.width();

                return Some((x, y, live));
            }
        }
    }
}

impl<'a> Iterator for LifeGameIterU8<'a> {
    type Item = (usize, usize, u8);
    fn next (&mut self) -> Option<(usize, usize, u8)> {
        loop {
            if self.pos >= self.max {
                return None;
            }

            let pos = self.pos;
            self.pos += 1;

            let cell = self.game.world[pos];
            let live = cell > 0;
            if (self.live == None) || (self.live == Some(live)) {
                let x = pos % self.game.width();
                let y = pos / self.game.width();

                return Some((x, y, cell));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn new_1x1() {
        let game = LifeGame::new(1, 1);
        assert_eq!(game.width(), 1);
        assert_eq!(game.height(), 1);
    }

    #[test]
    fn new_100x50() {
        let game = LifeGame::new(100, 50);
        assert_eq!(game.width(), 100);
        assert_eq!(game.height(), 50);
    }

    #[test]
    #[should_panic(expected = "Width or height must be not 0.")]
    fn new_width_and_height_are_0() {
        LifeGame::new(0, 0);
    }

    #[test]
    #[should_panic(expected = "Width or height must be not 0.")]
    fn new_width_is_0() {
        LifeGame::new(1, 0);
    }

    #[test]
    #[should_panic(expected = "Width or height must be not 0.")]
    fn new_height_is_0() {
        LifeGame::new(0, 1);
    }

    #[test]
    fn get_default_value_is_false() {
        let game = LifeGame::new(1, 1);
        assert_eq!(game.get(0, 0), false);
    }

    #[test]
    #[should_panic]
    fn get_x_over_width() {
        let game = LifeGame::new(1, 1);
        game.get(1, 0);
    }

    #[test]
    #[should_panic]
    fn get_y_over_height() {
        let game = LifeGame::new(1, 1);
        game.get(0, 1);
    }

    #[test]
    fn set_to_true_and_get() {
        let mut game = LifeGame::new(1, 1);
        game.set(0, 0, true);
        assert_eq!(game.get(0, 0), true);
    }

    #[test]
    fn set_to_false_and_get() {
        let mut game = LifeGame::new(1, 1);
        game.set(0, 0, true);
        game.set(0, 0, false);
        assert_eq!(game.get(0, 0), false);
    }

    #[test]
    #[should_panic]
    fn set_x_over_width() {
        let mut game = LifeGame::new(1, 1);
        game.set(1, 0, true);
    }

    #[test]
    #[should_panic]
    fn set_y_over_height() {
        let mut game = LifeGame::new(1, 1);
        game.set(0, 1, true);
    }

    #[test]
    fn reset() {
        let mut game = LifeGame::new(1, 1);
        game.set(0, 0, true);
        game.reset();
        assert_eq!(game.get(0, 0), false);
    }

    #[test]
    fn num_cells_default_is_0() {
        let game = LifeGame::new(1, 1);
        assert_eq!(game.num_cells(), 0);
    }

    #[test]
    fn num_cells_is_1() {
        let mut game = LifeGame::new(1, 1);
        game.set(0, 0, true);
        assert_eq!(game.num_cells(), 1);
    }

    #[test]
    fn num_cells_is_5000() {
        let mut game = LifeGame::new(100, 50);
        for y in 0..game.height() {
            for x in 0..game.width() {
                game.set(x, y, true);
            }
        }
        assert_eq!(game.num_cells(), 100 * 50);
    }

    #[test]
    fn evolution_with_generation() {
        /* .....    .....
         * .....    ..o..
         * .ooo. -> ..o..
         * .....    ..o..
         * .....    .....
         */
        let mut game = LifeGame::new(5, 5);
        game.set(1, 2, true);
        game.set(2, 2, true);
        game.set(3, 2, true);
        game.evolution();

        assert_eq!(game.get(0, 0), false);
        assert_eq!(game.get(1, 0), false);
        assert_eq!(game.get(2, 0), false);
        assert_eq!(game.get(3, 0), false);
        assert_eq!(game.get(4, 0), false);

        assert_eq!(game.get(0, 1), false);
        assert_eq!(game.get(1, 1), false);
        assert_eq!(game.get(2, 1), true);
        assert_eq!(game.get(3, 1), false);
        assert_eq!(game.get(4, 1), false);

        assert_eq!(game.get(0, 2), false);
        assert_eq!(game.get(1, 2), false);
        assert_eq!(game.get(2, 2), true);
        assert_eq!(game.get(3, 2), false);
        assert_eq!(game.get(4, 2), false);

        assert_eq!(game.get(0, 3), false);
        assert_eq!(game.get(1, 3), false);
        assert_eq!(game.get(2, 3), true);
        assert_eq!(game.get(3, 3), false);
        assert_eq!(game.get(4, 3), false);

        assert_eq!(game.get(0, 4), false);
        assert_eq!(game.get(1, 4), false);
        assert_eq!(game.get(2, 4), false);
        assert_eq!(game.get(3, 4), false);
        assert_eq!(game.get(4, 4), false);
    }

    #[test]
    fn evolution_with_survival() {
        /* ....      ....
         * .oo.  ->  .oo.
         * .oo.  ->  .oo.
         * ....      ....
         */
        let mut game = LifeGame::new(4, 4);
        game.set(1, 1, true);
        game.set(2, 1, true);
        game.set(1, 2, true);
        game.set(2, 2, true);
        game.evolution();

        assert_eq!(game.get(0, 0), false);
        assert_eq!(game.get(1, 0), false);
        assert_eq!(game.get(2, 0), false);
        assert_eq!(game.get(3, 0), false);
        assert_eq!(game.get(0, 1), false);
        assert_eq!(game.get(1, 1), true);
        assert_eq!(game.get(2, 1), true);
        assert_eq!(game.get(3, 1), false);
        assert_eq!(game.get(0, 2), false);
        assert_eq!(game.get(1, 2), true);
        assert_eq!(game.get(2, 2), true);
        assert_eq!(game.get(3, 2), false);
        assert_eq!(game.get(0, 3), false);
        assert_eq!(game.get(1, 3), false);
        assert_eq!(game.get(2, 3), false);
        assert_eq!(game.get(3, 3), false);
    }

    #[test]
    fn evolution_with_dead_by_depopulation_0() {
        /* ...      ...
         * ...  ->  ...
         * ...      ...
         */
        let mut game = LifeGame::new(3, 3);
        game.evolution();

        assert_eq!(game.get(0, 0), false);
        assert_eq!(game.get(1, 0), false);
        assert_eq!(game.get(2, 0), false);
        assert_eq!(game.get(0, 1), false);
        assert_eq!(game.get(1, 1), false);
        assert_eq!(game.get(2, 1), false);
        assert_eq!(game.get(0, 2), false);
        assert_eq!(game.get(1, 2), false);
        assert_eq!(game.get(2, 2), false);
    }

    #[test]
    fn evolution_with_dead_by_depopulation_1() {
        /* ...      ...
         * .o.  ->  ...
         * ...      ...
         */
        let mut game = LifeGame::new(3, 3);
        game.set(1, 1, true);
        game.evolution();

        assert_eq!(game.get(0, 0), false);
        assert_eq!(game.get(1, 0), false);
        assert_eq!(game.get(2, 0), false);
        assert_eq!(game.get(0, 1), false);
        assert_eq!(game.get(1, 1), false);
        assert_eq!(game.get(2, 1), false);
        assert_eq!(game.get(0, 2), false);
        assert_eq!(game.get(1, 2), false);
        assert_eq!(game.get(2, 2), false);
    }

    #[test]
    fn evolution_with_dead_by_depopulation_2() {
        /* .o.      ...
         * .o.  ->  ...
         * ...      ...
         */
        let mut game = LifeGame::new(3, 3);
        game.set(1, 1, true);
        game.set(1, 0, true);
        game.evolution();

        assert_eq!(game.get(0, 0), false);
        assert_eq!(game.get(1, 0), false);
        assert_eq!(game.get(2, 0), false);
        assert_eq!(game.get(0, 1), false);
        assert_eq!(game.get(1, 1), false);
        assert_eq!(game.get(2, 1), false);
        assert_eq!(game.get(0, 2), false);
        assert_eq!(game.get(1, 2), false);
        assert_eq!(game.get(2, 2), false);
    }

    #[test]
    fn evolution_with_dead_by_depopulation_3() {
        /* o..      ...
         * .o.  ->  ...
         * ...      ...
         */
        let mut game = LifeGame::new(3, 3);
        game.set(1, 1, true);
        game.set(0, 0, true);
        game.evolution();

        assert_eq!(game.get(0, 0), false);
        assert_eq!(game.get(1, 0), false);
        assert_eq!(game.get(2, 0), false);
        assert_eq!(game.get(0, 1), false);
        assert_eq!(game.get(1, 1), false);
        assert_eq!(game.get(2, 1), false);
        assert_eq!(game.get(0, 2), false);
        assert_eq!(game.get(1, 2), false);
        assert_eq!(game.get(2, 2), false);
    }

    #[test]
    fn evolution_with_dead_by_overpopulation_1() {
        /* ooo      ...
         * oo.  ->  ...
         * ...      ...
         */
        let mut game = LifeGame::new(3, 3);
        game.set(1, 1, true);
        game.set(0, 0, true);
        game.set(1, 0, true);
        game.set(2, 0, true);
        game.set(0, 1, true);
        game.evolution();

        assert_eq!(game.get(0, 0), false);
        assert_eq!(game.get(1, 0), false);
        assert_eq!(game.get(2, 0), false);
        assert_eq!(game.get(0, 1), false);
        assert_eq!(game.get(1, 1), false);
        assert_eq!(game.get(2, 1), false);
        assert_eq!(game.get(0, 2), false);
        assert_eq!(game.get(1, 2), false);
        assert_eq!(game.get(2, 2), false);
    }

    #[test]
    fn evolution_with_dead_by_overpopulation_2() {
        /* ooo      ...
         * ooo  ->  ...
         * ...      ...
         */
        let mut game = LifeGame::new(3, 3);
        game.set(1, 1, true);
        game.set(0, 0, true);
        game.set(1, 0, true);
        game.set(2, 0, true);
        game.set(0, 1, true);
        game.set(2, 1, true);
        game.evolution();

        assert_eq!(game.get(0, 0), false);
        assert_eq!(game.get(1, 0), false);
        assert_eq!(game.get(2, 0), false);
        assert_eq!(game.get(0, 1), false);
        assert_eq!(game.get(1, 1), false);
        assert_eq!(game.get(2, 1), false);
        assert_eq!(game.get(0, 2), false);
        assert_eq!(game.get(1, 2), false);
        assert_eq!(game.get(2, 2), false);
    }

    #[test]
    fn evolution_with_dead_by_overpopulation_3() {
        /* ooo      ...
         * ooo  ->  ...
         * o..      ...
         */
        let mut game = LifeGame::new(3, 3);
        game.set(1, 1, true);
        game.set(0, 0, true);
        game.set(1, 0, true);
        game.set(2, 0, true);
        game.set(0, 1, true);
        game.set(2, 1, true);
        game.set(0, 2, true);
        game.evolution();

        assert_eq!(game.get(0, 0), false);
        assert_eq!(game.get(1, 0), false);
        assert_eq!(game.get(2, 0), false);
        assert_eq!(game.get(0, 1), false);
        assert_eq!(game.get(1, 1), false);
        assert_eq!(game.get(2, 1), false);
        assert_eq!(game.get(0, 2), false);
        assert_eq!(game.get(1, 2), false);
        assert_eq!(game.get(2, 2), false);
    }

    #[test]
    fn evolution_with_dead_by_overpopulation_4() {
        /* ooo      ...
         * ooo  ->  ...
         * oo.      ...
         */
        let mut game = LifeGame::new(3, 3);
        game.set(1, 1, true);
        game.set(0, 0, true);
        game.set(1, 0, true);
        game.set(2, 0, true);
        game.set(0, 1, true);
        game.set(2, 1, true);
        game.set(0, 2, true);
        game.set(1, 2, true);
        game.evolution();

        assert_eq!(game.get(0, 0), false);
        assert_eq!(game.get(1, 0), false);
        assert_eq!(game.get(2, 0), false);
        assert_eq!(game.get(0, 1), false);
        assert_eq!(game.get(1, 1), false);
        assert_eq!(game.get(2, 1), false);
        assert_eq!(game.get(0, 2), false);
        assert_eq!(game.get(1, 2), false);
        assert_eq!(game.get(2, 2), false);
    }

    #[test]
    fn evolution_with_dead_by_overpopulation_5() {
        /* ooo      ...
         * ooo  ->  ...
         * ooo      ...
         */
        let mut game = LifeGame::new(3, 3);
        game.set(1, 1, true);
        game.set(0, 0, true);
        game.set(1, 0, true);
        game.set(2, 0, true);
        game.set(0, 1, true);
        game.set(2, 1, true);
        game.set(0, 2, true);
        game.set(1, 2, true);
        game.set(2, 2, true);
        game.evolution();

        assert_eq!(game.get(0, 0), false);
        assert_eq!(game.get(1, 0), false);
        assert_eq!(game.get(2, 0), false);
        assert_eq!(game.get(0, 1), false);
        assert_eq!(game.get(1, 1), false);
        assert_eq!(game.get(2, 1), false);
        assert_eq!(game.get(0, 2), false);
        assert_eq!(game.get(1, 2), false);
        assert_eq!(game.get(2, 2), false);
    }

    #[test]
    fn evolution_with_dead_by_overpopulation_roll() {
        /* ooo      ...
         * ...  ->  ...
         * oo.      ...
         */
        let mut game = LifeGame::new(3, 3);
        game.set(0, 0, true);
        game.set(1, 0, true);
        game.set(2, 0, true);
        game.set(0, 2, true);
        game.set(1, 2, true);
        game.evolution();

        assert_eq!(game.get(0, 0), false);
        assert_eq!(game.get(1, 0), false);
        assert_eq!(game.get(2, 0), false);
        assert_eq!(game.get(0, 1), false);
        assert_eq!(game.get(1, 1), false);
        assert_eq!(game.get(2, 1), false);
        assert_eq!(game.get(0, 2), false);
        assert_eq!(game.get(1, 2), false);
        assert_eq!(game.get(2, 2), false);
    }

    #[test]
    fn generation_default_is_0() {
        let game = LifeGame::new(1, 1);
        assert_eq!(game.generation(), 0);
    }

    #[test]
    fn generation_is_0_after_reset() {
        let mut game = LifeGame::new(1, 1);
        game.evolution();
        game.reset();
        assert_eq!(game.generation(), 0);
    }

    #[test]
    fn generation_is_0_after_reset_by_rand() {
        let mut game = LifeGame::new(1, 1);
        game.evolution();
        game.reset_by_rand();
        assert_eq!(game.generation(), 0);
    }

    #[test]
    fn callback() {
        let info: Arc<Mutex<Option<CallbackInfo>>> = Arc::new(Mutex::new(None));
        let infocb = info.clone();

        let mut game = LifeGame::new(3, 3)
                        .set_callback(move |i| {
                            let mut info = infocb.lock().unwrap();
                            *info = Some(i);
                        });
        {
            let info = info.lock().unwrap();
            assert_eq!(*info, None);
        }

        game.set(0, 0, true);
        {
            let info = info.lock().unwrap();
            assert_eq!(*info,
                       Some(CallbackInfo {
                               event: CallbackEvent::Set,
                               generation: 0,
                               width: game.width(),
                               height: game.height(),
                               num_cells: 1,
                               cell: Some(CellInfo{ x:0, y:0, live:true })
                       }));
        }

        game.evolution();
        {
            let info = info.lock().unwrap();
            assert_eq!(*info,
                       Some(CallbackInfo {
                               event: CallbackEvent::Evolution,
                               generation: 1,
                               width: game.width(),
                               height: game.height(),
                               num_cells: 0,
                               cell: None
                       }));
        }

        game.reset();
        {
            let info = info.lock().unwrap();
            assert_eq!(*info,
                       Some(CallbackInfo {
                               event: CallbackEvent::Reset,
                               generation: 0,
                               width: game.width(),
                               height: game.height(),
                               num_cells: 0,
                               cell: None
                       }));
        }

        game.evolution();
        game.reset_by_rand();
        {
            let num_cells = game.num_cells();
            let info = info.lock().unwrap();
            assert_eq!(*info,
                       Some(CallbackInfo {
                               event: CallbackEvent::Reset,
                               generation: 0,
                               width: game.width(),
                               height: game.height(),
                               num_cells: num_cells,
                               cell: None
                       }));
        }
    }

    #[test]
    fn iter() {
        let mut game = LifeGame::new(2, 2);
        game.set(0, 0, true);
        game.set(1, 1, true);

        let mut iter = game.iter(None);
        assert_eq!(iter.next(), Some((0,0,true)));
        assert_eq!(iter.next(), Some((1,0,false)));
        assert_eq!(iter.next(), Some((0,1,false)));
        assert_eq!(iter.next(), Some((1,1,true)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_filter_live_true() {
        let mut game = LifeGame::new(2, 2);
        game.set(0, 0, true);
        game.set(1, 1, true);

        let mut iter = game.iter(Some(true));
        assert_eq!(iter.next(), Some((0,0,true)));
        assert_eq!(iter.next(), Some((1,1,true)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_filter_live_false() {
        let mut game = LifeGame::new(2, 2);
        game.set(0, 0, true);
        game.set(1, 1, true);

        let mut iter = game.iter(Some(false));
        assert_eq!(iter.next(), Some((1,0,false)));
        assert_eq!(iter.next(), Some((0,1,false)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_as_u8() {
        /*
         *  1 1 . 1 .      1 2 . 1 .
         *  . . 1 . .  ->  . . 3 . .
         *  . . 1 . .      . . 1 . .
         *  . . . . .      . . . . .
         */
        let mut game = LifeGame::new(5, 4);
        game.set(0, 0, true);
        game.set(1, 0, true);
        game.set(3, 0, true);
        game.set(2, 1, true);
        game.set(2, 2, true);

        let mut iter = game.iter_as_u8(None);
        assert_eq!(iter.next(), Some((0,0,1)));
        assert_eq!(iter.next(), Some((1,0,2)));
        assert_eq!(iter.next(), Some((2,0,0)));
        assert_eq!(iter.next(), Some((3,0,1)));
        assert_eq!(iter.next(), Some((4,0,0)));

        assert_eq!(iter.next(), Some((0,1,0)));
        assert_eq!(iter.next(), Some((1,1,0)));
        assert_eq!(iter.next(), Some((2,1,3)));
        assert_eq!(iter.next(), Some((3,1,0)));
        assert_eq!(iter.next(), Some((4,1,0)));

        assert_eq!(iter.next(), Some((0,2,0)));
        assert_eq!(iter.next(), Some((1,2,0)));
        assert_eq!(iter.next(), Some((2,2,1)));
        assert_eq!(iter.next(), Some((3,2,0)));
        assert_eq!(iter.next(), Some((4,2,0)));

        assert_eq!(iter.next(), Some((0,3,0)));
        assert_eq!(iter.next(), Some((1,3,0)));
        assert_eq!(iter.next(), Some((2,3,0)));
        assert_eq!(iter.next(), Some((3,3,0)));
        assert_eq!(iter.next(), Some((4,3,0)));

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_as_u8_filter_live_true() {
        /*
         *  1 1 . 1 .      1 2 . 1 .
         *  . . 1 . .  ->  . . 3 . .
         *  . . 1 . .      . . 1 . .
         *  . . . . .      . . . . .
         */
        let mut game = LifeGame::new(5, 4);
        game.set(0, 0, true);
        game.set(1, 0, true);
        game.set(3, 0, true);
        game.set(2, 1, true);
        game.set(2, 2, true);

        let mut iter = game.iter_as_u8(Some(true));
        assert_eq!(iter.next(), Some((0,0,1)));
        assert_eq!(iter.next(), Some((1,0,2)));
        assert_eq!(iter.next(), Some((3,0,1)));

        assert_eq!(iter.next(), Some((2,1,3)));

        assert_eq!(iter.next(), Some((2,2,1)));

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_as_u8_filter_live_false() {
        /*
         *  1 1 . 1 .      1 2 . 1 .
         *  . . 1 . .  ->  . . 3 . .
         *  . . 1 . .      . . 1 . .
         *  . . . . .      . . . . .
         */
        let mut game = LifeGame::new(5, 4);
        game.set(0, 0, true);
        game.set(1, 0, true);
        game.set(3, 0, true);
        game.set(2, 1, true);
        game.set(2, 2, true);

        let mut iter = game.iter_as_u8(Some(false));
        assert_eq!(iter.next(), Some((2,0,0)));
        assert_eq!(iter.next(), Some((4,0,0)));

        assert_eq!(iter.next(), Some((0,1,0)));
        assert_eq!(iter.next(), Some((1,1,0)));
        assert_eq!(iter.next(), Some((3,1,0)));
        assert_eq!(iter.next(), Some((4,1,0)));

        assert_eq!(iter.next(), Some((0,2,0)));
        assert_eq!(iter.next(), Some((1,2,0)));
        assert_eq!(iter.next(), Some((3,2,0)));
        assert_eq!(iter.next(), Some((4,2,0)));

        assert_eq!(iter.next(), Some((0,3,0)));
        assert_eq!(iter.next(), Some((1,3,0)));
        assert_eq!(iter.next(), Some((2,3,0)));
        assert_eq!(iter.next(), Some((3,3,0)));
        assert_eq!(iter.next(), Some((4,3,0)));

        assert_eq!(iter.next(), None);
    }
}
