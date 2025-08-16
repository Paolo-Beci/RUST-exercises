
use std::time::SystemTime;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug)]
struct File {
    name: String,
    modified: SystemTime,
    content: Vec<u8>,
}

#[derive(Debug)]
struct Dir {
    name: String,
    modified: SystemTime,
    children: Vec<Node>,
}

#[derive(Debug)]
enum Node {
    File(File),
    Dir(Dir),
}

// RISPOSTA DI TEORIA
// Rust deve conoscere a compile-time la dimensione di Node.
// Per calcolarla, dovrebbe conoscere la dimensione di left e right…
// Ma left e right sono di tipo Node → il calcolo diventa ricorsivo infinito → dimensione infinita → il compilatore si arrabbia.

#[derive(Debug)]
enum FSError {
    NotFound,
    NotADir,
    Duplicate,
    DirNotEmpty,
    PermissionDenied,
    GenericError(String),
}

// define lifetimes
struct MatchResult<'a> {
    q: &'a str, // matched query string
    path: String, // matched path
    node: &'a Node, // matched node
}

struct Filesystem {
    root: Node,
}

impl Filesystem {
    // create a new empty filesystem with a root dir
    // (name of the root dir is empty string: "")
    pub fn new() -> Self {
        let dir = Dir {
            name: "".to_string(),
            modified: SystemTime::now(),
            children: Vec::new(),
        };
        let root = Node::Dir(dir);
        Filesystem { root }
    }

    // create a new filesystem reading from disk all the structure under the given path
    // in the file content just write the firt 1k bytes of the file
    // return the root node of the filesystem
    // (implement this function at the end, after all the other methods, the only purpose is to take a look std::fs functions, use std::fs:read_dir)
    // pub fn from(path: &str) -> Self {
    //     unimplemented!()
    // }


    pub fn navigate_filesystem_mut(&mut self, path: &str) -> Result<&mut Node, FSError> {
        // Navigate through the filesystem structure
        let mut current_node = &mut self.root;
        
        for part in path.split('/').filter(|s| !s.is_empty()) {
            // Check if current node is a directory
            match current_node {
                Node::Dir(ref mut dir) => {
                    // Find the child with the matching name
                    let found = dir.children.iter_mut().find(|child| {
                        match child {
                            Node::Dir(child_dir) => child_dir.name == part,
                            Node::File(child_file) => child_file.name == part,
                        }
                    });
                    
                    match found {
                        Some(node) => current_node = node,
                        None => return Err(FSError::NotFound),
                    }
                },
                Node::File(_) => return Err(FSError::NotADir),
            }
        }

        return Ok(current_node)
    }

    // create a new directory in the filesystem under the given path
    // return a reference the created dir
    // possible errors: NotFound, path NotADir, Duplicate
    pub fn mkdir(&mut self, path: &str, name: &str) -> Result<&mut Dir, FSError> {
        let new_path = format!("{}/{}", path, name);
        match fs::create_dir_all(&new_path) {
            Ok(_) => {
                // Navigate through the filesystem structure
                let mut current_node = self.navigate_filesystem_mut(path)?;
                
                // Now current_node should point to the parent directory
                // Check if it's actually a directory and add the new directory
                match current_node {
                    Node::Dir(ref mut parent_dir) => {
                        // Check if directory already exists
                        let already_exists = parent_dir.children.iter().any(|child| {
                            match child {
                                Node::Dir(child_dir) => child_dir.name == name,
                                _ => false,
                            }
                        });
                        
                        if already_exists {
                            return Err(FSError::Duplicate);
                        }
                        
                        // Create new directory
                        let new_dir = Dir {
                            name: name.to_string(),
                            modified: SystemTime::now(),
                            children: Vec::new(),
                        };
                        
                        parent_dir.children.push(Node::Dir(new_dir));
                        
                        // Return reference to the newly created directory
                        if let Some(Node::Dir(ref mut created_dir)) = parent_dir.children.last_mut() {
                            println!("Directory created successfully!");
                            Ok(created_dir)
                        } else {
                            Err(FSError::GenericError("Failed to create directory".to_string()))
                        }
                    },
                    Node::File(_) => Err(FSError::NotADir),
                }
            },
            Err(e) => match e.kind() {
                std::io::ErrorKind::AlreadyExists => {
                    println!("Directory already exists.");
                    Err(FSError::Duplicate)
                },
                std::io::ErrorKind::PermissionDenied => {
                    println!("Permission denied.");
                    Err(FSError::PermissionDenied)
                },
                std::io::ErrorKind::NotFound => {
                    println!("Path not found.");
                    Err(FSError::NotFound)
                },
                _ => {
                    println!("An error occurred: {:?}", e);
                    Err(FSError::GenericError(format!("IO Error: {}", e)))
                }
            }
        }
    }

