pub struct EntityChanges {
    /// indicates if this entity is alive! if it's dead, it's removed from the
    /// world
    alive: bool,
    /// a tuple of entities to spawn in
    /// 
    /// first element of tuple is the 
    /// 
    /// what new entities are added to the world
    spawn: Vec<Box<dyn Entity>>,
}

pub trait Entity {
    /// returns true if it's still alive! if it's dead, it's removed from the
    /// world
    fn update(&mut self, world_data: &mut serde_json::Value) -> Result<bool, String>;

    fn draw(&self)
    
    /// occurs each frame after each entity has been sequentially updated
    /// 
    /// parallel_update might be executed in parallel between all entities
    fn parallel_update(&mut self) -> Result<(), String>;
    
    /// occurs each frame after each entity has been updated in parallel
    /// 
    /// it is checked if this entity is alive or not. if it is dead, then it is
    /// removed from the world now and will not be processed further.
    fn alive(&self) -> bool;
    
    /// occurs each frame after each entity has had its alive check
    /// 
    /// draw self by inputting draw command into the pipeline. the pipeline will
    /// be flushed
    fn draw_layer(&self) -> Result<(), String>;
}

pub struct World {
    
}