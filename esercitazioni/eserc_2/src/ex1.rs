
trait MySlug {
    fn is_slug(&self) -> bool;
    fn to_slug(&self) -> String;
}

impl<T: AsRef<str>> MySlug for T {
    fn is_slug(&self) -> bool {
        self.as_ref() == slugify(self.as_ref())
    }

    fn to_slug(&self) -> String {
        slugify(self.as_ref())
    }
}

// impl MySlug for String {
//     fn is_slug(&self) -> bool {
//         *self == slugify(self)
//     }

//     fn to_slug(&self) -> String {
//         slugify(self)
//     }
// }

// impl MySlug for &str {
//     fn is_slug(&self) -> bool {
//         *self == slugify(self)
//     }

//     fn to_slug(&self) -> String {
//         slugify(&self)
//     }
// }

pub fn slugify(s: &str) -> String {
    let mut result = String::new();
    let mut prev_was_dash = false;
    
    for c in s.chars() {
        let converted = conv(c);
        if converted == '-' {
            if !prev_was_dash && !result.is_empty() {
                result.push('-');
                prev_was_dash = true;
            }
        } else {
            result.push(converted);
            prev_was_dash = false;
        }
    }
    
    if result.ends_with('-') {
        result.pop();
    }
    
    result
}

fn conv(c: char) -> char {
    const SUBS_I : &str = "àáâäæãåāăąçćčđďèéêëēėęěğǵḧîïíīįìıİłḿñńǹňôöòóœøōõőṕŕřßśšşșťțûüùúūǘůűųẃẍÿýžźż";
    const SUBS_O: &str = "aaaaaaaaaacccddeeeeeeeegghiiiiiiiilmnnnnoooooooooprrsssssttuuuuuuuuuwxyyzzz";
    const NORMAL_LETTRS: &str = "abcdefghijklmnopqrstuvwxyz";

    if let Some(pos) = SUBS_I.chars().position(|a| a == c) {
        if let Some(res) = SUBS_O.chars().nth(pos) {
            let res_lower = res.to_ascii_lowercase();
            return res_lower
        } else {
            return '-'
        }
    } else {
        let cl = c.to_ascii_lowercase();
        if NORMAL_LETTRS.contains(cl) {
            return cl;
        } else {
            return '-';
        }
    }
}

// MySlug
pub fn main_ex1() -> Result<String, Box<dyn std::error::Error>> {
    let s1 = String::from("Hello String");
    let s2 = "hello-slice";

    println!("{}", s1.is_slug()); // false
    println!("{}", s2.is_slug()); // true

    let s3: String = s1.to_slug();
    let s4: String = s2.to_slug();

    // stampa: s3:hello-string s4:hello-slice
    let res = format!("s3:{} s4:{}", s3, s4);
    Ok(res)
}