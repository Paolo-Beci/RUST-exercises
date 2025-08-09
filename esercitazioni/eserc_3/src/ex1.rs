// find all subsequences of seq in s and return a vector of tuples containing the start position
// and the found subsequences as string slices
// ignore overlaps: if a subsequence is found, the search must continue from the next character
// missing lifetimes: the result string slices depend only from one input parameter, which one?

// suggestion: write a function find_sub(&str, &str) -> Option<(usize, &str)> that finds the first 
// subsequence in a string, you can use it in all the following functions

#[derive(Debug)]
struct DnaSpec {
    base: char,
    min: usize,
    max: usize,
}

fn parse_seq(seq: &str) -> Vec<DnaSpec> {
    seq.split(',').map(|part| {
        let base = part.chars().next().unwrap();
        let rest = &part[1..]; // es. "1-2"
        let mut split = rest.split('-');
        let min = split.next().unwrap().parse().unwrap();
        let max = split.next().unwrap().parse().unwrap();
        DnaSpec { base, min, max }
    }).collect()
}

fn match_at(s: &str, start: usize, specs: &[DnaSpec]) -> Option<usize> {
    // Per ogni posizione i nella stringa s, proviamo a matchare tutta la sequenza specificata da seq.
    // Se va bene, salviamo (i, &s[i..j]) e saltiamo a i + 1.
    let chars = s.as_bytes();
    let mut idx = start;

    for spec in specs {
        let mut count = 0;
        while idx + count < chars.len() && chars[idx + count] == spec.base as u8 {
            count += 1;
        }

        if count < spec.min {
            return None;
        }

        // prendiamo al massimo `max`
        let take = count.min(spec.max);
        idx += take;
    }

    Some(idx) // posizione finale
}

fn subsequences1<'a>(s: &'a str, seq: &'a str) -> Vec<(usize, &'a str)> {
    let specs = parse_seq(seq);
    let mut result = Vec::new();
    let mut i = 0;

    while i < s.len() {
        if let Some(end) = match_at(s, i, &specs) {
            result.push((i, &s[i..end]));
        }
        i += 1;
    }

    result
}

pub fn demo1() {
    let a = "AACGGTAACC".to_string();
    let seq = "A1-1,C2-4";

    for (off, sub) in subsequences1(&a, seq) {
        println!("Found subsequence at position {}: {}", off, sub);
    }
}

// Now we want to find different subsequences at the same time, seq is a vector of string slices with many subsequence to search
// For each subsequence find all the matches and to the results (there may be overlaps, ignore them), but in this way you can reuse the previous solution
// The result will contain: the start position in s, the found subsequence as string slice and the mached subsequence in seq
// Now the string slices in the rsult depend from two input parameters, which ones?
fn subsequences2<'a>(s: &'a str, seqs: &'a [&'a str]) -> Vec<(usize, &'a str, &'a str)> {
    let mut result = Vec::new();
    for &seq in seqs {
        for (off, sub) in subsequences1(&s, &seq) {
            result.push((off, seq, sub));
        }
    }

    result
}

pub fn demo2() {
    let a = "AACGGTAACC".to_string();
    let seqs = ["A1-1,C2-4", "G1-1,T2-4"];

    for (off, matched, sub) in subsequences2(&a, &seqs) {
        println!("Found subsequence {} at position {}: {}", matched, off, sub);
    }
}

// Now we want to do some DNA editing! Therefore we receive a mutable string and we'd like to return a vector of mutable string slices
// Follow this steps:
// 1. adjust the lifetimes without any implementation yet: does it compile?
// 2. try to implement the function: does it compile?
// 3. if it doesn't compile, try to understand why from the compiler errors and draw all the necessary lifetimes
// 4. Spoiler: basically it's not possibile to return more then one mutable reference to the same data
// 5. Try this workaround: return a vector of indexes (first solution) and let the caller extract the mutable references
// 7. (later in the course you will learn about smart pointers, which can be used to solve this kind of problems in a more elegant way)
fn subsequences3<'a>(s: &'a mut str, seq: &'a str) -> Vec<(usize, &'a str)> {
    // rimosso mut dal return
    let specs = parse_seq(seq);
    let mut v = Vec::new();
    let mut i = 0;

    while i < s.len() {
        if let Some(end) = match_at(s, i, &specs) {
            v.push((i, &s[i..end]));
        }
        i += 1;
    }

    v
}

pub fn demo3() {
    let mut a = "AACGGTAACC".to_string();
    let seq = "A1-1,C2-4";

    for (off, sub) in subsequences3(&mut a, seq) {
        println!("Found subsequence at position {}: {}", off, sub);
    }
}

// DNA strings may be very long and we can get a lot of matches.
// Therefore we want to process a subsequence as soon as we find it, without storing it in a vector
// A solution is to pass a closure to the function, which will be called for each match
// do you need to put lifetime annotations in the closure? why?
fn subsequence4<F>(s: &str, seq: &str, f: F)
where
    F: Fn(usize, &str),
{
    let specs = parse_seq(seq);
    let mut i = 0;

    while i < s.len() {
        if let Some(end) = match_at(s, i, &specs) {
            f(i, &s[i..end]);
        }
        i += 1;
    }
}

