use std::{collections::HashMap, cell::RefCell, borrow::BorrowMut};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Node<T> {
    pub value: T,
    pub x: f32,
    pub y: f32,
    displacement_x: f32,
    displacement_y: f32,
}

#[derive(Debug, Clone)]
pub struct Graph<T> {
    dimensions: (f32, f32),
    pub nodes: HashMap<T, RefCell<Node<T>>>,
    pub edges: HashMap<(T, T), String>,
    adjacencies: HashMap<T, Vec<(T, String)>>,
}

struct Matrix {}

impl Matrix {
    pub fn calculate_difference<T>(node_x: &Node<T>, node_y: &Node<T>) -> (f32, f32) {
        let delta_x = node_x.x - node_y.x;
        let delta_y = node_x.y - node_y.y;
        
        return (delta_x, delta_y);
    }

    pub fn calculate_norm(x: f32, y: f32) -> f32 {
        (x * x + y * y).sqrt()
    }
}

/** 
 * https://www.dbc.wroc.pl/Content/59604/01-Andrzejak.pdf
 *
 * todo:
 * - przerobić na wybór algorytmu
 * - dodać kilka algorytmów generowania multigrafu
 */
impl<T: Eq + std::hash::Hash + Clone> Graph<T> {
    pub fn new(
        data: Vec<[T; 2]>,
        dimensions: (f32, f32),
        iterations: usize,
    ) -> Graph<T> {
        let mut rng = rand::thread_rng();
        let mut graph = Graph {
            dimensions,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            adjacencies: HashMap::new(),
        };

        let mut edge_counter = 0;
        for i in 0..data.len() {
            let edge: String;
            if graph.edges.contains_key(&(data[i][0].clone(), data[i][1].clone())) {
                edge = graph.edges.get(&(data[i][0].clone(), data[i][1].clone())).unwrap().to_string();
            } else if graph.edges.contains_key(&(data[i][1].clone(), data[i][0].clone())) {
                edge = graph.edges.get(&(data[i][1].clone(), data[i][0].clone())).unwrap().to_string();
            } else {
                edge = format!("e{}", edge_counter);
                edge_counter += 1;
                graph.edges.insert((data[i][0].clone(), data[i][1].clone()), edge.clone());
            }

            if graph.adjacencies.contains_key(&data[i][0]) {
                graph.adjacencies.get_mut(&data[i][0]).unwrap().push((data[i][1].clone(), edge));
            } else {
                graph.adjacencies.insert(data[i][0].clone(), vec![(data[i][1].clone(), edge)]);
            }

            if !graph.nodes.contains_key(&data[i][0]) {
                graph.nodes.insert(data[i][0].clone(), RefCell::new(Node {
                    value: data[i][0].clone(),
                    x: rng.gen_range(0.0..graph.dimensions.0 as f32).floor(),
                    y: rng.gen_range(0.0..graph.dimensions.1 as f32).floor(),
                    displacement_x: 0.0,
                    displacement_y: 0.0,
                }));
            }

            if !graph.nodes.contains_key(&data[i][1]) {
                graph.nodes.insert(data[i][1].clone(), RefCell::new(Node {
                    value: data[i][1].clone(),
                    x: rng.gen_range(0.0..graph.dimensions.0 as f32).floor(),
                    y: rng.gen_range(0.0..graph.dimensions.1 as f32).floor(),
                    displacement_x: 0.0,
                    displacement_y: 0.0,
                }));
            }
        }

        let factor = ((graph.dimensions.0 * graph.dimensions.1) as f32 / graph.nodes.len() as f32).sqrt();
        let mut temperature = (graph.dimensions.0 as f32).min(graph.dimensions.1 as f32) / 100.0;
        let nodes_len = graph.nodes.len();

        // https://dcc.fceia.unr.edu.ar/sites/default/files/uploads/materias/fruchterman.pdf
        for _iter in 0..iterations {
            // repulsion
            for node_x in graph.nodes.values() {
                let mut node_x = node_x.borrow_mut();
                node_x.displacement_x = 0.0;
                node_x.displacement_y = 0.0;
                for node_y in graph.nodes.values() {
                    let node_y = node_y.try_borrow();
                    if node_y.is_err() {
                        continue;
                    }

                    let node_y = node_y.unwrap();
                    if node_x.value == node_y.value {
                        continue;
                    }

                    let (delta_x, delta_y) = Matrix::calculate_difference(&(*node_x), &(*node_y));
                    let distance = Matrix::calculate_norm(delta_x, delta_y);

                    let displacement_x;
                    let displacement_y;

                    if distance > 0.0 {
                        displacement_x = (delta_x / distance) * ((-1.0 * factor) * factor / distance);
                        displacement_y = (delta_y / distance) * ((-1.0 * factor) * factor / distance);
                    } else {
                        displacement_x = ((-1.0 * factor) * factor) / 10.0;
                        displacement_y = ((-1.0 * factor) * factor) / 10.0;
                    }

                    node_x.displacement_x -= displacement_x;
                    node_x.displacement_y -= displacement_y;
                }
            }

            // attraction
            for (i, j) in graph.edges.keys() {
                let mut node_x = graph.nodes.get(&i).unwrap().borrow_mut();
                let mut node_y = graph.nodes.get(&j).unwrap().borrow_mut();
                let (delta_x, delta_y) = Matrix::calculate_difference(&(*node_x), &(*node_y));
                let distance = Matrix::calculate_norm(delta_x, delta_y);

                let displacement_x;
                let displacement_y;

                if distance > 0.0 {
                    displacement_x = (delta_x / distance) * (distance * distance / factor);
                    displacement_y = (delta_y / distance) * (distance * distance / factor);
                } else {
                    continue;
                }

                node_x.displacement_x -= displacement_x;
                node_x.displacement_y -= displacement_y;
                node_y.displacement_x += displacement_x;
                node_y.displacement_y += displacement_y;
            }

            for i in 0..nodes_len {
                let node = graph.nodes.keys().nth(i).unwrap();
                let mut node = graph.nodes.get(node).unwrap().borrow_mut();
                let norm = Matrix::calculate_norm(node.displacement_x, node.displacement_y);
                let displacement_x = (node.displacement_x / norm) * (norm.min(temperature));
                let displacement_y = (node.displacement_y / norm) * (norm.min(temperature));
                node.x += displacement_x;
                node.y += displacement_y;
            }

            if temperature > 0.001 {
                temperature = 0.001;
            } else {
                temperature *= 0.95;
            }
        }

        return graph;
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    #[test]
    fn it_works() {
        let mut rng = rand::thread_rng();
        let mut data = vec![];
        let max = 10;
        let iterations = 100;
        
        for i in 0..max {
            for j in 0..max {
                if i != j && rng.gen_bool(1.0 / 4.0) {
                // if i != j {
                    data.push([i, j]);
                }
            }
        }

        let graph = super::Graph::new(data, (500.0, 500.0), iterations);
        let mut xy = vec![];
        for node in graph.nodes.values() {
            let single = (node.borrow().x, node.borrow().y);
            if !xy.contains(&single) {
                xy.push(single);
            }
        }

        assert_eq!(xy.len(), graph.nodes.len());
        println!("{:#?}", graph.nodes);
    }

    #[test]
    fn sitegraph() {
        let file = std::fs::read_to_string("httpswwwfurgonetkapl.sitegraph").unwrap();
        let mut data = vec![];
        let iterations = 100;

        for line in file.lines() {
            let line = line.split_once(";").unwrap();
            data.push([line.0.to_string(), line.1.to_string()]);
        }

        println!("dane z sitegrapha {}", data.len());
        let graph = super::Graph::new(data, (500.0, 500.0), iterations);
        let mut xy = vec![];
        for node in graph.nodes.values() {
            let single = (node.borrow().x, node.borrow().y);
            if !xy.contains(&single) {
                xy.push(single);
            }
        }

        assert_eq!(xy.len(), graph.nodes.len());
        println!("{:#?}", graph.nodes);
    }
}
