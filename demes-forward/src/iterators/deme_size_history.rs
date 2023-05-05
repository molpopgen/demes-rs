use crate::DemesForwardError;
use crate::ForwardTime;

#[derive(Debug)]
pub struct DemeSizeHistory {
    deme: demes::Deme,
    forward_model_start_time: f64,
    current_epoch: usize,
    current_time: f64,
    end_time: f64,
}

impl DemeSizeHistory {
    fn resolve_times(
        deme: &demes::Deme,
        forward_model_start_time: f64,
        past: Option<demes::Time>,
        present: Option<demes::Time>,
    ) -> Result<(f64, f64), DemesForwardError> {
        let iterator_start_time = match past {
            Some(time) if time < deme.end_time() => {
                return Err(DemesForwardError::TimeError(format!(
                    "iteration start time {time} is more recent than deme's end time"
                )));
            }
            Some(time) if time < deme.start_time() => f64::from(time),
            _ => {
                let start_time: f64 = deme.start_time().into();
                if start_time.is_sign_positive() && start_time.is_infinite() {
                    forward_model_start_time
                } else {
                    f64::from(deme.start_time()) - 1.0
                }
            }
        };
        let iterator_end_time = match present {
            None => f64::from(deme.end_time()),
            Some(time) if time >= deme.start_time() => {
                return Err(DemesForwardError::TimeError(format!(
                    "iteration end time {time} is ancestral to deme start time"
                )));
            }
            Some(time) if time < deme.end_time() => f64::from(deme.end_time()),
            Some(time) => f64::from(time),
        };
        assert!(iterator_start_time.is_finite());
        assert!(iterator_end_time.is_finite());

        if iterator_start_time < iterator_end_time {
            Err(DemesForwardError::TimeError(format!(
                "invalid time interval: {iterator_start_time}, {iterator_end_time}"
            )))
        } else {
            Ok((iterator_start_time, iterator_end_time))
        }
    }

    pub fn new(
        deme: demes::Deme,
        forward_model_start_time: f64,
        past: Option<demes::Time>,
        present: Option<demes::Time>,
    ) -> Result<Self, DemesForwardError> {
        let (current_time, end_time) =
            Self::resolve_times(&deme, forward_model_start_time, past, present)?;

        let current_epoch = match deme.epochs().iter().enumerate().find(|(_, e)| {
            // TODO: is this what we want?
            current_time <= e.start_time() && current_time > e.end_time()
        }) {
            Some((index, _)) => index,
            None => deme.epochs().len(),
        };

        Ok(Self {
            deme,
            forward_model_start_time,
            current_epoch,
            current_time,
            end_time,
        })
    }
}

impl Iterator for DemeSizeHistory {
    type Item = Result<crate::DemeSizeAt, DemesForwardError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_time >= self.end_time && self.current_epoch < self.deme.num_epochs() {
            println!("{:?} {:?}", self.current_time, self.end_time);
            let epochs = self.deme.epochs();
            let size = match epochs[self.current_epoch].size_at(self.current_time) {
                Ok(size) => size,
                Err(e) => return Some(Err(DemesForwardError::from(e))),
            };
            let time = match demes::Time::try_from(self.current_time) {
                Ok(time) => time,
                Err(e) => return Some(Err(DemesForwardError::from(e))),
            };
            let forward_time: ForwardTime = (self.forward_model_start_time - self.current_time)
                .try_into()
                .ok()?;
            let rv = Some(Ok(crate::DemeSizeAt {
                time,
                forward_time,
                size: size.try_into().unwrap(),
            }));
            self.current_time -= 1.0;
            if self.current_time < epochs[self.current_epoch].end_time() {
                self.current_epoch += 1;
            }
            rv
        } else {
            None
        }
    }
}
