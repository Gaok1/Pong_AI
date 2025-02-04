use macroquad::input::{is_key_down, KeyCode};
use macroquad::prelude::Vec2;

use super::pong::PLAYER_VELOCITY;

/// Representa a direção do jogador.
#[derive(Debug)]
pub enum PlayerDirection {
    Up,
    Down,
    None,
}

/// Define o comportamento dos controladores.
pub trait Controller {
    /// Retorna a direção desejada com base nos inputs.
    /// Você pode usar Option<Vec2> caso precise da posição da bola ou do jogador.
    fn get_input(
        &mut self,
        ball_position: Vec2,
        ball_velocity: Vec2,
        player_position: Vec2,
    ) -> (PlayerDirection, f64); // velociade do jogador
}

/// Chaves de controle disponíveis.
#[derive(Debug)]
pub enum ControlKeys {
    Wasd,
    ArrowKeys,
}

/// Controlador humano.
pub struct HumanController {
    control_keys: ControlKeys,
}

impl HumanController {
    pub fn new(ctrl: ControlKeys) -> Self {
        HumanController { control_keys: ctrl }
    }
}

impl Controller for HumanController {
    fn get_input(
        &mut self,
        _ball_position: Vec2,
        _ball_velocity: Vec2,
        _player_position: Vec2,
    ) -> (PlayerDirection, f64) {
        let mut pd = PlayerDirection::None;
        if let ControlKeys::Wasd = self.control_keys {
            if is_key_down(KeyCode::W) {
                pd = PlayerDirection::Up;
            } else if is_key_down(KeyCode::S) {
                pd = PlayerDirection::Down;
            }
        } else if let ControlKeys::ArrowKeys = self.control_keys {
            if is_key_down(KeyCode::Up) {
                pd = PlayerDirection::Up;
            } else if is_key_down(KeyCode::Down) {
                pd = PlayerDirection::Down;
            }
        }
        (pd, 1.0 )
    }
}
