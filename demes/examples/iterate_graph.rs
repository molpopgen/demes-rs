fn main() {
    for input in std::env::args().skip(1) {
        println!("processing file {input}");
        let file = std::fs::File::open(input).unwrap();
        let graph = demes::load(file).unwrap();
        for deme in graph.demes() {
            println!("deme {}", deme.name());
            for epoch in deme.epochs() {
                println!("\tstart size = {}", epoch.start_size());
                println!("\tend size = {}", epoch.end_size());
                println!("\tstart time = {}", epoch.start_time());
                println!("\tend time = {}", epoch.end_time());
            }
        }
    }
}
