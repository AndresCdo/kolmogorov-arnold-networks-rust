use crate::network::layer::Layer;
use crate::data_structures::Matrix;
use crate::data_structures::Vector;
use std::io::{self, Read, Write};
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Network {
    pub layers: Vec<Layer>,
}

impl Network {
    pub fn new(layers: Vec<Layer>) -> Self {
        Network { layers }
    }

    pub fn forward(&self, input: Vector) -> Vector {
        let mut output = input;
        for layer in &self.layers {
            output = layer.forward(&output);
        }
        output
    }

    pub fn backward(&self, input: Vector, target: Vector) -> (Matrix, Vector) {
        let mut delta = target.clone();
        let mut weight_gradients = Matrix::new(vec![]);
        for layer in self.layers.iter().rev() {
            let output = layer.forward(&input);
            let (layer_weight_gradients, layer_delta) = layer.backward(&input, &output, &delta);
            weight_gradients = layer_weight_gradients.add(&weight_gradients);
            delta = layer_delta;
        }
        (weight_gradients, delta)
    }

    pub fn update(&mut self, weight_gradients: &Matrix, delta: &Vector, learning_rate: f64) {
        for layer in &mut self.layers {
            layer.update(weight_gradients, delta, learning_rate);
        }
    }

    pub fn train(&mut self, input: Vector, target: Vector, learning_rate: f64) {
        self.forward(input.clone());
        let (weight_gradients, delta) = self.backward(input, target);
        self.update(&weight_gradients, &delta, learning_rate);
    }


    pub fn predict(&self, input: Vector) -> Vector {
      
        let mut output = input;
        for layer in &self.layers {
            output = layer.forward(&output);
        }
        output
    }



    pub fn loss(&self, input: Vector, target: Vector) -> f64 {
        let output = self.forward(input);
        let error = output.subtract(&target);
        error.elementwise_multiply(&error).sum()
    }

    pub fn accuracy(&self, input: Vector, target: Vector) -> f64 {
        let output = self.forward(input);
        let error = output.subtract(&target);
        let correct = error.elements.iter().filter(|&&x| x.abs() < 0.5).count() as f64;
        correct / target.len() as f64
    }

    pub fn evaluate(&self, inputs: &[Vector], targets: &[Vector]) -> (f64, f64) {
        let mut total_loss = 0.0;
        let mut total_accuracy = 0.0;
        for (input, target) in inputs.iter().zip(targets) {
            total_loss += self.loss(input.clone(), target.clone());
            total_accuracy += self.accuracy(input.clone(), target.clone());
        }
        (total_loss / inputs.len() as f64, total_accuracy / inputs.len() as f64)
    }

    pub fn train_epoch(&mut self, inputs: &[Vector], targets: &[Vector], learning_rate: f64) {
        for (input, target) in inputs.iter().zip(targets) {
            self.train(input.clone(), target.clone(), learning_rate);
        }
    }

    pub fn train_epochs(&mut self, inputs: &[Vector], targets: &[Vector], learning_rate: f64, epochs: usize) {
        for _ in 0..epochs {
            self.train_epoch(inputs, targets, learning_rate);
        }
    }

    pub fn save(&self, path: &str) {
        let mut file = File::create(path).unwrap();
        file.write_all(self.to_string().as_bytes()).unwrap();
    }

    pub fn load(path: &str) -> Network {
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        for line in contents.lines() {
            println!("{}", line);
        }

        Network::from_str(&contents)
    }

    pub fn train_until_convergence(&mut self, inputs: &[Vector], targets: &[Vector], learning_rate: f64, max_epochs: usize, tolerance: f64) {
        let mut epoch = 0;
        let mut prev_loss = f64::INFINITY;
        let mut loss = self.evaluate(inputs, targets).0;
        while (prev_loss - loss).abs() > tolerance && epoch < max_epochs {
            prev_loss = loss;
            self.train_epoch(inputs, targets, learning_rate);
            loss = self.evaluate(inputs, targets).0;
            epoch += 1;
        }
    }

    pub fn predict_batch(&self, inputs: &[Vector]) -> Vec<Vector> {
        inputs.iter().map(|input| self.predict(input.clone())).collect()
    }

    pub fn loss_batch(&self, inputs: &[Vector], targets: &[Vector]) -> f64 {
        let total_loss: f64 = inputs.iter().zip(targets).map(|(input, target)| self.loss(input.clone(), target.clone())).sum();
        total_loss / inputs.len() as f64
    }

