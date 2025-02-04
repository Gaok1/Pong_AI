use std::cell::RefCell;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

use macroquad::math::Vec2;
use rand::distr::uniform::SampleBorrow;
use serde::{Deserialize, Serialize};

use crate::game::controller::{Controller, PlayerDirection};

use crate::neural_network::neuron::{ActivationFunction, Neuron};

use super::layers::{self, Layer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralNetwork {
    pub inputs: Vec<f64>, // <-- NOVO: guarda os valores de entrada
    pub layers: Vec<Layer>,
}

impl NeuralNetwork {
    /// Cria uma rede neural com os inputs inicializados com zeros.
    pub fn new(
        input_size: usize,
        layers_sizes: &Vec<usize>,
        activation_functions: &Vec<ActivationFunction>,
    ) -> Result<Self, Box<dyn Error>> {
        if layers_sizes.len() != activation_functions.len() {
            return Err("layers_sizes e activation_functions devem ter o mesmo tamanho".into());
        }
        // Inicializa os inputs com zeros
        let inputs = vec![0.0; input_size];
        let mut layers = Vec::with_capacity(layers_sizes.len());
        let mut previous_size = input_size;
        for (&size, &activation) in layers_sizes.iter().zip(activation_functions.iter()) {
            layers.push(Layer::new(size, previous_size, activation));
            previous_size = size;
        }
        Ok(NeuralNetwork { inputs, layers })
    }

    pub fn from_model(model: &NeuralNetworkModel) -> Self {
        NeuralNetwork::new(
            model.input_layer_size,
            &model.hidden_layers_sizes,
            &model.activation_functions,
        )
        .unwrap()
    }

    /// Armazena os inputs e executa o feedforward
    pub fn feed(&mut self, inputs: &[f64]) {
        // Guarda os inputs para poder desenhá-los depois
        self.inputs = inputs.to_vec();
        let mut current_inputs = inputs.to_vec();
        for layer in self.layers.iter_mut() {
            layer.feed(&current_inputs);
            current_inputs = layer.get_outputs();
        }
    }

    pub fn get_output(&self) -> Result<Vec<f64>, &'static str> {
        if self.layers.is_empty() {
            return Err("A rede neural não possui camadas.");
        }
        Ok(self.layers.last().unwrap().get_outputs())
    }

    pub fn mut_layers(&mut self) -> &mut Vec<Layer> {
        &mut self.layers
    }

    pub fn save_neural_network_bin(&self, filename: &str) -> std::io::Result<()> {
        let encoded: Vec<u8> =
            bincode::serialize(self).expect("Falha ao serializar a rede neural.");
        let mut file = File::create(filename)?;
        file.write_all(&encoded)?;
        Ok(())
    }

    pub fn load_neural_network_bin(filename: &str) -> std::io::Result<NeuralNetwork> {
        let file = File::open(filename)?;
        let decoded: NeuralNetwork =
            bincode::deserialize_from(file).expect("Falha ao deserializar a rede neural.");
        Ok(decoded)
    }

    pub fn print_NeuralNetworkModel(&self) {
        self.layers[0].neurons[0].print_Neuron();
    }

    pub fn all_weights_mut(&mut self) -> Vec<&mut f64> {
        let mut weights: Vec<&mut f64> = Vec::new();
        for layer in self.layers.iter_mut() {
            for neuron in layer.neurons.iter_mut() {
                for weight in neuron.weights.iter_mut() {
                    weights.push(weight);
                }
                weights.push(&mut neuron.bias);
            }
        }

        weights
    }
}

#[derive(Debug, Clone)]
pub struct NeuralNetworkModel {
    pub input_layer_size: usize,
    pub hidden_layers_sizes: Vec<usize>,
    pub activation_functions: Vec<ActivationFunction>,
}

impl NeuralNetworkModel {
    pub fn new(
        input_layer_size: usize,
        hidden_layers_sizes: Vec<usize>,
        activation_functions: Vec<ActivationFunction>,
    ) -> Self {
        NeuralNetworkModel {
            input_layer_size,
            hidden_layers_sizes,
            activation_functions,
        }
    }

    pub fn generate(&self) -> NeuralNetwork {
        NeuralNetwork::new(
            self.input_layer_size,
            &self.hidden_layers_sizes,
            &self.activation_functions,
        )
        .unwrap()
    }

    pub fn from_neural_network(nn: NeuralNetwork) -> Self {
        let mut hidden_layers_sizes = Vec::new();
        let mut activation_functions = Vec::new();

        for layer in nn.layers.iter().skip(1) {
            hidden_layers_sizes.push(layer.neurons.len());
            activation_functions.push(layer.neurons[0].activation_function);
        }

        NeuralNetworkModel {
            input_layer_size: nn.inputs.len(), // agora os inputs estão armazenados
            hidden_layers_sizes,
            activation_functions,
        }
    }
}

// neural network must return 3 values : up,down, velocity. and
// input must be velocity x,y and ball relative position x,y
impl Controller for Rc<RefCell<NeuralNetwork>> {
    fn get_input(
        &mut self,
        ball_position: Vec2,
        ball_velocity: Vec2,
        player_position: Vec2,
    ) -> (PlayerDirection, f64) {
        // Você precisa decidir como extrair os valores (ou usar unwrap) se necessário.
        let bp = ball_position;
        let pp = player_position;

        let distance_ball_player_x = bp.x - pp.x;
        let distance_ball_player_y = bp.y - pp.y;

        self.borrow_mut().feed(&[
            distance_ball_player_x as f64,
            distance_ball_player_y as f64,
            ball_velocity.x as f64,
            ball_velocity.y as f64,
        ]);
        let output = self.borrow().get_output().unwrap();

        let mut direcition = PlayerDirection::None;
        if output[0] > 0.5 && output[1] > 0.5 {
            direcition = PlayerDirection::None;
        } else if output[0] < 0.5 {
            direcition = PlayerDirection::Up;
        } else if output[1] > 0.5 {
            direcition = PlayerDirection::Down;
        }

        (direcition, output[2])
    }
}
