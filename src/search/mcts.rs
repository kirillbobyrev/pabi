/// Parameters for MCTS search algorithm.
#[derive(Debug)]
struct Config {
    /// Number of search iterations to perform.
    iterations: u16,
    /// Number of threads to use.
    threads: u16,
    /// Exploration constant ($c_puct$ in the original paper).
    cpuct: f32,
    temperature: f32,
    /// Dirichlet distribution parameter for action selection at the root node.
    dirichlet_alpha: f32,
    /// Fraction of the dirichlet noise to add to the prior probabilities
    /// ($\epsilon$ in the original paper).
    dirichlet_exploration_weight: f32,
}

/// Implements AlphaZero's Monte Carlo Tree Search algorithm.
///
/// 1. Selection: Start from root node and select the most promising child node.
/// 2. Expansion: If the selected node is not a leaf node, expand it by adding a
///    new child node.
/// 3. Simulation: Run a simulation from the child node until a result is reached.
/// 4. Backpropagation: Update the nodes on the path from the root to the
///    selected node with the result.
fn search(iterations: usize) {
    for _ in 0..iterations {
        todo!()
    }
}

fn backup() {
    todo!()
}

fn simulate() {
    todo!()
}
