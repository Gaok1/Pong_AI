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

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    vec,
};

mod game;
mod neural_network;

use neural_network::{
    network_drawer::NetworkDrawer,
    neural_network_f::NeuralNetwork,
    neuron::ActivationFunction,
};

const GAMES: i32 = 1900;
const GAMES_DRAWN: i32 = 100;
const GAMES_LINE: i32 = 20;

const ELITE_FRACTION: f64 = 0.12;
const MUTATION_RATE: f64 = 0.35;

#[macroquad::main(window_conf)]
async fn main() {
    let mut generation_average: Vec<(f64, f64)> = Vec::new();
    let mut generation_counter = Cell::new(0);
    
    // Cria a UI para desenhar (apenas a rede neural do melhor jogo)
    let mut network_drawer = NetworkDrawer::new(
        vec2(50.0, 50.0),
        15.0,
        50.0,
        30.0,
        Color::from_rgba(3, 223, 252, 255),
    );

    // Cria os jogos iniciais
    let mut games = create_initial_games();
    
    // Variáveis da câmera
    let mut scale = 1.0;
    let mut camera_pos = vec2(0.0, 0.0);
    let camera_speed = 5.0;

    // Índice do melhor jogo (aquele com a maior pontuação)
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

        // 4) Ajusta a câmera
        let camera = Camera2D {
            zoom: vec2(scale * 2.0 / screen_width(), scale * 2.0 / screen_height()),
            target: camera_pos,
            ..Default::default()
        };
        set_camera(&camera);

        // 5) Atualiza e desenha todos os jogos
        clear_background(WHITE);

        // Atualiza os jogos e determina quantos já terminaram
        let finished_count = update_all_games(&mut games, &mut best_game_index);
        draw_all_games(&mut games);

        // Se quase todos os jogos terminaram, gera uma nova geração
        if finished_count >= (GAMES as usize) - ((GAMES as f64 * ELITE_FRACTION).ceil() as usize) {
            let (avg, mut new_nns) = generate_nn(&mut games);
            regenerate_generation(&mut games, &mut new_nns);
            games.sort_by(|a, b| a.pong.position.y.partial_cmp(&b.pong.position.y).unwrap());
            generation_counter.set(generation_counter.get() + 1);
            generation_average.push((generation_counter.get() as f64, avg));
            
            // Desenha o gráfico da evolução da média de pontuação por geração.
            // Os dados do gráfico são gerados a partir dos pares (geração, média)
           
        }

       
        // 6) Desenha a rede neural do melhor jogo no canto superior esquerdo
        set_default_camera();
     
        network_drawer.draw(&mut games[best_game_index].neural_network.borrow_mut());
        draw_text(
            format!(
                "Pontuação: {}  {}/{} jogadores :D Geração {}",
                games[best_game_index].pontuation,
                finished_count,
                GAMES,
                generation_counter.get()
            )
            .as_str(),
            600.0,
            50.0,
            40.0,
            BLACK,
        );
        //use relative position of the screen
        let x = screen_width() - 330.0;
        draw_graphic(
            &generation_average.iter().map(|(gen, avg)| vec2(*gen as f32, *avg as f32)).collect(),
            vec2(x, 110.0), // Posição onde o gráfico será desenhado
            vec2(240.0, 160.0), // Tamanho da área do gráfico
        );
        next_frame().await;
    }
}

/// Função que retorna a configuração da janela.
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

/// Estrutura que mantém um Pong e a Rede Neural associada, além de armazenar a pontuação.
struct GamePack {
    pub neural_network: Rc<RefCell<NeuralNetwork>>,
    pub pong: Pong,
    pub finished: bool,
    pub pontuation: i32,
}

/// Cria o conjunto inicial de jogos (GamePack).
fn create_initial_games() -> Vec<GamePack> {
    let mut games: Vec<GamePack> = Vec::new();
    let mut game_x = 0.0;
    let mut game_y = 0.0;

    for i in 0..GAMES {
        // A cada nova linha, reinicia a posição horizontal e avança a vertical
        if i % GAMES_LINE == 0 {
            game_x = 0.0;
            game_y += 500.0;
        }

        let nn = Rc::new(RefCell::new(
            // Caso queira carregar uma rede treinada:
            NeuralNetwork::load_neural_network_bin("best_nn.bin").unwrap(),
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

        // Espaçamento horizontal entre os jogos
        game_x += 600.0;
    }
    games
}

/// Atualiza e conta quantos jogos estão finalizados. Também determina qual é o melhor jogo.
fn update_all_games(games: &mut [GamePack], best_game_index: &mut usize) -> usize {
    let mut finished_count = 0;
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
                let current_score = game.pong.pontuation.player1;
                if current_score > best_score {
                    best_score = current_score;
                    *best_game_index = i;
                }
                game.pontuation = current_score;
            }
        }
    }
    finished_count
}

/// Desenha todos os jogos ativos (não limpa o fundo a cada desenho).
fn draw_all_games(games: &mut [GamePack]) {
    let mut drawn = 0;
    for game in games.iter_mut() {
        if game.finished {
            continue;
        }
        game.pong.draw();
        drawn += 1;
        if drawn >= GAMES_DRAWN {
            break;
        }
    }
}

