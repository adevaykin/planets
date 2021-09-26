use super::planetblock;

// Array is declared for keeping 25 planet blocks. 
pub struct Planet { 
    planetblocks_array: [[planetblock::PlanetBlock; 5]; 5],
               
}

// Initialization of array with PlanetBlock objects 
impl Planet {
    pub fn new() -> Self {        
       Planet {   
            planetblocks_array: [[planetblock::PlanetBlock::new(); 5]; 5]                     

        } 
       
    }
    // Updating the planet blocks age
    pub fn update(&mut self, time_delta: std::time::Duration) {
        for i in 0..5 {
            for j in 0..5 {
                self.planetblocks_array[i][j].update(time_delta); 

                // Printing of age of planet block
                println!("{} {} {:?}",i, j, self.planetblocks_array[i][j]);  
            }
        }

    }


}    
