// Domanda List1

// la head in List1 è di tipo ListLink<T> e sta sempre nello stack, la head di List2
// invece è sempre nello heap essendo di tipo Option<Box<Node<T>>>. l'utlimo elemento
// nella List1 sarà un LinkList Nil, mentre in List2 non ci sarà nessun elemento
// allocato nello heap in quanto la lista termina senza un ulteriore allocazione
// che segnala la fine.

pub mod List2 {

    pub struct Node<T> {
        elem: T,
        next: NodeLink<T>,
    }

    type NodeLink<T> = Option<Box<Node<T>>>;

    pub struct List<T> {
        head: NodeLink<T>,
    }

    // for this implementattion, since we are using option, take a look at the take method in Option<T>.
    // It allows to move the value of the option into another option and replace it with None
    // let mut a = Some(5);
    // let b = a.take(); // a is now None and b is Some(5)
    impl<T> List<T> {
        pub fn new() -> Self {
            List { head: None }
        }

        pub fn push(&mut self, elem: T) {
            let new_node = Box::new(Node {
                elem,
                next: self.head.take(),
            });
            self.head = Some(new_node);
        }

        pub fn pop(&mut self) -> Option<T> {
            self.head.take().map(|boxed_node| {
                let Node { elem, next } = *boxed_node;
                self.head = next;
                elem
            })
        }

        pub fn popn(&mut self, n: usize) -> Option<T> {
            if n == 0 {
                return self.pop();
            }

            let mut current = &mut self.head;
            for _ in 0..n - 1 {
                match current {
                    Some(node) => {
                        current = &mut node.next;
                    }
                    None => return None,
                }
            }

            match current {
                Some(node) => {
                    let next = node.next.take();
                    match next {
                        Some(boxed_node) => {
                            let Node { elem, next } = *boxed_node;
                            node.next = next;
                            Some(elem)
                        }
                        None => None,
                    }
                }
                None => None,
            }
        }
        
        pub fn peek(&self) -> Option<&T> {
            let top = &self.head;
            match top {
                Some(node) => {
                    return Some(&node.elem)
                },
                None => None
            }
        }

        pub fn take(&mut self, n: usize) -> List<T> {
            let mut new_list = List::new();
            let mut new_tail = &mut new_list.head;

            for _ in 0..n {
                if let Some(mut boxed_node) = self.head.take() {
                    self.head = boxed_node.next.take();
                    *new_tail = Some(boxed_node);
                    
                    if let Some(ref mut tail_node) = new_tail {
                        new_tail = &mut tail_node.next;
                    }
                } else {
                    break;
                }
            }

            new_list
        }

    }
}

use List2::List;

pub fn main_ex1() {
    // Create a new list of integers using List1::List
    let mut list_of_ints: List<i32> = List::new();

    // Push elements onto the list
    list_of_ints.push(10);
    list_of_ints.push(20);
    list_of_ints.push(30);

    // Peek at the first element
    if let Some(first_elem) = list_of_ints.peek() {
        println!("First element: {}", first_elem); // Should print 20 (if peek returns a reference to the last pushed element)
    } else {
        println!("List is empty");
    }

    // Pop elements from the list
    if let Some(popped_elem) = list_of_ints.pop() {
        println!("Popped element: {}", popped_elem); // Should print 30
    } else {
        println!("List is empty");
    }

    // Pop more elements to verify
    while let Some(popped_elem) = list_of_ints.pop() {
        println!("Popped element: {}", popped_elem); // Should print 20 and then 10
    }

    if let Some(popped_elem) = list_of_ints.pop() {
        println!("Popped element: {}", popped_elem); // Should print List is empty
    } else {
        println!("List is empty");
    }
}