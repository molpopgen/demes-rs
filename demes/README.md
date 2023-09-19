[rust](https://www.rustlang.org) implementation of the [demes](https://popsim-consortium.github.io/demes-spec-docs/main/introduction.html#sec-intro) specification.

## Citation

If you use `demes` for your research, please cite:

```
Gower, G., A. P. Ragsdale, G. Bisschop, R. N. Gutenkunst, M. Hartfield, E. Noskova, S. Schiffels, T. J. Struck, J. Kelleher, K. R. Thornton (2022) Demes: a standard format for demographic models. Genetics 222 (3):iyac131    
```

[DOI](https://doi.org/10.1093/genetics/iyac131) for the paper.


## Example

This example reads in models from files.
The models are in `YAML` format.
After reading, we iterate over every deme in the model and
over every epoch of each deme.
The iteration order is past to present.

```rust
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
```

This example can be run from the root of the workspace:

```sh
cargo run --example iterate_graph -- demes/examples/jouganous.yaml
```

[Here](https://github.com/molpopgen/demes-rs/blob/main/demes/examples/iterate_graph_detail.rs) is a richer example.
To run it:

```sh
cargo run --example iterate_graph_detail -- demes/examples/jouganous.yaml
```

## Change log

See [here](https://github.com/molpopgen/demes-rs/blob/main/demes/CHANGELOG.md).
