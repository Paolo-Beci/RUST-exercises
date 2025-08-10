use std::marker::PhantomData;
use std::rc::Rc;
use std::cell::RefCell;

pub enum NodeFunction {
    Generator(bool),
    Switch(bool),
    Light,
}

type NodeLink = Option<Rc<RefCell<Node>>>;

pub struct Node {
    name: String,
    function: NodeFunction,
    // which type for parent?
    // PhantomData is just a placeholder to let it compile
    parent: PhantomData<Node>,
    outs: [NodeLink; 2]
}

impl Node {
    // turn on or off the switch or the generator, if it's a light return an error 
    pub fn switch(&mut self) /*add return */  {
        unimplemented!()
    }
}


pub struct CircuitTree {
    // choose the right type for root and names
    root: PhantomData<Node>,
    names: PhantomData<Node>
}

impl CircuitTree {
    pub fn new() -> Self {
        unimplemented!()
    }

    // get a node by name
    pub fn get(&self, name: &str) -> NodeLink {
        unimplemented!()
    }

    // add a new node
    pub fn add(&mut self, parent_name: &str, node: Node) {
        unimplemented!()
    }

    // is the light on? Error if it's not a light
    pub fn light_status(&self, name: &str) -> Result<(), String> {
        unimplemented!();

    }

    pub fn turn_light_on(&self, name: &str) {
        unimplemented!();       
    }
}