/// Gera nova geração de redes neurais e reinstancia cada jogo com a nova RNA, mantendo a mesma posição.
fn regenerate_generation(games: &mut Vec<GamePack>, new_nns: &mut Vec<Rc<RefCell<NeuralNetwork>>>) {
    for (i, game_pack) in games.iter_mut().enumerate() {
        let pos = game_pack.pong.position;

        let pong = Pong::new(
            GameWindow::new(500.0, 400.0),
            Box::new(new_nns[i].clone()),
            pos,
        );

        game_pack.neural_network = new_nns[i].clone();
        game_pack.pong = pong;
        game_pack.finished = false;
        game_pack.pontuation = 0;
    }
    println!("--- Nova geração recriada! ---");
}

/// Gera novas redes neurais com base na população anterior e retorna a média das pontuações.
fn generate_nn(game_packs: &mut [GamePack]) -> (f64, Vec<Rc<RefCell<NeuralNetwork>>>) {
    let total: i32 = game_packs.iter().map(|gp| gp.pontuation).sum();
    let avg = total as f64 / (GAMES as f64);
    println!("Pontuação média da geração anterior: {}", avg);
    game_packs.sort_by(|a, b| b.pontuation.cmp(&a.pontuation));

    let elite_count = (ELITE_FRACTION * GAMES as f64).ceil() as usize;
    let elite_count = elite_count.max(1).min(GAMES as usize);

    let best_nn = game_packs
        .iter()
        .take(elite_count)
        .map(|gp| gp.neural_network.clone())
        .collect::<Vec<_>>();

    // Salva a melhor rede neural
    best_nn[0].borrow().save_neural_network_bin("best_nn.bin");

    let mut new_nns = Vec::with_capacity(GAMES as usize);
    for nn_rc in &best_nn {
        new_nns.push(deep_clone_nn(nn_rc));
    }

    while new_nns.len() < GAMES as usize {
        let chosen = best_nn.choose(&mut rng()).unwrap();
        let mut nn = chosen.borrow().clone();

        // Aplica mutação nos pesos
        let weights = nn.all_weights_mut();
        for w in weights {
            if random::<f64>() < MUTATION_RATE {
                *w += random_range(-0.4..=0.4);
            }
        }
        new_nns.push(Rc::new(RefCell::new(nn)));
    }

    println!("Geradas {} novas redes neurais!", new_nns.len());
    (avg, new_nns)
}

/// Realiza deep clone de um Rc<RefCell<NeuralNetwork>>
fn deep_clone_nn(original: &Rc<RefCell<NeuralNetwork>>) -> Rc<RefCell<NeuralNetwork>> {
    let borrowed = original.borrow();
    let copy_of_nn = borrowed.clone();
    Rc::new(RefCell::new(copy_of_nn))
}

use macroquad::prelude::*;

/// Desenha um gráfico a partir dos dados, conectando os pontos com linhas,
/// e também desenha o contorno (quadrado) da área do gráfico.
///
/// - `data_points`: Vetor de pontos (x, y) representando os dados (ex.: média das pontuações por geração).
/// - `graph_origin`: Posição (x, y) do canto superior esquerdo onde o gráfico será desenhado.
/// - `graph_size`: Tamanho (largura, altura) da área disponível para o gráfico.
pub fn draw_graphic(data_points: &Vec<Vec2>, graph_origin: Vec2, graph_size: Vec2) {
    if data_points.is_empty() {
        return;
    }
    let title = "Média de Pontuação por Geração";

    draw_text(&title, graph_origin.x - 10.0, graph_origin.y - 10.0, 25.0, BLACK);

    // Calcula os limites dos dados.
    let data_min_x = data_points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
    let data_max_x = data_points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
    let data_min_y = data_points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
    let data_max_y = data_points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);

    // Determina o intervalo dos dados; evita divisão por zero.
    let data_range_x = if data_max_x - data_min_x == 0.0 { 1.0 } else { data_max_x - data_min_x };
    let data_range_y = if data_max_y - data_min_y == 0.0 { 1.0 } else { data_max_y - data_min_y };

    // Calcula os fatores de escala para mapear os dados à área do gráfico.
    let scale_x = graph_size.x / data_range_x;
    let scale_y = graph_size.y / data_range_y;

    // Converte os pontos dos dados para as coordenadas do gráfico.
    // O eixo Y é invertido para que valores maiores fiquem mais acima na área do gráfico.
    let transformed_points: Vec<Vec2> = data_points.iter().map(|p| {
        let x = graph_origin.x + (p.x - data_min_x) * scale_x;
        let y = graph_origin.y + graph_size.y - (p.y - data_min_y) * scale_y;
        Vec2::new(x, y)
    }).collect();

    // Desenha o contorno (quadrado) da área do gráfico.
    draw_rectangle_lines(
        graph_origin.x,
        graph_origin.y,
        graph_size.x,
        graph_size.y,
        2.0,  // espessura da linha
        BLACK,
    );

    // Desenha linhas conectando os pontos transformados.
    for i in 0..transformed_points.len() - 1 {
        let start = transformed_points[i];
        let end = transformed_points[i + 1];
        draw_line(start.x, start.y, end.x, end.y, 2.0, RED);
    }
}