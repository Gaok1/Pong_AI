use macroquad::shapes::{draw_circle_lines, draw_rectangle, draw_rectangle_lines};
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
        // Cache de propriedades usadas frequentemente
        let mut x = self.position.x;
        let mut y = self.position.y;
        let neuron_distance = self.neuron_distance;
        let node_radius = self.node_radius;
        let color = self.color;
        
        // Pré-aloca os vetores para inputs
        let input_nodes_count = neural_network.inputs.len();
        let mut input_positions = Vec::with_capacity(input_nodes_count);
        let mut input_texts = Vec::with_capacity(input_nodes_count);
    
        // Desenha os inputs e armazena suas posições e textos
        for &input in neural_network.inputs.iter() {

            let pos = (x, y);
            input_positions.push(pos);
            draw_rectangle(pos.0, pos.1, node_radius * 1.3,node_radius * 1.3, Color { r: color.r, g: color.g, b: color.b, a: 1.0 });
            draw_rectangle_lines(pos.0, pos.1, node_radius * 1.3,node_radius * 1.3, 3.0, BLACK);
            input_texts.push((pos.0, pos.1, format!("{:.2}", input as f32)));
            y += neuron_distance;
            
        }
    
        // Avança para a primeira camada oculta
        x += self.layer_distance;
    
        // Define as posições dos neurônios em cada camada oculta
        for layer in neural_network.layers.iter_mut() {
            let mut neuron_y = self.position.y; // reinicia y para cada camada
            for neuron in layer.neurons.iter_mut() {
                neuron.position = (x, neuron_y);
                neuron_y += neuron_distance;
            }
            x += self.layer_distance;
        }
    
        // Desenha as conexões entre inputs e a primeira camada oculta (se existir)
        if let Some(first_layer) = neural_network.layers.first() {
            for &input_pos in input_positions.iter() {
                for neuron in first_layer.neurons.iter() {
                    draw_line(
                        input_pos.0,
                        input_pos.1,
                        neuron.position.0,
                        neuron.position.1,
                        1.0,
                        color,
                    );
                }
            }
        }
    
        // Desenha as conexões entre as camadas ocultas
        let num_layers = neural_network.layers.len();
        for i in 0..num_layers.saturating_sub(1) {
            let current_layer = &neural_network.layers[i];
            let next_layer = &neural_network.layers[i + 1];
            for neuron in current_layer.neurons.iter() {
                let line_color = if neuron.output > 0.5 { color } else { BLACK };
                for next_neuron in next_layer.neurons.iter() {
                    draw_line(
                        neuron.position.0,
                        neuron.position.1,
                        next_neuron.position.0,
                        next_neuron.position.1,
                        1.0,
                        line_color,
                    );
                }
            }
        }
    
        // Desenha os neurônios das camadas ocultas
        for (i, layer) in neural_network.layers.iter().enumerate() {
            let is_last = i == num_layers - 1;
            for neuron in layer.neurons.iter() {
                draw_circle_lines(neuron.position.0, neuron.position.1, node_radius - 1.0, 3.0, BLACK);
                let circle_color = if is_last {
                    if neuron.output > 0.5 { color } else { BLACK }
                } else {
                    Color {
                        r: neuron.output as f32 * color.r,
                        g: neuron.output as f32 * color.g,
                        b: neuron.output as f32 * color.b,
                        a: 1.0,
                    }
                };
                draw_circle(neuron.position.0, neuron.position.1, node_radius, circle_color);
            }
        }
    
        // Desenha os textos dos outputs da última camada
        if let Some(last_layer) = neural_network.layers.last() {
            // Evita repetir unwrap desnecessário
            let first_neuron = &last_layer.neurons[0];
            let text_x_position = first_neuron.position.0 + node_radius + 5.0;
            let mut text_y_position = first_neuron.position.1;
            let text_params = TextParams {
                font_size: 30,
                font: None,
                color: BLACK,
                ..Default::default()
            };
            let outputs = last_layer.get_outputs();
            draw_text_ex(&format!("UP: {:.2}", outputs[0]), text_x_position, text_y_position, text_params.clone());
            text_y_position += neuron_distance;
            draw_text_ex(&format!("DOWN: {:.2}", outputs[1]), text_x_position, text_y_position, text_params.clone());
            text_y_position += neuron_distance;
            draw_text_ex(&format!("VELOCITY: {:.2}", outputs[2]), text_x_position, text_y_position, text_params);
        }
    
        // Desenha os textos dos inputs para evitar sobreposição
        for &(x, y, ref text) in input_texts.iter() {
            let font_size = 25.0;
            let text_offset_x = node_radius * 0.6;
            let text_offset_y = node_radius * 0.4;
            draw_text(text, x - text_offset_x, y + text_offset_y, font_size, BLACK);
        }
    }
    
}
