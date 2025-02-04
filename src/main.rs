use ::rand::{
    random, random_range, rng,
    seq::{IndexedRandom, SliceRandom},
    thread_rng, Rng,
};
use game::{
    controller::{ControlKeys, HumanController},
    pong::{GameStats, GameWindow, Pong},
};
use macroquad::{
    audio::{load_sound, play_sound, PlaySoundParams},
    prelude::*,
    rand::{gen_range as mq_gen_range, rand as mq_rand},
};

use std::{cell::RefCell, rc::Rc, vec};

mod game;
mod neural_network;

use neural_network::{
    network_drawer::NetworkDrawer, neural_network_f::NeuralNetwork, neuron::ActivationFunction,
};

const GAMES: i32 = 1900;
const GAMES_DRAWN: i32 = 10;
const GAMES_LINE: i32 = 20;

const ELITE_FRACTION: f64 = 0.08;
const MUTATION_RATE: f64 = 0.35;

#[macroquad::main(window_conf)]
async fn main() {
    let mut generation_media: Vec<(f64, f64)> = Vec::new();

    // Cria a UI pra desenhar (apenas a rede do melhor)
    let mut drawer = NetworkDrawer::new(
        vec2(50.0, 50.0),
        15.0,
        50.0,
        30.0,
        Color::from_rgba(3, 223, 252, 255),
    );

    // Cria os jogos iniciais
    let mut games = create_initial_games();
    let mut generations = 1;
    // Variáveis de câmera
    let mut scale = 1.0;
    let mut camera_pos = vec2(0.0, 0.0);
    let camera_speed = 5.0;

    // Index do melhor game
    let mut best_game_index: usize = 0;

    loop {
        // 1) Ajusta o zoom com a roda do mouse
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

        // 3) Se apertar Space, finaliza todos os jogos
        if is_key_pressed(KeyCode::Space) {
            for game in games.iter_mut() {
                if !game.finished {
                    game.finished = true;
                }
            }
        }

        // 4) Ajusta câmera
        let camera = Camera2D {
            zoom: vec2(scale * 2.0 / screen_width(), scale * 2.0 / screen_height()),
            target: camera_pos,
            ..Default::default()
        };

        set_camera(&camera);

        // 5) Atualiza e desenha todos os jogos
        clear_background(WHITE);

        // Se todos terminarem, regeneramos a geração
        let finished_count = update_all_games(&mut games, &mut best_game_index);
        draw_all_games(&mut games);

        if finished_count >= (GAMES as usize) - ((GAMES as f64 * ELITE_FRACTION).ceil() as usize) {
            regenerate_generation(&mut games);
            generations += 1;
            games.sort_by(|a, b| a.pong.position.y.partial_cmp(&b.pong.position.y).unwrap());
            // let total: i32 = games.iter().map(|gp| gp.pontuation).sum();
            // let media = total as f64 / (GAMES as f64);
            // generation_media.push((generations as f64, media));
        }

        // 6) Desenha a rede do melhor no canto superior esquerdo
        set_default_camera();
        // draw_media_graph(vec2(400.0, 200.0), &generation_media);
        drawer.draw(&mut games[best_game_index].neural_network.borrow_mut());
        draw_text(
            format!(
                "Pontuação: {}  {}/{} jogadores :D Geração {generations}",
                games[best_game_index].pontuation, finished_count, GAMES
            )
            .as_str(),
            600.0,
            50.0,
            40.0,
            BLACK,
        );
        next_frame().await;
    }
}

/// Função que retorna a configuração da janela
fn window_conf() -> Conf {
    Conf {
        window_title: "Pong AI com Scroll Zoom e WASD".to_owned(),
        window_width: 800,
        window_height: 600,
        high_dpi: false,
        fullscreen: false,
        ..Default::default()
    }
}

/// Estrutura que mantém um Pong e a Rede Neural associada, além de "pontuation".
struct GamePack {
    pub neural_network: Rc<RefCell<NeuralNetwork>>,
    pub pong: Pong,
    pub finished: bool,
    pub pontuation: i32,
}

/// Cria o conjunto inicial de jogos (GamePack)
fn create_initial_games() -> Vec<GamePack> {
    let mut games: Vec<GamePack> = Vec::new();
    let mut game_x = 0.0;
    let mut game_y = 0.0;

    for i in 0..GAMES {
        // A cada nova linha
        if i % GAMES_LINE == 0 {
            game_x = 0.0;
            game_y += 500.0;
        }

        let nn = Rc::new(RefCell::new(
            NeuralNetwork::new(
                4,
                &[5, 3].to_vec(),
                &[ActivationFunction::Sigmoid, ActivationFunction::Sigmoid].to_vec(),
            )
            .unwrap(),
        ));

        let pong = Pong::new(
            GameWindow::new(500.0, 400.0),
            Box::new(nn.clone()),
            vec2(game_x, game_y),
        );

        games.push(GamePack {
            neural_network: nn,
            pong,
            finished: false,
            pontuation: 0,
        });

        // Vai espaçando os jogos na horizontal
        game_x += 600.0;
    }
    games
}

/// Atualiza e conta quantos jogos estão finalizados. Atualiza também qual é o "melhor" game.
fn update_all_games(games: &mut [GamePack], best_game_index: &mut usize) -> usize {
    let mut finished_count = 0;

    // Precisamos saber qual a maior pontuação p1 e quem é o index
    let mut best_score = 0;

    for (i, game) in games.iter_mut().enumerate() {
        if game.finished {
            finished_count += 1;
            continue;
        }

        match game.pong.update() {
            Some(_stats) => {
                game.finished = true;
                finished_count += 1;
            }
            None => {
                // Se a pontuação atual do player1 é melhor, atualiza "best"
                let current_score = game.pong.pontuation.player1;
                if current_score > best_score {
                    best_score = current_score;
                    *best_game_index = i;
                }
                // Armazena a pontuação no GamePack
                game.pontuation = current_score;
            }
        }
    }

    finished_count
}