    pub fn accuracy_batch(&self, inputs: &[Vector], targets: &[Vector]) -> f64 {
        let total_accuracy: f64 = inputs.iter().zip(targets).map(|(input, target)| self.accuracy(input.clone(), target.clone())).sum();
        total_accuracy / inputs.len() as f64
    }

    pub fn evaluate_batch(&self, inputs: &[Vector], targets: &[Vector]) -> (f64, f64) {
        (self.loss_batch(inputs, targets), self.accuracy_batch(inputs, targets))
    }

    pub fn train_minibatch(&mut self, inputs: &[Vector], targets: &[Vector], learning_rate: f64, batch_size: usize) {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let mut indices: Vec<usize> = (0..inputs.len()).collect();
        indices.shuffle(&mut rng);

        for i in (0..inputs.len()).step_by(batch_size) {
            let batch_indices = &indices[i..i + batch_size];
            let batch_inputs: Vec<Vector> = batch_indices.iter().map(|&i| inputs[i].clone()).collect();
            let batch_targets: Vec<Vector> = batch_indices.iter().map(|&i| targets[i].clone()).collect();
            for (input, target) in batch_inputs.iter().zip(batch_targets) {
                self.train(input.clone(), target.clone(), learning_rate);
            }
        }
    }

    pub fn train_minibatches(&mut self, inputs: &[Vector], targets: &[Vector], learning_rate: f64, batch_size: usize, epochs: usize) {
        for _ in 0..epochs {
            self.train_minibatch(inputs, targets, learning_rate, batch_size);
        }
    }

    pub fn train_minibatches_until_convergence(&mut self, inputs: &[Vector], targets: &[Vector], learning_rate: f64, batch_size: usize, max_epochs: usize, tolerance: f64) {
        let mut epoch = 0;
        let mut prev_loss = f64::INFINITY;
        let mut loss = self.evaluate_batch(inputs, targets).0;
        while (prev_loss - loss).abs() > tolerance && epoch < max_epochs {
            prev_loss = loss;
            self.train_minibatch(inputs, targets, learning_rate, batch_size);
            loss = self.evaluate_batch(inputs, targets).0;
            epoch += 1;
        }
    }

    pub fn train_minibatches_until_convergence_with_validation(&mut self, inputs: &[Vector], targets: &[Vector], validation_inputs: &[Vector], validation_targets: &[Vector], learning_rate: f64, batch_size: usize, max_epochs: usize, tolerance: f64) {
        let mut epoch = 0;
        let mut prev_loss = f64::INFINITY;
        let mut loss = self.evaluate_batch(inputs, targets).0;
        let mut validation_loss = self.evaluate_batch(validation_inputs, validation_targets).0;
        while (prev_loss - loss).abs() > tolerance && epoch < max_epochs {
            prev_loss = loss;
            self.train_minibatch(inputs, targets, learning_rate, batch_size);
            loss = self.evaluate_batch(inputs, targets).0;
            validation_loss = self.evaluate_batch(validation_inputs, validation_targets).0;
            epoch += 1;
        }
    }

    pub fn train_minibatches_until_convergence_with_validation_and_early_stopping(&mut self, inputs: &[Vector], targets: &[Vector], validation_inputs: &[Vector], validation_targets: &[Vector], learning_rate: f64, batch_size: usize, max_epochs: usize, tolerance: f64, patience: usize) {
        let mut epoch = 0;
        let mut prev_loss = f64::INFINITY;
        let mut loss = self.evaluate_batch(inputs, targets).0;
        let mut validation_loss = self.evaluate_batch(validation_inputs, validation_targets).0;
        let mut best_loss = validation_loss;
        let mut best_epoch = 0;
        let mut early_stopping = false;
        let mut patience_counter = 0;
        while (prev_loss - loss).abs() > tolerance && epoch < max_epochs && !early_stopping {
            prev_loss = loss;
            self.train_minibatch(inputs, targets, learning_rate, batch_size);
            loss = self.evaluate_batch(inputs, targets).0;
            validation_loss = self.evaluate_batch(validation_inputs, validation_targets).0;
            if validation_loss < best_loss {
                best_loss = validation_loss;
                best_epoch = epoch;
                patience_counter = 0;
            } else {
                patience_counter += 1;
                if patience_counter >= patience {
                    early_stopping = true;
                }
            }
            epoch += 1;
        }
    }

    pub fn from_str(s: &str) -> Network {
        let layers: Vec<Layer> = s.split("Layer").map(|s| Layer::from_str(s)).collect();
        Network::new(layers)
    }

    pub fn to_string(&self) -> String {
        self.layers.iter().map(|layer| layer.to_string()).collect::<Vec<String>>().join("\n")
    }
}
