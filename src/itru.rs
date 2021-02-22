use std::collections::HashMap;

pub fn insert_tag(seq: &str, insert: &str) -> String {
    let trans = translate_dna(insert);

    seq.replace("*", &trans).to_uppercase()
}

fn translate_dna(insert: &str) -> String {
    let libs = get_dna_libs();
    let dna = String::from(insert.to_uppercase());

    let mut translate = String::new();

    dna.chars()
        .for_each(|b| {
            let base = libs.get(&b).unwrap();
            translate.push(*base);
        });

    translate
}

fn get_dna_libs() -> HashMap<char, char> {
    let dna = String::from("AGTC");
    let comp = String::from("TCAG");

    let mut trans = HashMap::new();

    dna.chars()
        .zip(comp.chars())
        .for_each(|(b, c)| {
            trans.insert(b,c);
        });
    
    trans
}