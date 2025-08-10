use std::rc::Rc;
use std::cell::RefCell;

#[derive(PartialEq, Eq)]
pub enum NodeFunction {
    Generator(bool),
    Switch(bool),
    Light,
}

type NodeLink = Option<Rc<RefCell<Node>>>;

pub struct Node {
    name: String,
    function: NodeFunction,
    parent: Option<Rc<RefCell<Node>>>,
    outs: [NodeLink; 2],
}

impl Node {
    // turn on or off the switch or the generator, if it's a light return an error 
    pub fn switch(&mut self) -> Result<(), ()>  {
        match self.function {
            NodeFunction::Generator(mut status) => {
                if status == true {
                    status = false;
                } else {
                    status = true;
                }
                return Ok(());
            },
            NodeFunction::Switch(mut status) => {
                if status == true {
                    status = false;
                } else {
                    status = true;
                }
                return Ok(());
            },
            NodeFunction::Light => {
                return Err(())
            }
        }
    }
}

pub struct CircuitTree {
    // The root node of the circuit tree
    root: Option<Rc<RefCell<Node>>>,
    // Map from node names to their Rc<RefCell<Node>> for quick lookup
    names: std::collections::HashMap<String, Rc<RefCell<Node>>>,
}

impl CircuitTree {
    pub fn new() -> Self {
        CircuitTree {
            root: None,
            names: std::collections::HashMap::new(),
        }
    }

    // loads a circuit from file
    pub fn from_file(path: &str) -> Self {
        // TODO

        CircuitTree {
            root: None,
            names: std::collections::HashMap::new(),
        }
    }

    // get a node by name
    pub fn get(&self, name: &str) -> NodeLink {
        self.names.get(name).cloned()
    }

    // add a new node
    pub fn add(&mut self, parent_name: &str, node: Node) {
        let node_rc = Rc::new(RefCell::new(node));

        if parent_name == "-" {
            // Set as root
            self.root = Some(node_rc.clone());
        } else {
            // Find parent
            if let Some(parent_rc) = self.names.get(parent_name) {
                let mut parent_ref = parent_rc.borrow_mut();
                // find first free output slot
                if parent_ref.outs[0].is_none() {
                    parent_ref.outs[0] = Some(node_rc.clone());
                } else if parent_ref.outs[1].is_none() {
                    parent_ref.outs[1] = Some(node_rc.clone());
                } else {
                    panic!("Parent node {} already has two children", parent_name);
                }
                // set parent link
                node_rc.borrow_mut().parent = Some(parent_rc.clone());
            } else {
                panic!("Parent node {} not found", parent_name);
            }
        }

        // Store in lookup map
        self.names.insert(node_rc.borrow().name.clone(), node_rc.clone());
    }

    // is the light on? Error if it's not a light
    pub fn light_status(&self, name: &str) -> Result<bool, String> {
        if let Some(light_node_rc) = self.names.get(name) {
            let light_node = light_node_rc.borrow();
            if light_node.function != NodeFunction::Light {
                Err("not a light".to_string())
            } else {
                if let Some(node_switch_rc) = &light_node.parent {
                    let node_switch = node_switch_rc.borrow();
                    match node_switch.function {
                        NodeFunction::Generator(status) | NodeFunction::Switch(status) => {
                            return Ok(status);
                        }
                        NodeFunction::Light => {
                            return Err("parent is not a switch or generator".to_string());
                        }
                    }
                } else {
                    return Err("no parent switch".to_string());
                }
            }
        } else {
            Err("node not found".to_string())
        }
    }

    pub fn turn_light_on(&self, name: &str) -> Result<bool, String> {
        if let Some(light_node_rc) = self.names.get(name) {
            let light_node = light_node_rc.borrow();
            if light_node.function != NodeFunction::Light {
                Err("not a light".to_string())
            } else {
                if let Some(node_switch_rc) = &light_node.parent {
                    let node_switch = node_switch_rc.borrow();
                    match node_switch.function {
                        NodeFunction::Generator(mut status) | NodeFunction::Switch(mut status) => {
                            status = true;
                            return Ok(status);
                        }
                        NodeFunction::Light => {
                            return Err("parent is not a switch or generator".to_string());
                        }
                    }
                } else {
                    return Err("no parent switch".to_string());
                }
            }
        } else {
            Err("node not found".to_string())
        }
    }
}

pub fn main_ex2() {}


// ------------------- TESTS ---------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn build_sample_circuit() -> CircuitTree {
        let mut tree = CircuitTree::new();

        tree.add("-", Node {
            name: "gen1".to_string(),
            function: NodeFunction::Generator(true),
            parent: None,
            outs: [None, None],
        });

        tree.add("gen1", Node {
            name: "sw01".to_string(),
            function: NodeFunction::Switch(false),
            parent: None,
            outs: [None, None],
        });

        tree.add("sw01", Node {
            name: "l01".to_string(),
            function: NodeFunction::Light,
            parent: None,
            outs: [None, None],
        });

        tree
    }

    #[test]
    fn test_add_and_get_node() {
        let tree = build_sample_circuit();
        assert!(tree.get("sw01").is_some());
        assert!(tree.get("missing").is_none());
    }

    #[test]
    fn test_light_status_off_initially() {
        let tree = build_sample_circuit();
        let status = tree.light_status("l01");
        assert!(status.is_ok());
        assert_eq!(status.unwrap(), false);
    }

    #[test]
    fn test_turn_light_on() {
        let tree = build_sample_circuit();
        let _ = tree.turn_light_on("l01");
        let status = tree.light_status("l01").unwrap();
        assert_eq!(status, true);
    }

    #[test]
    #[should_panic(expected = "not a light")]
    fn test_light_status_on_non_light_panics() {
        let tree = build_sample_circuit();
        let _ = tree.light_status("sw01").unwrap();
    }

    #[test]
    fn test_switch_toggle() {
        let mut node = Node {
            name: "sw01".to_string(),
            function: NodeFunction::Switch(false),
            parent: None,
            outs: [None, None],
        };
        assert!(node.switch().is_ok());
        if let NodeFunction::Switch(status) = node.function {
            assert!(status);
        } else {
            panic!("Wrong function type");
        }
    }
}