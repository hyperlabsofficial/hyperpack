use std::collections::{HashSet, HashMap};
use regex::Regex;

// Represents a node in the dependency graph
#[derive(Debug, Clone)]
struct Node {
    id: String,
    dependencies: HashSet<String>,
}

impl Node {
    // Creates a new node with the given ID
    fn new(id: &str) -> Self {
        Node {
            id: id.to_string(),
            dependencies: HashSet::new(),
        }
    }

    // Adds a dependency to this node
    fn add_dependency(&mut self, dependency: &str) {
        self.dependencies.insert(dependency.to_string());
    }

    // Checks if the node has a valid ID
    fn has_valid_id(&self) -> bool {
        let re = Regex::new(r"^[A-Za-z0-9_]+$").unwrap();
        re.is_match(&self.id)
    }
}

// Tree Shaker algorithm to remove unused nodes
fn tree_shaker(nodes: &HashMap<String, Node>, entry_points: &[&str]) -> HashSet<String> {
    let mut reachable = HashSet::new(); // Set to track reachable nodes
    let mut to_visit = entry_points.iter().map(|&id| id.to_string()).collect::<Vec<_>>(); // Nodes to visit

    while let Some(id) = to_visit.pop() {
        if reachable.insert(id.clone()) { // Mark the node as reachable
            if let Some(node) = nodes.get(&id) {
                for dep in &node.dependencies { // Check each dependency
                    if !reachable.contains(dep) { // If dependency is not already reachable
                        to_visit.push(dep.clone()); // Add to visit list
                    }
                }
            }
        }
    }

    reachable
}

// Removes unreachable nodes from the graph
fn remove_unreachable_nodes(nodes: &HashMap<String, Node>, reachable: &HashSet<String>) -> HashMap<String, Node> {
    nodes.iter()
        .filter_map(|(id, node)| {
            if reachable.contains(id) {
                Some((id.clone(), node.clone()))
            } else {
                None
            }
        })
        .collect()
}

// Detects cycles in the graph
fn detect_cycles(nodes: &HashMap<String, Node>) -> HashSet<String> {
    let mut visited = HashSet::new();
    let mut stack = HashSet::new();
    let mut cycles = HashSet::new();

    fn visit(
        node_id: &str,
        nodes: &HashMap<String, Node>,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
        cycles: &mut HashSet<String>,
    ) {
        if stack.contains(node_id) {
            cycles.insert(node_id.to_string());
            return;
        }
        if visited.contains(node_id) {
            return;
        }

        visited.insert(node_id.to_string());
        stack.insert(node_id.to_string());

        if let Some(node) = nodes.get(node_id) {
            for dep in &node.dependencies {
                visit(dep, nodes, visited, stack, cycles);
            }
        }

        stack.remove(node_id);
    }

    for node_id in nodes.keys() {
        if !visited.contains(node_id) {
            visit(node_id, nodes, &mut visited, &mut stack, &mut cycles);
        }
    }

    cycles
}

// Prints the nodes in a given graph
fn print_graph(nodes: &HashMap<String, Node>) {
    for (id, node) in nodes {
        println!("Node ID: {}", id);
        println!("Dependencies: {:?}", node.dependencies);
        println!("Valid ID: {}", node.has_valid_id());
    }
}

fn main() {
    let mut nodes = HashMap::new();

    // Create some example nodes
    let mut node1 = Node::new("Node1");
    node1.add_dependency("Node2");
    nodes.insert("Node1".to_string(), node1);

    let mut node2 = Node::new("Node2");
    node2.add_dependency("Node3");
    nodes.insert("Node2".to_string(), node2);

    let mut node3 = Node::new("Node3");
    node3.add_dependency("Node1"); // This creates a cycle
    nodes.insert("Node3".to_string(), node3);

    let entry_points = ["Node1"];
    let reachable_nodes = tree_shaker(&nodes, &entry_points);

    println!("Reachable nodes:");
    for id in &reachable_nodes {
        println!("{}", id);
    }

    let pruned_nodes = remove_unreachable_nodes(&nodes, &reachable_nodes);

    println!("\nPruned Graph:");
    print_graph(&pruned_nodes);

    let cycles = detect_cycles(&nodes);
    if !cycles.is_empty() {
        println!("\nDetected Cycles:");
        for cycle in &cycles {
            println!("{}", cycle);
        }
    } else {
        println!("\nNo cycles detected.");
    }
}