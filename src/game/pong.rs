use ::rand::random_range;
use macroquad::prelude::*;

use crate::game::controller::{Controller, PlayerDirection};
#[derive(Debug, Clone, Copy)]
pub struct GameWindow {
    pub width: f32,
    pub height: f32,
}

impl GameWindow {
    pub fn new(width: f32, height: f32) -> Self {
        GameWindow { width, height }
    }
}

pub const PLAYER_VELOCITY: f32 = 8.0;
const PLAYER_HEIGHT: f32 = 70.0;
const PLAYER_WIDTH: f32 = 10.0;
const BALL_RADIUS: f32 = 10.0;


struct Player {
    pub position: Vec2,
    pub controller: Box<dyn Controller>,
}

impl Player {
    pub fn new(position: Vec2, controller: Box<dyn Controller>) -> Self {
        Player {
            position,
            controller,
        }
    }

    pub fn update(&mut self, ball_position: Vec2, ball_velocity: Vec2, player_position: Vec2) {
        let (direction, velocity) = self.controller.get_input(ball_position, ball_velocity, player_position);
        match direction {
            PlayerDirection::Up => {
                self.position.y -= (velocity as f32 * PLAYER_VELOCITY);
            }
            PlayerDirection::Down => {
                self.position.y += (velocity as f32 * PLAYER_VELOCITY);
            }
            PlayerDirection::None => {}
        }
    }
}

#[derive(Clone, Copy)]
pub struct Pontuation {
    pub player1: i32,
    pub player2: i32,
}

impl Pontuation {
    pub fn new() -> Self {
        Pontuation {
            player1: 0,
            player2: 0,
        }
    }

    pub fn increase_p1_score(&mut self) {
        self.player1 += 1;
    }

    pub fn increase_p2_score(&mut self) {
        self.player2 += 1;
    }
}

#[derive(Debug)]
pub enum Winner {
    Player1,
    Player2,
}

pub struct GameStats {
    pub winner: Winner,
    pub pontuation: Pontuation,
}

pub struct Pong {
    pub window: GameWindow,
    /// Posição do canto superior esquerdo do campo na tela
    pub position: Vec2,
    pub player1: Player,
 //   pub player2: Player,
    pub pontuation: Pontuation,
    pub player_scale: Vec2,
    pub ball: Ball,
    pub finished: bool,
}

impl Pong {
    /// Cria um Pong em determinada posição na tela,
    /// com tamanho definido por `window.width` e `window.height`.
    pub fn new(
        window: GameWindow,
        p1_controller: Box<dyn Controller>,
      //  p2_controller: Box<dyn Controller>,
        position: Vec2,
    ) -> Self {
        let player_scale = Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT);

        // Define a posição inicial dos jogadores com base em `position`
        let player1_pos = Vec2::new(
            position.x + 20.0,
            position.y + window.height / 2.0 - player_scale.y / 2.0,
        );
        // let player2_pos = Vec2::new(
        //     position.x + window.width - 30.0,
        //     position.y + window.height / 2.0 - player_scale.y / 2.0,
        // );

        // Cria a bola no centro do "campo"
        let ball_start_pos = Vec2::new(
            position.x + window.width / 2.0,
            position.y + window.height / 2.0,
        );
        let ball = Ball::new(ball_start_pos);

