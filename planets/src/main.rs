mod gameloop;
mod sun;

use crate::gameloop::GameLoop;
use crate::sun::Sun;

fn main() {
    println!("Hello, world!");

    let mut gameloop = GameLoop::new();
    gameloop.set_max_fps(2);
    
    let mut fame_num = 0 as u64;
    
    let mut sun = Sun::new();

    loop {
        // Notify frame start
        gameloop.start_frame();

        // All the game logic entry point is here.
        // Update single planet block object here using e.g. gameloop.get_prev_frame_time() function to get passed time
        println!("Frame {} started.", fame_num);
        
        sun.sun_age_updt(gameloop.get_prev_frame_time());
        /// Что такое {:?}
        println!("Sun age is {}", sun.get_sun_age().as_millis()); 
        
        // Increase frame count in the end
        fame_num += 1;
        
    }
}
