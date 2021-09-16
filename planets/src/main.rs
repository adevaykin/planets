mod gameloop;

use crate::gameloop::GameLoop;

fn main() {
    println!("Hello, world!");

    let mut gameloop = GameLoop::new();
    gameloop.set_max_fps(2);
    let mut fame_num = 0 as u64;
    loop {
        // Notify frame start
        gameloop.start_frame();

        // All the game logic entry point is here.
        // Update single planet block object here using e.g. gameloop.get_prev_frame_time() function to get passed time
        println!("Frame {} started.", fame_num);

        // Increase frame count in the end
        fame_num += 1;
    }
}
