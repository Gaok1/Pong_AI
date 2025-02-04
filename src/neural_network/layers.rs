use serde::{Deserialize, Serialize};

use crate::neural_network::neuron::{ActivationFunction, Neuron, NeuronTrait};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub neurons: Vec<Neuron>,
}

impl Layer {
    pub fn new(neurons_len: usize, input_len: usize, activation_function: ActivationFunction) -> Self {
        let neurons: Vec<Neuron> = (0..neurons_len)
            .map(|_| Neuron::new(input_len, activation_function))
            .collect();
        Layer { neurons }
    }

    pub fn feed(&mut self, input: &[f64]) {
        for neuron in &mut self.neurons {
            neuron.calculate_output(input);
        }
    }

    pub fn get_outputs(&self) -> Vec<f64> {
        self.neurons.iter().map(|neuron| neuron.get_output()).collect()
    }

    pub fn mut_neurons(&mut self) -> &mut Vec<Neuron> {
        &mut self.neurons
    }
}
