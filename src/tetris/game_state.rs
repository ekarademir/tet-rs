use super::Vertex;

#[derive(Default)]
pub(crate) struct GameState {}

impl<'a> GameState {
    pub(crate) fn game_area(&self) -> Vec<Vertex> {
        vec![
            [-1.0, -1.0, 0.0, 1.0].into(),
            [0.0, 1.0, 0.0, 1.0].into(),
            [1.0, -1.0, 0.0, 1.0].into(),
        ]
    }
}
