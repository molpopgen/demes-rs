pub struct ForwardGraph {
    graph: demes::Graph,
}

impl ForwardGraph {
    pub fn new(
        graph: demes::Graph,
        rounding: Option<demes::RoundTimeToInteger>,
    ) -> Result<Self, demes::DemesError> {
        Ok(Self { graph })
    }
}

#[cfg(test)]
mod graph_tests {
    use super::*;

    fn two_epoch_model() -> demes::Graph {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 200
      end_time: 50
    - start_size: 100
";
        demes::loads(yaml).unwrap()
    }

    fn two_epoch_model_invalid_conversion_to_generations() -> demes::Graph {
        let yaml = "
time_units: years
description:
  50/1000 = 0.05, rounds to zero.
  Thus, the second epoch has length zero.
generation_time: 1000.0
demes:
 - name: A
   epochs:
    - start_size: 200
      end_time: 50
    - start_size: 100
";
        demes::loads(yaml).unwrap()
    }

    #[test]
    fn initialize_graph() {
        let demes_graph = two_epoch_model();
        let graph = ForwardGraph::new(demes_graph, None).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_conversion_error() {
        let demes_graph = two_epoch_model_invalid_conversion_to_generations();
        let g = demes_graph
            .to_integer_generations(demes::RoundTimeToInteger::F64)
            .unwrap();
        for d in g.demes() {
            for e in d.start_times() {
                println!("{}", e);
            }
            for e in d.end_times() {
                println!("{}", e);
            }
        }
        // let graph = ForwardGraph::new(demes_graph, None).unwrap();
    }
}
