pub mod List1 {
    use std::mem;

    pub enum ListLink<T> {
        Cons(T, Box<ListLink<T>>),
        Nil,
    }

    pub struct List<T> {
        head: ListLink<T>,
    }

    impl<T> List<T> {
        pub fn new() -> Self {
            List { head: ListLink::Nil }
        }

        // insert a new element at the beginning of the list
        // you may encouter a problem with the borrow checker while trying to move self.head to a new variable
        // why? look at mem::replace for solving it
        pub fn push(&mut self, elem: T) {
            let new_node = ListLink::Cons(elem, Box::new(mem::replace(&mut self.head, ListLink::Nil)));
            self.head = new_node;
        }

        pub fn pop(&mut self) -> Option<T> {
            match mem::replace(&mut self.head, ListLink::Nil) {
                ListLink::Cons(elem, next) => {
                    self.head = *next;
                    Some(elem)
                }
                ListLink::Nil => None,
            }
        }

        // return a referece to the first element of the list
        pub fn peek(&self) -> Option<&T> {
            match &self.head {
                ListLink::Cons(elem, _) => Some(elem),    // some reiterates the match on the next element
                ListLink::Nil => None,
            }
        }

        // uncomment after having implemented the ListIter struct
        // return an interator over the list values
        //fn iter(&self) -> ListIter<T> {
        //    unimplemented!()
        //}

        // take the first n elements of the list and return a new list with them
        pub fn take(&mut self, n: usize) -> List<T> {
            unimplemented!()
        }
    }


    //struct ListIter {
    //    // implement the iterator trait for ListIter
    //}
    //
    //impl Iterator for ListIter {
    //    //type Item = ...
    //
    //    fn next(&mut self) -> Option<Self::Item> {
    //        unimplemented!()
    //    }
    //}

    // something that may be useful for the iterator implementation:
    // let a = Some(T);
    // let b = &a;
    // match b { Some(i) => ... } // here i is a reference to T

}


// pub mod List2 {

//     pub struct Node<T> {
//         elem: T,
//         next: NodeLink<T>,
//     }

//     type NodeLink<T> = Option<Box<Node<T>>>;

//     pub struct List<T> {
//         head: NodeLink<T>,
//     }

//     // for this implementattion, since we are using option, take a look at the take method in Option<T>.
//     // It allows to move the value of the option into another option and replace it with None
//     // let mut a = Some(5);
//     // let b = a.take(); // a is now None and b is Some(5)
//     impl<T> List<T> {
//         // same methods as List1
//     }
// }

// *****
// double linked list suggestion: use Rc, since we need more than one reference to the same node
// for mutating the list and changing the next and prev fields we also need to be able to mutate the node, therefore we can use RefCell

// how to access content of Rc<RefCell<T>>:
// es let a = Rc::new(RefCell::new(5));
// let mut x = (*a).borrow_mut();  // with (*a) we dereference the Rc, with (*a).borrow_mut() we get a mutable reference to the content of the RefCell
// *x = 6; // we can now change the content of the RefCell

// to take a value from a Rc (useful when popping a value from the list): usually it is not possible since it may be referenced elsewhere.
// if you can guarantee it's the only reference to the value  youu can use Rc::try_unwrap(a).unwrap().into_inner() to get the value
// it first takes out the value from the Rc, then it tries to unwrap the value from the Result, and finally it takes the inner value from the Result
// see here
// https://stackoverflow.com/questions/70404603/how-to-return-the-contents-of-an-rc

// other hint that may be useful: Option<T> has a default clone implementation which calls the clone of T. Therefore: 
// Some(T).clone() ->  Some(T.clone())
// None.clone() -> None

//  type NodeLink = Option<Rc<RefCell<DNode>>>; // we define a type alias for better readibility
// Example
//  type NodeBackLink = ... 

// struct DNode {
    // v: i32,
    // prev: NodeBackLink // here we can't put NodeLink to avoid a cycle reference, what do we use?
    // next: NodeLink 
// }

// struct DList {
    // head: NodeLink,
    // tail: NodeLink
// }

use List1::List;

fn main() {
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