/// Desenha todos os jogos (sem limpar o fundo repetidamente)
fn draw_all_games(games: &mut [GamePack]) {
    let mut i = 0;
    for game in games.iter_mut() {
        if game.finished {
            continue;
        }
        game.pong.draw();
        i += 1;
        if i >= GAMES_DRAWN {
            break;
        }
    }
}

/// Gera nova geração de RNAs e reinstancia cada jogo com a nova RNA (no mesmo local).
fn regenerate_generation(games: &mut Vec<GamePack>) {
    // Gera as novas redes neurais
    let new_nns = generate_nn(games);

    // Recria os jogos, zerando as pontuações e "finished"
    for (i, game_pack) in games.iter_mut().enumerate() {
        // Pega a posição do jogo antigo
        let pos = game_pack.pong.position;

        // Cria um novo Pong com a nova RNA
        let pong = Pong::new(
            GameWindow::new(500.0, 400.0),
            Box::new(new_nns[i].clone()),
            pos,
        );

        // Substitui o game_pack
        game_pack.neural_network = new_nns[i].clone();
        game_pack.pong = pong;
        game_pack.finished = false;
        game_pack.pontuation = 0;
    }

    println!("--- Nova geração recriada! ---");
}

/// Gera novas redes neurais com base na população anterior
fn generate_nn(game_packs: &mut [GamePack]) -> Vec<Rc<RefCell<NeuralNetwork>>> {
    // Calcula média das pontuações
    let total: i32 = game_packs.iter().map(|gp| gp.pontuation).sum();
    let media = total as f64 / (GAMES as f64);
    println!("Pontuação média da geração anterior: {media}");

    game_packs.sort_by(|a, b| b.pontuation.cmp(&a.pontuation));

    // Pega a elite
    let elite_count = (ELITE_FRACTION * GAMES as f64).ceil() as usize;
    let elite_count = elite_count.max(1).min(GAMES as usize);

    let best_nn = game_packs
        .iter()
        .take(elite_count)
        .map(|gp| gp.neural_network.clone())
        .collect::<Vec<_>>();

    //salva a melhor rede neural
    best_nn[0].borrow().save_neural_network_bin("best_nn.bin");

    // Copia a elite inicial pro novo vetor
    let mut new_nns = Vec::with_capacity(GAMES as usize);
    for nn_rc in &best_nn {
        new_nns.push(deep_clone_nn(nn_rc));
    }

    while new_nns.len() < GAMES as usize {
        let chosen = best_nn.choose(&mut rng()).unwrap();
        let mut nn = chosen.borrow().clone();

        // Aplica mutação
        let weights = nn.all_weights_mut();
        for w in weights {
            if random::<f64>() < MUTATION_RATE {
                *w += random_range(-1.0..=1.0);
            }
        }
        new_nns.push(Rc::new(RefCell::new(nn)));
    }

    println!("Geradas {} novas redes neurais!", new_nns.len());
    new_nns
}

/// Função para fazer deep-clone de Rc<RefCell<NeuralNetwork>>
fn deep_clone_nn(original: &Rc<RefCell<NeuralNetwork>>) -> Rc<RefCell<NeuralNetwork>> {
    let borrowed = original.borrow();
    let copy_of_nn = borrowed.clone();
    Rc::new(RefCell::new(copy_of_nn))
}

const GRAPH_X_LEN: f32 = 150.0;
const GRAPH_Y_LEN: f32 = 100.0;

fn draw_media_graph(position: Vec2, generation_media: &[(f64, f64)]) {
    if generation_media.is_empty() {
        return;
    }

    // Calcula o mínimo e máximo dos valores de "media" em uma única passada
    let (min_media, max_media) = generation_media.iter().fold(
        (f64::INFINITY, f64::NEG_INFINITY),
        |(min, max), &(_, media)| (min.min(media), max.max(media)),
    );

    // Evita divisão por zero e adiciona uma margem de 10%
    let range = if (max_media - min_media).abs() < f64::EPSILON {
        1.0
    } else {
        max_media - min_media
    };
    let margin = range * 0.1;
    let adjusted_min = min_media - margin;
    let adjusted_range = (max_media + margin) - adjusted_min;

    // Desenha os eixos (porque até um gráfico precisa de direção na vida)
    draw_line(
        position.x,
        position.y,
        position.x + GRAPH_X_LEN,
        position.y,
        2.0,
        BLACK,
    );
    draw_line(
        position.x,
        position.y,
        position.x,
        position.y - GRAPH_Y_LEN,
        2.0,
        BLACK,
    );

    // Precalcula o divisor para a posição X (seu código também merece não repetir cálculos)
    let count = (generation_media.len() - 1) as f32;
    let mut last_point: Option<Vec2> = None;

    for (i, &(_, media)) in generation_media.iter().enumerate() {
        let i = i as f32;
        // Posiciona X linearmente de acordo com o índice
        let x = position.x + (i / count) * GRAPH_X_LEN;
        // Normaliza o valor para a altura do gráfico
        let normalized = ((media - adjusted_min) / adjusted_range) as f32;
        let y = position.y - normalized * GRAPH_Y_LEN;
        let current_point = vec2(x, y);

        // Conecta o ponto atual com o anterior (se houver, claro)
        if let Some(prev) = last_point {
            draw_line(prev.x, prev.y, x, y, 2.0, RED);
        }
        draw_circle(x, y, 2.0, RED);
        draw_text(&format!("{:.2}", media), x, y, 10.0, BLACK);
        last_point = Some(current_point);
    }
}