        Pong {
            window,
            position, // salva o offset de desenho
            player1: Player::new(player1_pos, p1_controller),
           // player2: Player::new(player2_pos, p2_controller),
            player_scale,
            pontuation: Pontuation::new(),
            ball,
            finished: false,
        }
    }

    /// Atualiza o jogo e retorna Some(GameStats) se terminou (por pontuação), ou None se continua.
    pub fn update(&mut self) -> Option<GameStats> {
        let dt = get_frame_time();
        self.player1.update(self.ball.position, self.ball.velocity, self.player1.position);
        //self.player2.update(self.ball.position, self.ball.velocity, self.player2.position);
        self.ball.update_position(dt);

        match self.check_collision() {
            Some(winner) => {
                // Reinicia a bola no centro após pontuação
                self.ball = Ball::new(Vec2::new(
                    self.position.x + self.window.width / 2.0,
                    self.position.y + self.window.height / 2.0,
                ));
                Some(GameStats {
                    winner,
                    pontuation: self.pontuation,
                })
            }
            None => None,
        }
    }

    pub fn draw(&self) {
        // Desenha o campo
        
        draw_rectangle_lines(
            self.position.x,
            self.position.y,
            self.window.width,
            self.window.height,
            5.0,
            BLACK,
        );

        // Desenha cada jogador
        draw_rectangle(
            self.player1.position.x,
            self.player1.position.y,
            self.player_scale.x,
            self.player_scale.y,
            BLACK,
        );
        // draw_rectangle(
        //     self.player2.position.x,
        //     self.player2.position.y,
        //     self.player_scale.x,
        //     self.player_scale.y,
        //     BLACK,
        // );

        // Escreve a pontuação
        draw_text(
            &format!(
                "Pontuação p1: {}  p2: {}",
                self.pontuation.player1, self.pontuation.player2
            ),
            self.position.x + 100.0,
            self.position.y - 20.0, // Desenha acima do campo, por exemplo
            30.0,
            PINK,
        );

        // Desenha a bola
        self.ball.draw();
    }

    fn check_collision(&mut self) -> Option<Winner> {
        let left_wall = self.position.x;
        let right_wall = self.position.x + self.window.width;
        let top_wall = self.position.y;
        let bottom_wall = self.position.y + self.window.height;

        // Se a bola sair pela esquerda ou direita, declara vencedor
        if self.ball.position.x - BALL_RADIUS <= left_wall {
            self.finished = true;
            return Some(Winner::Player2);
        }
        if self.ball.position.x + BALL_RADIUS >= right_wall {
            // self.finished = true;
            // return Some(Winner::Player1);
            self.ball.invert_velocity_x();
        }

        // Colisão top/bottom
        if self.ball.position.y - BALL_RADIUS <= top_wall
            || self.ball.position.y + BALL_RADIUS >= bottom_wall
        {
            self.ball.invert_velocity_y();
        }

        // Player1
        let player1_rect = Rect::new(
            self.player1.position.x,
            self.player1.position.y,
            PLAYER_WIDTH,
            PLAYER_HEIGHT,
        );
        if self.ball.collision_cooldown <= 0.0 && self.ball.rect().overlaps(&player1_rect) {
            self.ball.invert_velocity_x();
            self.pontuation.increase_p1_score();
            self.ball.collision_cooldown = 0.2;
            self.ball.position.x = self.player1.position.x + PLAYER_WIDTH + BALL_RADIUS;
        }

        // // Player2
        // let player2_rect = Rect::new(
        //     self.player2.position.x,
        //     self.player2.position.y,
        //     PLAYER_WIDTH,
        //     PLAYER_HEIGHT,
        // );
        // if self.ball.collision_cooldown <= 0.0 && self.ball.rect().overlaps(&player2_rect) {
        //     self.ball.invert_velocity_x();
        //     self.pontuation.increase_p2_score();
        //     self.ball.collision_cooldown = 0.2;
        //     self.ball.position.x = self.player2.position.x - BALL_RADIUS;
        // }

        // Impede que os players saiam do campo
        if self.player1.position.y < top_wall {
            self.player1.position.y = top_wall;
        }
        if self.player1.position.y + PLAYER_HEIGHT > bottom_wall {
            self.player1.position.y = bottom_wall - PLAYER_HEIGHT;
        }
        // if self.player2.position.y < top_wall {
        //     self.player2.position.y = top_wall;
        // }
        // if self.player2.position.y + PLAYER_HEIGHT > bottom_wall {
        //     self.player2.position.y = bottom_wall - PLAYER_HEIGHT;
        // }

        None
    }
}

struct Ball {
    pub position: Vec2,
    velocity: Vec2,
    acceleration: Vec2,
    pub collision_cooldown: f32,
}

const MIN_HORIZONTAL_SPEED: f32 = 3.0;

impl Ball {
    fn new(position: Vec2) -> Self {
        let speed = 3.0;
        let angle = random_range(0.0..std::f32::consts::TAU);
        let mut vx = angle.cos() * speed;
        let vy = angle.sin() * speed;

        // Ajusta componente horizontal se for muito pequena
        if vx.abs() < MIN_HORIZONTAL_SPEED {
            vx = if vx == 0.0 {
                MIN_HORIZONTAL_SPEED
            } else {
                vx.signum() * MIN_HORIZONTAL_SPEED
            };
        }

        let velocity = Vec2::new(vx, vy);
        Ball {
            position,
            velocity,
            acceleration: Vec2::ZERO,
            collision_cooldown: 0.0,
        }
    }

    /// Atualiza posição e collision_cooldown
    fn update_position(&mut self, dt: f32) {
        self.position += self.velocity;
        self.velocity += self.acceleration;
        if self.collision_cooldown > 0.0 {
            self.collision_cooldown -= dt;
        }
    }

    fn draw(&self) {
        draw_circle(self.position.x, self.position.y, BALL_RADIUS, RED);
    }

    fn rect(&self) -> Rect {
        Rect::new(
            self.position.x - BALL_RADIUS,
            self.position.y - BALL_RADIUS,
            BALL_RADIUS * 2.0,
            BALL_RADIUS * 2.0,
        )
    }

    pub fn invert_velocity_x(&mut self) {
        self.velocity.x = -self.velocity.x;
        if self.velocity.x.abs() < MIN_HORIZONTAL_SPEED {
            self.velocity.x = self.velocity.x.signum() * MIN_HORIZONTAL_SPEED;
        }
        self.velocity.y += random_range(-2.0..2.0);
    }

    pub fn invert_velocity_y(&mut self) {
        self.velocity.y = -self.velocity.y;
    }
}
