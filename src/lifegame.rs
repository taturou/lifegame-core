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

pub struct CellInfo {
    pub x: usize,
    pub y: usize,
    pub live: bool
}

pub enum CallbackEvent {
    Set,
    Evolution
}

pub struct CallbackInfo {
    pub event: CallbackEvent,
    pub generation: usize,
    pub width: usize,
    pub height: usize,
    pub num_cells: usize,
    pub cell: Option<CellInfo>
}

impl LifeGame {
    pub fn new(width: usize, height: usize) -> LifeGame {
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

    fn get_raw(&self, x: usize, y: usize) -> u8 {
        let i = self.xy2i(x, y);
        self.world[i]
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        let live = self.get_raw(x, y);
        if live == 1 { true } else { false }
    }

    fn set_raw(&mut self, x: usize, y: usize, live: u8) {
        let i = self.xy2i(x, y);
        self.world[i] = live;
    }

    pub fn set(&mut self, x: usize, y: usize, live: bool) {
        let live = if live { 1 } else { 0 };
        self.set_raw(x, y, live);
        self.on_set(x, y, live);
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

    fn cell_evolution(&self, x: usize, y: usize) -> u8 {
        let live = self.get_raw(x, y);
        let x = x as isize;
        let y = y as isize;
        let width = self.width();
        let height = self.height();

        let mut count: u8 = 0;
        for j in (y-1)..(y+2) {
            for i in (x-1)..(x+2) {
                let i = LifeGame::coordinate_normalize(i, width);
                let j = LifeGame::coordinate_normalize(j, height);
                count += self.get_raw(i, j);
            }
        }
        count -= live;

        if live == 1 {
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
                new.set_raw(x, y, live);
            }
        }
        self.world = new.world;
        let new_generation = self.generation() + 1;
        self.set_generation(new_generation);
        self
    }

    pub fn reset(&mut self) -> &Self {
        let len = self.width * self.height;
        self.world = vec![0; len];
        self.set_generation(0);
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
                self.set_raw(x, y, live);
            }
        }
        self.set_generation(0);
        self
    }

    pub fn generation(&self) -> usize {
        self.generation
    }

    fn set_generation(&mut self, generation: usize) {
        self.generation = generation;
        self.on_evolution();
    }

    pub fn set_callback<F>(mut self, callback: F) -> Self
        where F: FnMut(CallbackInfo) + 'static {
        self.callback = Box::new(callback);
        self
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

    pub fn num_cells(&self) -> usize {
        self.world.iter().fold(0, |sum, &live| sum + (live as usize))
    }
}

impl fmt::Display for LifeGame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let summary = format!("({}, {})", self.width, self.height);

        let mut world = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let live = self.get(x, y);
                let cell = if live { "o " } else { "x " };
                world.push_str(cell);
            }
            world.push_str("\n");
        }

        write!(f, "{}\n{}", summary, world)
    }
}
