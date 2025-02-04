use game::pong::{GameWindow, Pong};
use macroquad::prelude::*;
use neural_network::{
    network_drawer::NetworkDrawer,
    neural_network_f::{NeuralNetwork, NeuralNetworkModel},
};
use std::{cell::RefCell, rc::Rc};

mod game;
mod neural_network;

#[macroquad::main("Pong AI")]
async fn main() {
   
    let mut drawer = NetworkDrawer::new(
        vec2(0.0, 50.0),
        15.0,
        50.0,
        30.0,
        Color::from_rgba(3, 223, 252, 255),
    );

    let mut drawer2 = NetworkDrawer::new(
        vec2(600.0, 50.0),
        15.0,
        50.0,
        30.0,
        Color::from_rgba(250, 60, 60, 255),
    );

    let nn = Rc::new(RefCell::new(
        NeuralNetwork::load_neural_network_bin("best_nn.bin").unwrap(),
    ));
    let my_champion = NeuralNetwork::load_neural_network_bin("best_nn.bin").unwrap();
    let my_champion = Rc::new(RefCell::new(my_champion));   
    let mut game = Pong::new(
        GameWindow::new(500.0, 400.0),
        Box::new(nn.clone()),
        //Box::new(my_champion.clone()),
        Vec2::new(100.0, 100.0),
    );

    // Variáveis de câmera
    let mut scale = 1.0;
    let mut camera_pos = vec2(0.0, 0.0);
    let camera_speed = 5.0;


    loop {

        let scroll = mouse_wheel();
        if scroll.1 > 0.0 {
            scale += 0.1;
        } else if scroll.1 < 0.0 {
            scale -= 0.1;
            if scale < 0.1 {
                scale = 0.1;
            }
        }

        // 2) Move a câmera com WASD
        if is_key_down(KeyCode::W) {
            camera_pos.y -= camera_speed;
        }
        if is_key_down(KeyCode::S) {
            camera_pos.y += camera_speed;
        }
        if is_key_down(KeyCode::A) {
            camera_pos.x -= camera_speed;
        }
        if is_key_down(KeyCode::D) {
            camera_pos.x += camera_speed;
        }
        let camera = Camera2D {
            zoom: vec2(scale * 2.0 / screen_width(), scale * 2.0 / screen_height()),
            target: camera_pos,
            ..Default::default()
        };


        clear_background(WHITE);
        set_camera(&camera);
        match game.update(){
            Some(_) => {
                game = Pong::new(
                    GameWindow::new(500.0, 400.0),
                    Box::new(nn.clone()),
                    //Box::new(my_champion.clone()),
                    Vec2::new(100.0, 100.0),
                );
            }
            _ => {}
        }
        game.draw();
        drawer.draw(&mut nn.borrow_mut());
        drawer2.draw(&mut my_champion.borrow_mut());

        next_frame().await;
    }
}



