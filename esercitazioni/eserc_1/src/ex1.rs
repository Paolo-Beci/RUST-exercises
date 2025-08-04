use clap::Parser;
#[derive(Parser, Debug)]
pub struct Args {
    pub slug_in: String,
}

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

pub fn main_ex1(slug_in: &str) -> Result<String, Box<dyn std::error::Error>> {
    let res = slugify(&slug_in);
    Ok(res)
}

// ----------------- TESTS -----------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conv_lettera_accentata() {
        assert_eq!(conv('à'), 'a');
        assert_eq!(conv('ç'), 'c');
        assert_eq!(conv('ø'), 'o');
    }

    #[test]
    fn test_conv_lettera_non_accentata() {
        assert_eq!(conv('a'), 'a');
        assert_eq!(conv('Z'), 'z'); // deve diventare minuscola
    }

    #[test]
    fn test_conv_lettera_non_ammessa_sconosciuta() {
        assert_eq!(conv('!'), '-');
        assert_eq!(conv('@'), '-');
    }

    #[test]
    fn test_conv_lettera_accentata_non_compressa() {
        assert_eq!(conv('ῶ'), '-'); // non presente in SUBS_I
    }

    #[test]
    fn test_slugify_parole_separate_da_spazio() {
        let s = "Questo è un test";
        assert_eq!(slugify(s), "questo-e-un-test");
    }

    #[test]
    fn test_slugify_con_caratteri_accentati() {
        let s = "Città già usate";
        assert_eq!(slugify(s), "citta-gia-usate");
    }

    #[test]
    fn test_slugify_stringa_vuota() {
        let s = "";
        assert_eq!(slugify(s), "");
    }

    #[test]
    fn test_slugify_con_spazi_consecutivi() {
        let s = "questo  è   un    test";
        assert_eq!(slugify(s), "questo-e-un-test");
    }

    #[test]
    fn test_slugify_caratteri_non_validi_consecutivi() {
        let s = "test!!!@@##slug";
        assert_eq!(slugify(s), "test-slug");
    }

    #[test]
    fn test_slugify_solo_caratteri_non_validi() {
        let s = "!@#$%^&*()";
        assert_eq!(slugify(s), "");
    }

    #[test]
    fn test_slugify_spazio_finale() {
        let s = "slug finale ";
        assert_eq!(slugify(s), "slug-finale");
    }

    #[test]
    fn test_slugify_caratteri_non_validi_finali() {
        let s = "slug???";
        assert_eq!(slugify(s), "slug");
    }
}