pub fn demo4() {
    let a = "AACGGTAACC".to_string();
    let seq = "A1-1,C2-4";

    subsequence4(&a, seq, |pos, sub| {
        println!("Found subsequence at position {}: {}", pos, sub);
    });
}

// Now let's define a struct SimpleDNAIter (add the required lifetimes), memorizing a DNA sequence and the subsequence to search
// Then we add a next() method to the struct, which will return the next subsequence found in the DNA sequence after each call
// The result of next() is a tuple, but it's wrapped in an Option, because a call to next() may find no more subsequences in the DNA sequence
// In order to implement it, you may add any other attribute to the struct (remember: the struct is stateful and after each call to next() you must start from the last position found)
// The struct may be used as shown in the demo_SimpleDNAIter() function
// This approach is similar to the previous one, but it's more flexible and it can be used in more complex scenarios. For example you may interrupt it
// at any time and resume it later

struct SimpleDNAIter<'a> {
    s: &'a str,
    seq: &'a str,
    current_pos: usize,
}

impl<'a> SimpleDNAIter<'a> {
    pub fn new(s: &'a str, seq: &'a str) -> SimpleDNAIter<'a> {
        SimpleDNAIter { s, seq, current_pos: 0 }
    }

    pub fn next(&mut self) -> Option<(usize, &'a str)> {
        let specs = parse_seq(self.seq);
        
        while self.current_pos < self.s.len() {
            if let Some(end) = match_at(self.s, self.current_pos, &specs) {
                let start = self.current_pos;
                let result = (start, &self.s[start..end]);
                self.current_pos += 1;
                return Some(result);
            }
            self.current_pos += 1;
        }
        
        None
    }
}

fn demo_SimpleDNAIter() {
    let mut dna_iter = SimpleDNAIter::new("ACGTACGTACGTACGT", "A1-1,C1-1");

    while let Some((pos, subseq)) = dna_iter.next() {
        println!("Found subsequence at position {}: {}", pos, subseq);
        // we can break and stop if we have found what we were looking for
    }
}

// finally we want to implement a real iterator, so that it can be used in a for loop and it may be combined we all the most common iterator methods
// The struct DNAIter is already defined, you have to implement the Iterator trait for it and add lifetimes
struct DNAIter<'a> {
    s: &'a str,
    seq: &'a str,
    current_pos: usize,
}

impl<'a> DNAIter<'a> {
    pub fn new(s: &'a str, seq: &'a str) -> DNAIter<'a> {
        DNAIter {
            s,
            seq,
            current_pos: 0,
        }
    }
}

impl<'a> Iterator for DNAIter<'a> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let specs = parse_seq(self.seq);
        
        while self.current_pos < self.s.len() {
            if let Some(end) = match_at(self.s, self.current_pos, &specs) {
                let start = self.current_pos;
                let result = (start, &self.s[start..end]);
                self.current_pos += 1;
                return Some(result);
            }
            self.current_pos += 1;
        }
        
        None
    }
}

fn demo_dna_iter() {
    let mut dna_iter = DNAIter::new("ACGTACGTAAACCCGTACGT", "A1-3,C1-2");

    // now you can combine it with all the iterator modifiers!!!
    dna_iter
        .filter(|(pos, sub)| sub.len() >= 5)
        .for_each(|(pos, sub)| {
            println!(
                "Found subsequence at least long 5 at position {}: {}",
                pos, sub
            )
        });
}

// now let's return an iterator without defining a struct, just using a closure
// the std lib of rust support you with the std::from_fn() function
// we supply a skeleton implementation, you have to fill the closure
fn subsequence5_iter<'a>(s: &'a str, seq: &'a str) -> impl Iterator<Item = (usize, &'a str)> {
    let mut pos = 0;
    // and any other necessary variable to remember the state
    std::iter::from_fn(move || {
        if pos < s.len() {
            if let Some((relative_pos, sub)) = find_sub(&s[pos..], seq) {
                let absolute_pos = pos + relative_pos;
                pos += 1; // move to next position
                Some((absolute_pos, sub))
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn find_sub<'a>(s: &'a str, seq: &'a str) -> Option<(usize, &'a str)> {
    let specs = parse_seq(seq);
    let mut i = 0;

    while i < s.len() {
        if let Some(end) = match_at(s, i, &specs) {
            return Some((i, &s[i..end]));
        }
        i += 1;
    }

    None
}

fn demo_dna_iter2() {
    subsequence5_iter("ACGTACGTAAACCGTACGT", "A1-3,C1-2")
        .filter(|(pos, sub)| sub.len() >= 5)
        .for_each(|(pos, sub)| {
            println!(
                "Found subsequence at least long 5 at position {}: {}",
                pos, sub
            )
        });
}


pub fn main_ex1() -> Result<String, Box<dyn std::error::Error>> { 
    demo1();
    demo2();
    demo3();
    demo4();
    demo_SimpleDNAIter();
    demo_dna_iter();
    demo_dna_iter2();

    return Ok("OK".to_string())
}