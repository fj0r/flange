use std::collections::HashMap;

fn main () {
    let mut x = HashMap::new();
    for i in 1..10000 {
        x.insert(i.to_string(), i.to_string());

    }
    println!("{:?}", &x.keys().len());

}
