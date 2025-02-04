// neuron.rs

use rand::{random_range, Rng};
use serde::{Deserialize, Serialize};

/// Define as funções de ativação disponíveis para o neurônio.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ActivationFunction {
    Sigmoid,
    Relu,
    Tanh,
    Linear,
}

/// Estrutura que representa um único neurônio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neuron {
    pub weights: Vec<f64>,
    pub bias: f64,
    pub output: f64,
    pub activation_function: ActivationFunction,
    pub position: (f32, f32), // adaptaão pra desenho
}


/// Trait que define o comportamento de um neurônio.
pub trait NeuronTrait {
    /// Cria um novo neurônio com pesos e bias inicializados aleatoriamente.
    fn new(input_len: usize, activation_function: ActivationFunction) -> Self;

    /// Cria uma cópia de um neurônio existente.
    fn new_clone(neuron: &Neuron) -> Self;

    /// Cria um neurônio com pesos, bias e inputs definidos.
    fn new_seted(
        weights: Vec<f64>,
        bias: f64,
        activation_function: ActivationFunction,
    ) -> Self;

    /// Calcula a saída do neurônio dado um conjunto de inputs.
    fn calculate_output(&mut self, inputs: &[f64]);

    /// Obtém uma referência aos pesos do neurônio.
    fn get_weights(&self) -> &[f64];

    /// Obtém o bias do neurônio.
    fn get_bias(&self) -> f64;

    /// Obtém a saída calculada do neurônio.
    fn get_output(&self) -> f64;

    /// Define os pesos do neurônio.
    fn set_weights(&mut self, weights: Vec<f64>);

    /// Define o bias do neurônio.
    fn set_bias(&mut self, bias: f64);

    fn activation_function(&self) -> ActivationFunction;

}

impl NeuronTrait for Neuron {
    fn new(input_len: usize, activation_function: ActivationFunction) -> Self {

        let weights: Vec<f64> = (0..input_len).map(|_| random_range(-1.0..=1.0)).collect();
        let bias: f64 = random_range(-1.0..=1.0);

        Neuron {
            weights,
            bias,
            output: 0.0,
            activation_function,
            position: (0.0, 0.0),
        }
    }

    fn new_clone(neuron: &Neuron) -> Self {
        neuron.clone()
    }

    fn new_seted(
        weights: Vec<f64>,
        bias: f64,
        activation_function: ActivationFunction,
    ) -> Self {
        Neuron {
            weights,
            bias,
            output: 0.0,
            activation_function,
            position: (0.0, 0.0),
        }
    }

    fn calculate_output(&mut self, inputs: &[f64]) {
        // Verifica se o número de inputs corresponde ao número de pesos.
        assert!(
            inputs.len() == self.weights.len(),
            "Número de inputs ({}) não corresponde ao número de pesos ({})",
            inputs.len(),
            self.weights.len()
        );

        // Calcula a soma ponderada dos inputs e pesos, adicionando o bias.
        let sum: f64 = inputs
            .iter()
            .zip(self.weights.iter())
            .map(|(&x, &w)| x * w)
            .sum::<f64>()
            + self.bias;

        // Aplica a função de ativação escolhida.
        self.output = match self.activation_function {
            ActivationFunction::Sigmoid => sigmoid(sum),
            ActivationFunction::Relu => relu(sum),
            ActivationFunction::Tanh => tanh(sum),
            ActivationFunction::Linear => linear(sum),
        };
    }

    fn get_weights(&self) -> &[f64] {
        &self.weights
    }

    fn get_bias(&self) -> f64 {
        self.bias
    }

    fn get_output(&self) -> f64 {
        self.output
    }

    fn set_weights(&mut self, weights: Vec<f64>) {
        assert!(
            weights.len() == self.weights.len(),
            "Novo conjunto de pesos ({}) deve ter o mesmo tamanho ({})",
            weights.len(),
            self.weights.len()
        );
        self.weights = weights;
    }

    fn set_bias(&mut self, bias: f64) {
        self.bias = bias;
    }

    fn activation_function(&self) -> ActivationFunction {
        self.activation_function
    }
}


impl Neuron {
    pub fn mutate_weight(&mut self, index: usize, delta: f64) {

        if let Some(weight) = self.weights.get_mut(index) {

            *weight += delta;

        }

    }

    pub fn mutate_bias(&mut self, delta: f64) {

        self.bias += delta;

    }

    pub fn print_Neuron(&self) {

        println!("Neuron: {:?}", self);

    }
}
/// Função de ativação Sigmoid.
fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// Função de ativação ReLU (Rectified Linear Unit).
fn relu(x: f64) -> f64 {
    if x > 0.0 {
        x
    } else {
        0.0
    }
}

/// Função de ativação Tanh (Tangente Hiperbólica).
fn tanh(x: f64) -> f64 {
    x.tanh()
}

/// Função de ativação Linear.
fn linear(x: f64) -> f64 {
    x
}
