# LifeGame-core

The core logics for 'Conway's Game of Life'.

## Feature

* Made by Rust langurage

## Require

This requires the following products to build.

* Rust
    * A system programming language.

## How to embedded to your product

Add dependence to the `Cargo.toml` in your project.

```toml
[dependencies.lifegame]
git = "https://github.com/taturou/lifegame-core.git"
rev = "ver2.0.0"
```

Use the crate.

```rust
extern crate lifegame;

use lifegame::*;

fn main () {
    let (width, height) = (50, 25);

    let game = LifeGame::new(width, height);
    game.reset_by_rand();

    loop {
        game.evolution();
        print!("{}", game);
    }
}
```

## License

MIT License

