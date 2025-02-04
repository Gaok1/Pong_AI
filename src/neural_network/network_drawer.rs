use macroquad::text::{draw_text_ex, TextParams};

use macroquad::{
    color::{Color, BLACK},
    math::Vec2,
    shapes::{draw_circle, draw_line},
    text::draw_text,
};

use crate::NeuralNetwork;

pub struct NetworkDrawer {
    position: Vec2,
    node_radius: f32,
    layer_distance: f32,
    neuron_distance: f32,
    color: Color,
}

impl NetworkDrawer {
    pub fn new(
        position: Vec2,
        node_radius: f32,
        layer_distance: f32,
        neuron_distance: f32,
        color: Color,
    ) -> Self {
        NetworkDrawer {
            position,
            node_radius,
            layer_distance,
            neuron_distance,
            color,
        }
    }

    pub fn draw(&mut self, neural_network: &mut NeuralNetwork) {
        let mut x = self.position.x;
        let mut y = self.position.y;

        // Desenha a camada de inputs
        let input_nodes_count = neural_network.inputs.len();
        let mut input_positions = Vec::new();
        for i in 0..input_nodes_count {
            let pos: (f32, f32) = (x, y);
            input_positions.push(pos);
            let input_value = neural_network.inputs[i] as f32;
            let node_color = Color {
                r: self.color.r,
                g: self.color.g,
                b: self.color.b,
                a: 1.0,
            };
            // Desenha a bolinha do input
            draw_circle(pos.0, pos.1, self.node_radius, node_color);
            // Desenha o texto (valor do input) no centro da bolinha
            // Ajusta o posicionamento do texto para centralizar (pode ser refinado conforme sua fonte)
            let text = format!("{:.2}", input_value);
            let font_size = 25.0;
            let text_offset_x = self.node_radius * 0.6; // Ajuste conforme necessário
            let text_offset_y = self.node_radius * 0.4;
            draw_text(
                &text,
                pos.0 - text_offset_x,
                pos.1 + text_offset_y,
                font_size,
                BLACK,
            );
            y += self.neuron_distance;
        }

        // Avança para a próxima camada (a primeira camada oculta)
        x += self.layer_distance;

        // Para cada camada oculta, define as posições dos neurônios
        for layer in neural_network.layers.iter_mut() {
            let mut neuron_y = self.position.y; // reinicia y para cada camada
            for neuron in layer.neurons.iter_mut() {
                neuron.position = (x, neuron_y);
                neuron_y += self.neuron_distance;
            }
            x += self.layer_distance;
        }

        // Desenha as conexões entre a camada de inputs e a primeira camada oculta (se existir)
        if !neural_network.layers.is_empty() {
            for &input_pos in input_positions.iter() {
                for neuron in neural_network.layers[0].neurons.iter() {
                    draw_line(
                        input_pos.0,
                        input_pos.1,
                        neuron.position.0,
                        neuron.position.1,
                        1.0,
                        self.color,
                    );
                }
            }
        }

        // Desenha conexões entre as camadas ocultas
        for i in 0..neural_network.layers.len() - 1 {
            for neuron in neural_network.layers[i].neurons.iter() {
                for next_neuron in neural_network.layers[i + 1].neurons.iter() {
                    let color = if neuron.output > 0.5 {
                        self.color
                    } else {
                        BLACK
                    };
                    draw_line(
                        neuron.position.0,
                        neuron.position.1,
                        next_neuron.position.0,
                        next_neuron.position.1,
                        1.0,
                        color,
                    );
                }
            }
        }

        // Desenha os neurônios das camadas ocultas
        for (i, layer) in neural_network.layers.iter().enumerate() {
            for neuron in layer.neurons.iter() {
                if i == neural_network.layers.len() - 1 {
                    let color = if neuron.output > 0.5 {
                        self.color
                    } else {
                        BLACK
                    };
                    draw_circle(
                        neuron.position.0,
                        neuron.position.1,
                        self.node_radius,
                        color,
                    );
                    continue;
                }
                let node_color = Color {
                    r: neuron.output as f32 * self.color.r,
                    g: neuron.output as f32 * self.color.g,
                    b: neuron.output as f32 * self.color.b,
                    a: 1.0,
                };
                draw_circle(
                    neuron.position.0,
                    neuron.position.1,
                    self.node_radius,
                    node_color,
                );
            }
        }

        let text_x_position =
            neural_network.layers.last().unwrap().neurons[0].position.0 + self.node_radius + 5.0;
        let mut text_y_position = neural_network.layers.last().unwrap().neurons[0].position.1;
        let text_params = TextParams {
            font_size: 30,
            font: None, // Você pode substituir por uma fonte customizada, se desejar
            color: BLACK,
            ..Default::default()
        };

        draw_text_ex(
            format!(
                "UP: {:.2}",
                neural_network.layers.last().unwrap().get_outputs()[0]
            )
            .as_str(),
            text_x_position,
            text_y_position,
            text_params.clone(),
        );
        text_y_position += self.neuron_distance;
        draw_text_ex(
            format!(
                "DOWN: {:.2}",
                neural_network.layers.last().unwrap().get_outputs()[1]
            )
            .as_str(),
            text_x_position,
            text_y_position,
            text_params.clone(),
        );
        text_y_position += self.neuron_distance;

        draw_text_ex(
            format!(
                "VELOCITY: {:.2}",
                neural_network.layers.last().unwrap().get_outputs()[2]
            )
            .as_str(),
            text_x_position,
            text_y_position,
            text_params,
        );
    }
}
