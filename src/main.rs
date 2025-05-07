use rand::{thread_rng, Rng, ThreadRng};
use termion::raw::IntoRawMode;
use std::io::{stdout, Read};


const BOARD_WIDTH : usize = 40;
const BOARD_HEIGHT : usize = 40;
const NUM_ZOMBIES : usize = 10;
const NUM_HOLES : usize = 40;

const EMPTY_GLYPH : char = '🟫';
const ZOMBIE_GLYPH : char = '🧟';
const HOLE_GLYPH : char = '🟣';
const PLAYER_GLYPH : char = '😀';
const BORDER_GLYPH : char = '🟩';

struct Zombie {
    pos: (usize, usize),
    is_dead: bool,
}

struct Player {
    pos: (usize, usize),
}

struct Board([[char; BOARD_WIDTH]; BOARD_HEIGHT]);

enum WinCondition {
    None,
    Win,
    Lose,
}

impl Board {
    fn set(&mut self, pos: (usize, usize), glyph: char) {
        self.0[pos.1][pos.0] = glyph;
    }

    fn get(&self, pos: (usize, usize)) -> char {
        self.0[pos.1][pos.0]
    }

    fn draw(&self) {
        print!("\x1b[{}AZombies: use qwe|asd|zxc to move and t to teleport\r\n", BOARD_HEIGHT+1);
        for y in 0..BOARD_HEIGHT {
            let mut line = String::new();
            for cell in &self.0[y] {
                line.push(*cell);
            }
            print!("{}\r\n", line);
        }
    }

    fn gen_random_point(&self, rng: &mut ThreadRng) -> (usize, usize) {
        loop {
            let pos = (rng.gen_range(1, BOARD_WIDTH-1), rng.gen_range(1, BOARD_HEIGHT-1));
            if self.get(pos) == EMPTY_GLYPH {
                break pos;
            }
        }
    }
}


fn main() {
    // Set the terminal to RAW mode.
    let _stdout = stdout().into_raw_mode().unwrap();

    // Clear the screen.
    print!("\x1b[H\x1b[2J\x1b[3J");
    
    // Use the rand crate to generate holes, zombies and the player.
    // This gets re-seeded every time we run.
    let mut rng = thread_rng();

    let mut board = Board([[EMPTY_GLYPH; BOARD_WIDTH]; BOARD_HEIGHT]);

    // Make a border for the board
    for x in 0..BOARD_WIDTH {
        board.set((x, 0), BORDER_GLYPH);
        board.set((x, BOARD_HEIGHT-1), BORDER_GLYPH);
    }

    for y in 0..BOARD_HEIGHT {
        board.set((0, y), BORDER_GLYPH);
        board.set((BOARD_WIDTH-1, y), BORDER_GLYPH);
    }

    // Generate a set of zombies.
    let mut zombies = (0..NUM_ZOMBIES).map(|_| {
        let pos = board.gen_random_point(&mut rng);
        board.set(pos, ZOMBIE_GLYPH);
        Zombie { pos, is_dead: false }
    }).collect::<Vec<_>>();

    // And some holes.
    (0..NUM_HOLES).for_each(|_| {
        let pos = board.gen_random_point(&mut rng);
        board.set(pos, HOLE_GLYPH);
    });

    // And one player.
    let mut player = {
        let pos = board.gen_random_point(&mut rng);
        board.set(pos, PLAYER_GLYPH);
        Player { pos }
    };

    board.draw();
    loop {
        let mut buf = [0_u8; 1];
        let r = std::io::stdin().read(&mut buf).unwrap();
        let win = match &buf[0..r] {
            [b'q'] => do_move(&mut board, &mut player, &mut zombies, (-1, -1)),
            [b'w'] => do_move(&mut board, &mut player, &mut zombies, (0, -1)),
            [b'e'] => do_move(&mut board, &mut player, &mut zombies, (1, -1)),
            [b'a'] => do_move(&mut board, &mut player, &mut zombies, (-1, 0)),
            [b's'] => do_move(&mut board, &mut player, &mut zombies, (0, 0)),
            [b'd'] => do_move(&mut board, &mut player, &mut zombies, (1, 0)),
            [b'z'] => do_move(&mut board, &mut player, &mut zombies, (-1, 1)),
            [b'x'] => do_move(&mut board, &mut player, &mut zombies, (0, 1)),
            [b'c'] => do_move(&mut board, &mut player, &mut zombies, (1, 1)),
            [b't'] => {
                // Teleport.
                board.set(player.pos, EMPTY_GLYPH);
                let pos = board.gen_random_point(&mut rng);
                player.pos = pos;
                board.set(player.pos, PLAYER_GLYPH);
                do_move(&mut board, &mut player, &mut zombies, (0, 0))
            }
            [b'\x03'] => WinCondition::Lose,
            _ => WinCondition::None,
        };
        board.draw();
        match win {
            WinCondition::None => (),
            WinCondition::Win => {
                println!("You win!");
                break;
            }
            WinCondition::Lose => {
                println!("You lose!");
                break;
            }
        }
    }
}

fn do_move(board: &mut Board, player: &mut Player, zombies: &mut Vec<Zombie>, dir: (i32, i32)) -> WinCondition {
    let new_pos = (
        (player.pos.0 as i32 + dir.0) as usize,
        (player.pos.1 as i32 + dir.1) as usize
    );

    if dir != (0, 0) && board.get(new_pos) != EMPTY_GLYPH {
        return WinCondition::None;
    }

    board.set(player.pos, EMPTY_GLYPH);
    board.set(new_pos, PLAYER_GLYPH);
    player.pos = new_pos;

    for zombie in zombies.iter_mut() {
        if zombie.is_dead { continue }
        if board.get(zombie.pos) == ZOMBIE_GLYPH {
            board.set(zombie.pos, EMPTY_GLYPH);
            let dir = (
                if zombie.pos.0 > new_pos.0 { -1 } else if zombie.pos.0 < new_pos.0 { 1 } else { 0 },
                if zombie.pos.1 > new_pos.1 { -1 } else if zombie.pos.1 < new_pos.1 { 1 } else { 0 }
            );
            let new_zombie_pos = (
                (zombie.pos.0 as i32 + dir.0) as usize,
                (zombie.pos.1 as i32 + dir.1) as usize
            );
            match board.get(new_zombie_pos) {
                EMPTY_GLYPH => {
                     // Zombie moves.
                     board.set(new_zombie_pos, ZOMBIE_GLYPH); zombie.pos = new_zombie_pos
                }
                HOLE_GLYPH => {
                     // Zombie dies.
                     board.set(new_zombie_pos, EMPTY_GLYPH); zombie.is_dead = true
                }
                PLAYER_GLYPH => {
                     // Player dies.
                    board.set(new_zombie_pos, ZOMBIE_GLYPH); zombie.pos = new_zombie_pos;
                    return WinCondition::Lose;
                }
                ZOMBIE_GLYPH => {
                    // Zombie collision.
                    board.set(zombie.pos, ZOMBIE_GLYPH);
                }
                _ => (),
            }
        }
    }

    let win = zombies.iter().all(|z| z.is_dead);
    if win {
        WinCondition::Win
    } else {
        WinCondition::None
    }
}
