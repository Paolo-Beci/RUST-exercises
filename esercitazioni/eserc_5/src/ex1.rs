use itertools::Itertools;

struct Permutations {
    vec: Vec<(Vec<i32>, Vec<char>)>
}

impl Permutations {
    fn new() -> Self {
        return Permutations { vec: Vec::new() }
    }

    fn create_permutations(&mut self, numbers: Vec<i32>) -> &mut Permutations {
        let symbols = ['+', '-', '/', '*'];
        for nums in numbers.into_iter().permutations(5) {
            for sym_perm in symbols.iter().permutations(4) {
                let sym_chars: Vec<char> = sym_perm.into_iter().cloned().collect();
                self.vec.push((nums.clone(), sym_chars));
            }
        }
        return self
    }

    fn find_match(self) -> Option<(Vec<i32>, Vec<char>)>  {
        self.vec.into_iter().find_map(|(nums, ops)| {
            if nums.is_empty() || ops.is_empty() || nums.len() < ops.len() + 1 {
                return None;
            }
            
            let mut result = nums[0];
            
            for (i, op) in ops.iter().enumerate() {
                let next = nums[i + 1];
                
                match op {
                    '+' => result += next,
                    '-' => result -= next,
                    '*' => result *= next,
                    '/' => {
                        if next == 0 { return None; } // evita divisione per zero
                        result /= next;
                    }
                    _ => panic!("Operatore non valido"),
                };
            }
            
            if result == 10 {
                Some((nums, ops))
            } else {
                None
            }
        })
    }
}

// Parallelizzazione del lavoro
pub fn main_ex1() -> Result<String, Box<dyn std::error::Error>> {
    let mut ex = Permutations::new();
    ex.create_permutations(vec![1,2,3,4,5]);
    let res = ex.find_match();

    let msg = match res {
        Some((nums, ops)) => format!("OK, {:?}, {:?}", nums, ops),
        None => "No match found".to_string(),
    };

    Ok(msg)
}
