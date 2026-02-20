use slotmap::new_key_type;

pub enum GameState {
    MainMenu,
    LoadingLevel(Level),
    LoadedLevel(Level)
}

pub struct Level(pub SceneHandle);

new_key_type! {
    pub struct SceneHandle;
}