    // possible errors: NotFound, path is NotADir, Duplicate
    pub fn create_file(&mut self, path: &str, name: &str) -> Result<&mut File, FSError> {
        let file_path = format!("{}/{}", path, name);
        let path_obj = Path::new(path);
        
        if path_obj.is_dir() {
            match fs::metadata(&file_path) {
                Ok(_) => {
                    println!("File already exists.");
                    Err(FSError::Duplicate)
                },
                Err(_) => {
                    // Navigate to the parent directory 
                    let parent_node = self.navigate_filesystem_mut(path)?;

                    // Create new file
                    let newfile = File {
                        name: name.to_string(),
                        modified: SystemTime::now(),
                        content: Vec::new(),
                    };

                    // parent_node should be a directory, so we need to match on it
                    match parent_node {
                        Node::Dir(ref mut parent_dir) => {
                            parent_dir.children.push(Node::File(newfile));
                            
                            // Return reference to the newly created file
                            if let Some(Node::File(ref mut created_file)) = parent_dir.children.last_mut() {
                                println!("File created successfully!");
                                Ok(created_file)
                            } else {
                                Err(FSError::GenericError("Failed to create file".to_string()))
                            }
                        },
                        Node::File(_) => {
                            Err(FSError::NotADir)
                        }
                    }
                }
            }
        } else {
            println!("Path is not a directory.");
            Err(FSError::NotADir)
        }
    }

    // updated modification time of the file or the dir
    // possible errors: NotFound
    pub fn touch(&mut self, path: &str) -> Result<(), FSError> {
        // Navigate to the node 
        let node = self.navigate_filesystem_mut(path)?;

        match node {
            Node::File(ref mut file) => {
                file.modified = SystemTime::now();
                Ok(())
            },
            Node::Dir(ref mut dir) => {
                dir.modified = SystemTime::now();
                Ok(())
            }
        }
    }

    // remove a node from the filesystem and return it
    // if it's a dir, it must be empty
    // possible errors: NotFound, DirNotEmpty
    pub fn delete(&mut self, path: &str) -> Result<Node, FSError> {
        // Navigate to the node 
        let node = self.navigate_filesystem_mut(path)?;
        let path_obj = Path::new(path);

        match node {
            Node::File(ref mut file) => {
                let parent_node = self.navigate_filesystem_mut((path_obj).parent())?;
                parent_node.child.pop(node);
                return node
            },
            Node::Dir(ref mut dir) => {
                if dir.child.empty() {
                    let parent_node = self.navigate_filesystem_mut((path_obj).parent())?;
                    parent_node.child.pop(node);
                    return node
                } else {
                    return Err(FSError::DirNotEmpty)
                }
            }
        }
    }

    // get a reference to a node in the filesystem, given the path
    // pub fn get(&mut self, path: &str) -> Result<&Node, FSError> {
    //     unimplemented!()
    // }

    // get a mutable reference to a node in the filesystem, given the path
    // pub fn get_mut(&mut self, path: &str) -> Result<&mut Node, FSError> {
    //     unimplemented!()
    // }

    // search for a list of paths in the filesystem
    // qs is a list query strings with constraints
    // the constraints must be matched in or (it's returned any node matching at least one constraint)
    // constraint format: "type:pattern"
    // constraints:
    // - "type:dir" -> match only directories
    // - "type:file" -> match only files
    // - "name:value" -> match only nodes with the given name
    // - "partname:value" -> match only nodes with the given string in the name

    // pub fn find<'a>(&'a self, qs: &[&'a str]) -> Vec<MatchResult> {
    //     unimplemented!()
    // }


    // walk the filesystem, starting from the root, and call the closure for each node with its path
    // the first parameter of the closure is the path of the node, second is the node itself
    // pub fn walk(&self, f: impl Fn(&str, &Node)) {
    //     unimplemented!()
    // }
}

fn demo() {

    let mut fs = Filesystem::new();

    // create a directory structure, 10 dirs with a child dir and file each one
    for i in 0..10 {
        fs.mkdir("/", format!("dir{}", i).as_str()).unwrap();
        fs.mkdir(format!("/dir{}", i).as_str(), "child1").unwrap();
        fs.create_file(format!("/dir{}", i).as_str(), "file1").unwrap();
    }

    // println!("find /child2");
    // if let Ok(res) = fs.get("/dir2/child1") {
    //     match res {
    //         Node::Dir(d) => {
    //             d.name = "dir2 found".to_string();
    //         }
    //         // try to match all possible errros
    //         _ => {}
    //     }
    // } else {
    //     println!("not found");
    // }

    // // let's try with matches
    // let matches = fs.find(&["name:child1", "type:file"]);
    // for m in matches {
    //     match m.node {
    //         Node::File(f) => {
    //             // inspect content
    //         },
    //         Node::Dir(d) => {
    //             // inspect children
    //         },
    //         _ => {}
    //     }
    // }

    // // see note "riferimenti mutabili" in exercise text 
    // // now let's try to modify the filesystem using the found matches
    // // is it possible to do it? which error do you get from the compiler?
    // let matches = fs.find(&["/dir2/child1", "/dir3/child1"]);
    // for m in matches {
    //     let node = fs.get_mut(m.path).unwrap();
    //     match node {
    //         Node::File(f) => {
    //             // inspect content
    //         }
    //         _ => {}
    //     }
    // }
    
    // // how can you fix the previous code?
    // // suggestion: this code using paths which are not referenced by MatchResults should compile. Why?
    // // Therefore how can you use the paths returned in the MatchResults to modify the filesystem?
    // let paths = ["/dir1/child1", "/dir2/child1", "/dir3/child1"];
    // for p in paths {
    //     let n = fs.get_mut(p.as_str());
    // }


    // // now let's try to walk the filesystem
    // fs.walk(|path, node| {
    //     match node {
    //         Node::File(f) => {
    //             println!("file: {}", path);
    //         }
    //         Node::Dir(d) => {
    //             println!("dir: {}", path);
    //         }
    //     }
    // });

}

pub fn main_ex2() -> Result<(), Box<dyn std::error::Error>> { 
    Ok(demo())
}