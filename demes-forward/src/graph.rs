use crate::iterators::DemeSizeHistory;
use crate::square_matrix::SquareMatrix;
use crate::time::ModelTime;
use crate::CurrentSize;
use crate::DemeSizeAt;
use crate::DemesForwardError;
use crate::ForwardTime;

enum Generation {
    Parent,
    Child,
}

fn time_minus_1(time: demes::Time) -> demes::Time {
    demes::Time::try_from(f64::from(time) - 1.0).unwrap()
}

fn get_epoch_start_time_discrete_time_model(
    deme: &demes::Deme,
    epoch_index: usize,
) -> Result<demes::Time, DemesForwardError> {
    let epoch = deme.epochs().get(epoch_index).ok_or_else(|| {
        DemesForwardError::InternalError(format!(
            "could not obtain epoch {} from deme {}",
            epoch_index,
            deme.name()
        ))
    })?;
    Ok(time_minus_1(epoch.start_time()))
}

fn time_is_rounded(time: demes::Time, level: &str) -> Result<(), DemesForwardError> {
    let f = f64::from(time);
    if f.fract() == 0.0 || f.is_infinite() {
        Ok(())
    } else {
        Err(DemesForwardError::TimeError(format!(
            "{level} not rounded to integer: {time}",
        )))
    }
}

fn validate_model_times(graph: &demes::Graph) -> Result<(), DemesForwardError> {
    graph
        .demes()
        .iter()
        .try_for_each(|deme| time_is_rounded(deme.start_time(), "Deme start_time"))?;
    graph.demes().iter().try_for_each(|deme| {
        deme.epochs()
            .iter()
            .try_for_each(|epoch| time_is_rounded(epoch.end_time(), "Epoch end_time"))
    })?;
    graph
        .pulses()
        .iter()
        .try_for_each(|pulse| time_is_rounded(pulse.time(), "Pulse time"))?;
    graph.migrations().iter().try_for_each(|migration| {
        time_is_rounded(migration.start_time(), "Migration start_time")?;
        time_is_rounded(migration.end_time(), "Migration end_time")
    })?;
    Ok(())
}

// #[derive(Copy, Clone)]
struct SizeFunctionDetails {
    epoch_start_time: demes::Time,
    epoch_end_time: demes::Time,
    epoch_start_size: demes::DemeSize,
    epoch_end_size: demes::DemeSize,
    backwards_time: demes::Time,
}

impl SizeFunctionDetails {
    fn duration(&self) -> f64 {
        f64::from(self.epoch_start_time) - f64::from(self.epoch_end_time)
    }

    fn time_from_epoch_start(&self) -> f64 {
        f64::from(self.epoch_start_time) - f64::from(self.backwards_time)
    }
}

fn linear_size_change(details: SizeFunctionDetails) -> f64 {
    let duration = details.duration() + 1.0;
    let x = details.time_from_epoch_start() + 1.0;
    let size_diff = f64::from(details.epoch_end_size) - f64::from(details.epoch_start_size);
    (f64::from(details.epoch_start_size) + (x / duration) * size_diff).round()
}

fn exponential_size_change(details: SizeFunctionDetails) -> f64 {
    let duration = details.duration() + 1.0;
    let nt = f64::from(details.epoch_end_size).round();
    let n0 = f64::from(details.epoch_start_size).round();
    let growth_rate = (nt / n0).powf(1. / duration) - 1.;
    let x = details.time_from_epoch_start() + 1.0;
    (n0 * (1. + growth_rate).powf(x)).round()
}

fn apply_size_function(
    deme: &demes::Deme,
    epoch_index: usize,
    backwards_time: Option<demes::Time>,
) -> Result<Option<CurrentSize>, DemesForwardError> {
    match backwards_time {
        Some(btime) => {
            let epoch_start_time = get_epoch_start_time_discrete_time_model(deme, epoch_index)?;
            let current_epoch = match deme.get_epoch(epoch_index) {
                Some(epoch) => epoch,
                None => {
                    return Err(DemesForwardError::InternalError(format!(
                        "could not retrieve epoch {} from deme {}",
                        epoch_index,
                        deme.name()
                    )))
                }
            };

            let epoch_end_time = current_epoch.end_time();
            let epoch_start_size = current_epoch.start_size();
            let epoch_end_size = current_epoch.end_size();

            let size_function_details = match current_epoch.size_function() {
                demes::SizeFunction::Constant => return Ok(Some(epoch_start_size.try_into()?)),
                demes::SizeFunction::Linear => linear_size_change,
                demes::SizeFunction::Exponential => exponential_size_change,
                _ => unimplemented!("unimplemented size function variant"),
            };

            let size: f64 = size_function_details(SizeFunctionDetails {
                epoch_start_time,
                epoch_end_time,
                epoch_start_size,
                epoch_end_size,
                backwards_time: btime,
            });

            if !size.gt(&0.0) || !size.is_finite() {
                Err(DemesForwardError::InvalidDemeSize(size))
            } else {
                Ok(Some(CurrentSize::try_from(size)?))
            }
        }
        None => Ok(None),
    }
}

#[derive(Debug, Clone)]
struct Deme {
    deme: demes::Deme,
    status: DemeStatus,
    backwards_time: Option<demes::Time>,
    ancestors: Vec<usize>,
    proportions: Vec<demes::Proportion>,
}

#[derive(Debug, Clone)]
enum DemeStatus {
    /// Before the deme first appears.
    /// (Moving forwards in time.)
    Before,
    /// During the deme's Epochs
    During(usize),
    /// After the deme ceases to exist.
    /// (Moving forwards in time.)
    After,
}

impl Deme {
    fn new(deme: demes::Deme) -> Self {
        Self {
            deme,
            status: DemeStatus::Before,
            backwards_time: None,
            ancestors: vec![],
            proportions: vec![],
        }
    }

    fn is_extant(&self) -> bool {
        matches!(self.status, DemeStatus::During(_))
    }

    fn epoch_index_for_update(&self) -> usize {
        match self.status {
            DemeStatus::Before => 0,
            DemeStatus::During(x) => x,
            DemeStatus::After => self.deme.num_epochs(),
        }
    }

    // return None if !self.is_extant()
    fn current_size(&self) -> Result<Option<CurrentSize>, DemesForwardError> {
        match self.status {
            DemeStatus::During(epoch_index) => match self.deme.get_epoch(epoch_index) {
                Some(_) => apply_size_function(&self.deme, epoch_index, self.backwards_time),
                None => panic!("fatal error: epoch_index out of range"),
            },
            _ => Ok(None),
        }
    }

    fn ancestors(&self) -> &[usize] {
        &self.ancestors
    }

    fn proportions(&self) -> &[demes::Proportion] {
        &self.proportions
    }

    fn update(
        &mut self,
        time: demes::Time,
        update_ancestors: bool,
        deme_to_index: &std::collections::HashMap<String, usize>,
    ) -> Result<CurrentSize, DemesForwardError> {
        self.ancestors.clear();
        self.proportions.clear();
        let mut current_size = CurrentSize::try_from(0.0)?;
        if time < self.deme.start_time() {
            let i = self.epoch_index_for_update();

            // NOTE: by having enumerate BEFORE
            // skip, the j value is the offset
            // from .epoch()[0]!!!
            if let Some((j, _epoch)) = self
                .deme
                .epochs()
                .iter()
                .enumerate()
                .skip(i)
                .find(|index_epoch| time >= index_epoch.1.end_time())
            {
                self.status = DemeStatus::During(j);
                self.backwards_time = Some(time);
                if update_ancestors {
                    // NOTE: PR 272 introduced much stricter value
                    // checking in demes. The strict checks
                    // led to a regression here if we try
                    // to convert this value into demes::Time.
                    // (The value can get < 0.0.)
                    //
                    // TODO: investigate why we subtract 2 here.
                    // It seems "off" to me.
                    let generation_to_check_ancestors = f64::from(self.deme.start_time()) - 2.0;
                    if time > generation_to_check_ancestors {
                        for (name, proportion) in self
                            .deme
                            .ancestor_names()
                            .iter()
                            .zip(self.deme.proportions().iter())
                        {
                            let index = *deme_to_index.get(name).ok_or_else(|| {
                                DemesForwardError::InternalError(format!(
                                    "could not get deme {name} from deme_to_index map",
                                ))
                            })?;
                            self.ancestors.push(index);
                            self.proportions.push(*proportion);
                        }
                    }
                    //}
                }
                let deme_size = self.current_size()?;
                current_size = deme_size.ok_or_else(|| {
                    DemesForwardError::InternalError(format!(
                        "failed up update current size of deme {}",
                        self.deme.name(),
                    ))
                })?;
            } else {
                self.status = DemeStatus::After;
                self.backwards_time = None;
            }
        }
        Ok(current_size)
    }
}

fn update_demes(
    backwards_time: Option<demes::Time>,
    update_ancestors: bool,
    deme_to_index: &std::collections::HashMap<String, usize>,
    graph: &demes::Graph,
    demes: &mut Vec<Deme>,
    sizes: &mut Vec<CurrentSize>,
) -> Result<(), DemesForwardError> {
    match backwards_time {
        Some(time) => {
            if demes.is_empty() {
                sizes.clear();
                for deme in graph.demes().iter() {
                    demes.push(Deme::new(deme.clone()));
                    sizes.push(CurrentSize::try_from(0.0)?);
                }
            }

            demes.iter_mut().enumerate().try_for_each(
                |(i, deme)| -> Result<(), DemesForwardError> {
                    let size = deme.update(time, update_ancestors, deme_to_index)?;
                    sizes[i] = size;
                    Ok(())
                },
            )?;
        }
        None => demes.clear(),
    }
    Ok(())
}

/// Forward-time representation of a [`demes::Graph`].
#[derive(Debug, Clone)]
pub struct ForwardGraph {
    graph: demes::Graph,
    model_times: ModelTime,
    parent_demes: Vec<Deme>,
    child_demes: Vec<Deme>,
    last_time_updated: Option<ForwardTime>,
    deme_to_index: std::collections::HashMap<String, usize>,
    pulses: Vec<demes::Pulse>,
    migrations: Vec<demes::AsymmetricMigration>,
    ancestry_proportions: SquareMatrix,
    migration_matrix: SquareMatrix,
    cloning_rates: Vec<demes::CloningRate>,
    selfing_rates: Vec<demes::SelfingRate>,
    parental_deme_sizes: Vec<CurrentSize>,
    child_deme_sizes: Vec<CurrentSize>,
}

impl ForwardGraph {
    /// Constructor
    ///
    /// # Parameters
    ///
    /// * graph: a [`demes::Graph`].
    /// * burnin_time: Burn-in time for the model.
    pub fn new_discrete_time<F: Into<ForwardTime> + std::fmt::Debug + Copy>(
        graph: demes::Graph,
        burnin_time: F,
    ) -> Result<Self, crate::DemesForwardError> {
        if let Some((name, index)) = graph.has_non_integer_sizes() {
            let deme = graph.get_deme(name).unwrap();
            let epoch = deme.epochs()[index];
            for i in [f64::from(epoch.start_size()), f64::from(epoch.end_size())] {
                if i.is_finite() && i.fract() != 0.0 {
                    return Err(DemesForwardError::InvalidDemeSize(i));
                }
            }
        }
        let burnin_time = burnin_time.into();
        if !burnin_time.valid() {
            return Err(DemesForwardError::TimeError(format!(
                "invalid time value: {burnin_time:?}",
            )));
        }
        let graph = graph.into_integer_generations()?;

        validate_model_times(&graph)?;

        let model_times = ModelTime::new_from_graph(burnin_time, &graph)?;
        let child_demes = vec![];
        let parent_demes = vec![];
        let mut deme_to_index = std::collections::HashMap::default();
        for (i, deme) in graph.demes().iter().enumerate() {
            deme_to_index.insert(deme.name().to_string(), i);
        }
        let pulses = vec![];
        let ancestry_proportions = SquareMatrix::zeros(deme_to_index.len());
        let migration_matrix = SquareMatrix::zeros(deme_to_index.len());
        Ok(Self {
            graph,
            model_times,
            parent_demes,
            child_demes,
            last_time_updated: None,
            deme_to_index,
            pulses,
            migrations: vec![],
            ancestry_proportions,
            migration_matrix,
            cloning_rates: vec![],
            selfing_rates: vec![],
            parental_deme_sizes: vec![],
            child_deme_sizes: vec![],
        })
    }

    fn update_pulses(&mut self, backwards_time: Option<demes::Time>) {
        self.pulses.clear();
        match backwards_time {
            None => (),
            Some(time) => self.graph.pulses().iter().for_each(|pulse| {
                if !(time > pulse.time() || time < pulse.time()) {
                    self.pulses.push(pulse.clone());
                }
            }),
        }
    }

    // NOTE: performance here is poop emoji.
    // Migrations tend to occur over long epochs
    // and we are figuring this out from scratch each time.
    // This may not be a "big deal" so this note is here in
    // case of future profiling.
    // Alternative:
    // * Maintain a vec of current epochs that are (index, Mig)
    // * Remove epochs no longer needed
    // * Only add epochs not already there.
    fn update_migrations(&mut self, backwards_time: Option<demes::Time>) {
        self.migrations.clear();
        match backwards_time {
            None => (),
            Some(time) => self.graph.migrations().iter().for_each(|migration| {
                if time >= migration.end_time() && time < migration.start_time() {
                    self.migrations.push(migration.clone());
                }
            }),
        }
    }

    fn initialize_ancestry_proportions(&mut self) {
        self.ancestry_proportions.fill(0.0);
        for (i, deme) in self.child_demes.iter().enumerate() {
            if deme.is_extant() {
                if deme.ancestors().is_empty() {
                    self.ancestry_proportions.set(i, i, 1.0);
                } else {
                    deme.ancestors()
                        .iter()
                        .zip(deme.proportions().iter())
                        .for_each(|(ancestor, proportion)| {
                            self.ancestry_proportions
                                .set(i, *ancestor, f64::from(*proportion));
                        });
                }
            }
        }
    }

    fn update_ancestry_proportions_from_pulses(
        &mut self,
        parental_generation_time: ForwardTime,
    ) -> Result<(), DemesForwardError> {
        let mut sources = vec![];
        let mut proportions = vec![];
        for pulse in &self.pulses {
            sources.clear();
            proportions.clear();
            let dest = *self.deme_to_index.get(pulse.dest()).ok_or_else(|| {
                DemesForwardError::InternalError(format!(
                    "could not fetch {} from deme_to_index map",
                    pulse.dest()
                ))
            })?;

            {
                let child_deme = self.child_demes.get(dest).ok_or_else(|| {
                    DemesForwardError::InternalError(format!(
                        "destination deme index {dest} is invalid",
                    ))
                })?;
                if !child_deme.is_extant() {
                    return Err(DemesForwardError::InternalError(format!(
                    "pulse dest deme {} is extinct at (forward) time {:?}, which is backwards time {:?}",
                    self.graph.demes()[dest].name(),
                    parental_generation_time, self.model_times.convert(parental_generation_time),
                )));
                }
            }

            let mut sum = 0.0;

            for (source, proportion) in pulse.sources().iter().zip(pulse.proportions().iter()) {
                let index: usize = *self.deme_to_index.get(source).ok_or_else(|| {
                    DemesForwardError::InternalError(format!(
                        "could not fetch deme {source} from deme_to_index map",
                    ))
                })?;
                let parent_deme = self.parent_demes.get(index).ok_or_else(|| {
                    DemesForwardError::InternalError(format!("deme index {index} is invalid",))
                })?;
                if !parent_deme.is_extant() {
                    return Err(DemesForwardError::InternalError(format!(
                            "pulse source deme {} is extinct at (forward) time {:?}, which is backwards time {:?},",
                            self.graph.demes()[index].name(),
                            parental_generation_time, self.model_times.convert(parental_generation_time),
                        )));
                }
                sources.push(index);
                let p = f64::from(*proportion);
                sum += p;
                proportions.push(p);
            }

            self.ancestry_proportions
                .row_mut(dest)
                .iter_mut()
                .for_each(|v| *v *= 1. - sum);
            sources
                .iter()
                .zip(proportions.iter())
                .for_each(|(source, proportion)| {
                    self.ancestry_proportions.set(dest, *source, *proportion);
                });
        }
        Ok(())
    }

    fn update_migration_matrix(
        &mut self,
        parental_generation_time: ForwardTime,
    ) -> Result<(), DemesForwardError> {
        self.migration_matrix.fill(0.0);
        for migration in &self.migrations {
            let source = self.deme_to_index.get(migration.source()).ok_or_else(|| {
                DemesForwardError::InternalError(format!(
                    "could not fetch deme {} from deme_to_index map",
                    migration.source()
                ))
            })?;
            let dest = self.deme_to_index.get(migration.dest()).ok_or_else(|| {
                DemesForwardError::InternalError(format!(
                    "could not fetch deme {} from deme_to_index map",
                    migration.dest()
                ))
            })?;
            if !self.parent_demes[*source].is_extant() {
                // This block was changed to return Ok(())
                // in PR238.
                // The reason is that a deme may be first appearing
                // now and immediately involved in migration.
                // However, the fact that it doesn't exist as parents
                // to itself should not be an error, provided that
                // it does have ancestry from elsewhere.
                return Ok(());
                //return Err(DemesForwardError::InternalError(format!(
                //    "migration source deme {} is extinct at (forward) time {:?}, which is backwards time {:?}",
                //    self.graph.demes()[*source].name(),
                //    parental_generation_time,
                //    self.model_times.convert(parental_generation_time),
                //)));
            }
            if !self.child_demes[*dest].is_extant() {
                return Err(DemesForwardError::InternalError(format!(
                    "migration dest deme {} is extinct at forward time {:?}, which is backwards time {:?}",
                    self.graph.demes()[*dest].name(),
                    parental_generation_time,
                    self.model_times.convert(parental_generation_time),
                )));
            }
            self.migration_matrix
                .set(*dest, *source, migration.rate().into());
        }
        Ok(())
    }

    // NOTE: this doesn't Err b/c:
    // It is called after update_migration_matrix, which
    // does the extant/extinct checks already
    fn update_ancestry_proportions_from_migration_matrix(&mut self) {
        for row in 0..self.ancestry_proportions.nrows() {
            let migs = self.migration_matrix.row(row);
            let props = self.ancestry_proportions.row_mut(row);
            let one_minus_sum: f64 = 1. - migs.iter().sum::<f64>();

            props.iter_mut().zip(migs.iter()).for_each(|(i, j)| {
                *i *= one_minus_sum;
                *i += *j;
            });
        }
    }

    fn deme_slice(&self, generation: Generation) -> &[Deme] {
        match generation {
            Generation::Parent => self.parent_demes.as_slice(),
            Generation::Child => self.child_demes.as_slice(),
        }
    }

    fn any_extant_demes(&self, generation: Generation) -> bool {
        let s = self.deme_slice(generation);
        s.iter().any(|deme| deme.is_extant())
    }

    fn num_extant_demes(&self, generation: Generation) -> usize {
        let s = self.deme_slice(generation);
        s.iter().filter(|deme| deme.is_extant()).count()
    }

    fn get_slice_if<'a, T: Sized>(&self, generation: Generation, s: &'a [T]) -> Option<&'a [T]> {
        let is_empty = match generation {
            Generation::Parent => self.parent_demes.is_empty(),
            Generation::Child => self.child_demes.is_empty(),
        };
        if !is_empty {
            Some(s)
        } else {
            None
        }
    }

    /// Update the internal state of the graph to the *parental*
    /// generation time `parental_generation_time`.
    pub fn update_state<F: Into<ForwardTime> + std::fmt::Debug + Copy>(
        &mut self,
        parental_generation_time: F,
    ) -> Result<(), DemesForwardError> {
        let parental_generation_time = parental_generation_time.into();
        if parental_generation_time.value().is_sign_negative()
            || !parental_generation_time.value().is_finite()
        {
            return Err(DemesForwardError::TimeError(format!(
                "invalid time for update_state: {parental_generation_time:?}",
            )));
        }
        if let Some(time) = self.last_time_updated {
            if parental_generation_time < time {
                // gotta reset...
                self.parent_demes.clear();
                self.child_demes.clear();
            }
        }
        let backwards_time = self.model_times.convert(parental_generation_time)?;
        update_demes(
            backwards_time,
            false,
            &self.deme_to_index,
            &self.graph,
            &mut self.parent_demes,
            &mut self.parental_deme_sizes,
        )?;
        let child_generation_time = ForwardTime::from(parental_generation_time.value() + 1.0);
        let backwards_time = self.model_times.convert(child_generation_time)?;
        update_demes(
            backwards_time,
            true,
            &self.deme_to_index,
            &self.graph,
            &mut self.child_demes,
            &mut self.child_deme_sizes,
        )?;
        self.update_pulses(backwards_time);
        self.update_migrations(backwards_time);

        self.selfing_rates.clear();
        self.cloning_rates.clear();
        for deme in &self.child_demes {
            match deme.status {
                DemeStatus::During(x) => {
                    self.cloning_rates
                        .push(deme.deme.epochs()[x].cloning_rate());
                    self.selfing_rates
                        .push(deme.deme.epochs()[x].selfing_rate());
                }
                _ => {
                    self.cloning_rates.push(demes::CloningRate::try_from(0.0)?);
                    self.selfing_rates.push(demes::SelfingRate::try_from(0.0)?);
                }
            }
        }

        self.initialize_ancestry_proportions();
        self.update_ancestry_proportions_from_pulses(parental_generation_time)?;
        self.update_migration_matrix(parental_generation_time)?;
        self.update_ancestry_proportions_from_migration_matrix();
        self.last_time_updated = Some(parental_generation_time);

        Ok(())
    }

    /// The total number of demes in the graph.
    pub fn num_demes_in_model(&self) -> usize {
        self.graph.num_demes()
    }

    /// The ancestry proporitions for a given offspring deme at the current time.
    ///
    /// # Parameters
    ///
    /// * offspring_deme: the index of an offspring deme.
    ///
    /// # Returns
    ///
    /// * `Some(&[f64])` if `offspring_deme` is a valid index and extant
    ///    offspring demes exist.
    /// * `None` otherwise.
    pub fn ancestry_proportions(&self, offspring_deme: usize) -> Option<&[f64]> {
        if offspring_deme >= self.num_demes_in_model() {
            return None;
        }
        if !self.child_demes.is_empty() {
            Some(self.ancestry_proportions.row(offspring_deme))
        } else {
            None
        }
    }

    /// Get cloning rates of all offspring demes.
    ///
    /// Returns `None` if there are no extant offspring
    /// demes.
    pub fn cloning_rates(&self) -> Option<&[demes::CloningRate]> {
        self.get_slice_if(Generation::Child, self.cloning_rates.as_slice())
    }

    /// Get selfing rates of all offspring demes.
    ///
    /// Returns `None` if there are no extant offspring
    /// demes.
    pub fn selfing_rates(&self) -> Option<&[demes::SelfingRate]> {
        self.get_slice_if(Generation::Child, self.selfing_rates.as_slice())
    }

    /// Obtain the time corresponding to the last
    /// call of [`ForwardGraph::update_state`].
    pub fn last_time_updated(&self) -> Option<ForwardTime> {
        self.last_time_updated
    }

    /// Obtain the end time of the model.
    pub fn end_time(&self) -> ForwardTime {
        let burnin_gen = self.model_times.burnin_generation();
        let model_duration = self.model_times.model_duration();

        (burnin_gen + model_duration).into()
    }

    /// Return an iterator over time values.
    ///
    /// The iterator starts at the last updated time and
    /// continues until the end time.
    pub fn time_iterator(&self) -> impl Iterator<Item = ForwardTime> {
        self.model_times.time_iterator(self.last_time_updated)
    }

    /// Obtain the sizes of each parental deme.
    ///
    /// The length of the slice is equal to the number of demes
    /// in the graph (see [`ForwardGraph::num_demes_in_model`]).
    ///
    /// Returns `None` if there are no parental demes at the current time.
    pub fn parental_deme_sizes(&self) -> Option<&[CurrentSize]> {
        self.get_slice_if(Generation::Parent, self.parental_deme_sizes.as_slice())
    }

    /// Obtain the sizes of each offspring deme.
    ///
    /// The length of the slice is equal to the number of demes
    /// in the graph (see [`ForwardGraph::num_demes_in_model`]).
    ///
    /// Returns `None` if there are no offspring demes at the current time.
    pub fn offspring_deme_sizes(&self) -> Option<&[CurrentSize]> {
        self.get_slice_if(Generation::Child, self.child_deme_sizes.as_slice())
    }

    /// Return `true` if there are any extant parental
    /// demes at the current time.
    pub fn any_extant_parental_demes(&self) -> bool {
        self.any_extant_demes(Generation::Parent)
    }

    /// Return `true` if there are any extant offspring
    /// demes at the current time.
    pub fn any_extant_offspring_demes(&self) -> bool {
        self.any_extant_demes(Generation::Child)
    }

    /// Return the number of extant parental demes
    /// at the current time.
    pub fn num_extant_parental_demes(&self) -> usize {
        self.num_extant_demes(Generation::Parent)
    }

    /// Return the number of extant offspring demes
    /// at the current time.
    pub fn num_extant_offspring_demes(&self) -> usize {
        self.num_extant_demes(Generation::Child)
    }

    /// Obtain the size (number of parental individuals) of a given
    /// deme at a given time.
    ///
    /// # Parameters
    ///
    /// * `deme`: a deme identifier
    /// * `time`: a time
    ///
    /// # Returns
    ///
    /// * `Some(DemeSize)` if there are parents at the given time.
    /// * `None` if `deme` does not exist or the deme exists but is
    ///    not in existence at `time`.
    ///
    /// # Errors
    ///
    /// [`DemesForwardError`] if the calculation of deme size returns an
    /// invalid [`demes::DemeSize`] or if `time` is not a valid value.
    ///
    /// # Notes
    ///
    /// * Time is measured backwards in time from 0.0 (inclusive of zero).
    ///   The reason is that it is usually easier to reason about time
    ///   in this way.
    /// * Unlike a [`demes::Graph`], time does not go back in the past
    ///   to "infinity". The first parental generation is considered
    ///   to exist at a time one generation prior to the start of events
    ///   in the graph plus the burn-in time.
    pub fn size_at<'a, I: Into<demes::DemeId<'a>>, T: Into<crate::time::BackwardTimeWrapper>>(
        &self,
        deme: I,
        time: T,
    ) -> Result<Option<CurrentSize>, DemesForwardError> {
        let time_raw: f64 = match time.into() {
            crate::time::BackwardTimeWrapper::Float(time) => demes::Time::try_from(time)?.into(),
            crate::time::BackwardTimeWrapper::Time(time) => time.into(),
        };
        if time_raw.is_nan() || !time_raw.is_sign_positive() {
            return Err(DemesForwardError::TimeError(format!(
                "invalid time value: {time_raw}"
            )));
        }
        if time_raw > self.model_times.model_start_time() {
            return Ok(None);
        }
        let id = deme.into();
        let deme = match self.graph.get_deme(id) {
            Some(d) => d,
            None => return Ok(None),
        };
        if let Some((index, _epoch)) = deme
            .epochs()
            .iter()
            .enumerate()
            .find(|(_, e)| time_raw >= e.end_time() && time_raw < e.start_time())
        {
            // We can unwrap b/c we either made a valid demes::Time
            // or had one passed in.
            apply_size_function(deme, index, Some(time_raw.try_into().unwrap()))
        } else {
            Ok(None)
        }
    }

    /// Generate an iterator over the size history of a deme.
    ///
    /// # Parameters
    ///
    /// * `deme`: the index or name of the deme
    ///
    /// # Returns
    ///
    /// An iterator over instances of [`DemeSizeAt`].
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * `deme` does not exist in the graph
    ///
    /// # Note
    ///
    /// The iterated values are lazily evaluated.
    /// First `self` is cloned.
    /// Then, the entire model history is iterated over,
    /// and the output data is returned during iteration.
    ///
    /// # Panics
    ///
    /// * Cloning the graph requires allocations which will
    ///   panic if the system runs out of memory
    pub fn deme_size_history<'a, I>(
        &self,
        deme: I,
    ) -> Result<impl Iterator<Item = DemeSizeAt>, DemesForwardError>
    where
        I: std::fmt::Debug + Into<demes::DemeId<'a>>,
    {
        let deme_index = match deme.into() {
            demes::DemeId::Index(index) => index,
            demes::DemeId::Name(name) => match self.deme_to_index.get(name) {
                Some(&index) => index,
                None => {
                    return Err(
                        demes::DemesError::DemeError(format!("invalid deme id {name}")).into(),
                    );
                }
            },
        };
        if deme_index >= self.graph.num_demes() {
            return Err(
                demes::DemesError::DemeError(format!("invalid deme index {deme_index}")).into(),
            );
        }
        let graph = self.clone();
        let forward_model_start_time = f64::from(self.model_times.model_start_time());
        DemeSizeHistory::new(graph, deme_index, forward_model_start_time)
    }

    /// Return the time in the past when the first generation
    /// of the model exists.
    /// This time point represents the individuals alive at
    /// time zero (forwards in time) who are the ancestors
    /// of those individuals born at time 1 of the model.
    pub fn backwards_start_time(&self) -> demes::Time {
        self.model_times.model_start_time()
    }

    /// Return the time in the model at the end of
    /// any "burn-in".
    ///
    /// # Note
    ///
    /// This value is distinct from the burn-in *duration*.
    /// Instead, this value represents the time when all alive
    /// individuals are the ancestors of the first finite
    /// time in the [`demes::Graph`].
    pub fn backwards_burn_in_time(&self) -> demes::Time {
        self.model_times.backwards_burn_in_time()
    }

    /// Convert a forward time value ([`ForwardTime`]) into a backwards time value
    /// ([`demes::Time`])
    ///
    /// # Returns
    ///
    /// * `None` if `time` is ancestral to the start of the model
    ///   or more recent than the model's end.
    pub fn time_to_backward<T: Into<crate::time::ForwardTimeWrapper>>(
        &self,
        time: T,
    ) -> Result<Option<demes::Time>, DemesForwardError> {
        let time: ForwardTime = match time.into() {
            crate::time::ForwardTimeWrapper::Float(time) => time.into(),
            crate::time::ForwardTimeWrapper::Time(time) => time,
        };
        if !time.valid() {
            return Err(DemesForwardError::TimeError(format!(
                "invalid time value: {time:?}"
            )));
        }
        self.model_times.convert(time)
    }

    /// Convert a backwards time value ([`demes::Time`]) into a forward time value
    /// ([`ForwardTime`])
    ///
    /// # Returns
    ///
    /// * `None` if `time` is ancestral to the start of the model
    ///   or more recent than the model's end.
    pub fn time_to_forward<T: Into<crate::time::BackwardTimeWrapper>>(
        &self,
        time: T,
    ) -> Result<Option<ForwardTime>, DemesForwardError> {
        let time: demes::Time = match time.into() {
            crate::time::BackwardTimeWrapper::Float(value) => value.try_into()?,
            crate::time::BackwardTimeWrapper::Time(value) => value,
        };
        if time > self.backwards_start_time() || time < self.graph.most_recent_deme_end_time() {
            Ok(None)
        } else {
            Ok(Some(
                (f64::from(self.backwards_start_time()) - f64::from(time)).into(),
            ))
        }
    }

    /// Get the names of each deme in the model.
    ///
    /// # Note
    ///
    /// Implemented by calling [`demes::Graph::deme_names`].
    pub fn deme_names(&self) -> Box<[&str]> {
        self.graph.deme_names()
    }

    /// Access to the underlying [`demes::Graph`]
    pub fn demes_graph(&self) -> &demes::Graph {
        &self.graph
    }
}

#[cfg(test)]
mod graphs_for_testing {
    pub fn four_deme_model() -> demes::Graph {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 100
      end_time: 50
 - name: B
   ancestors: [A]
   epochs:
    - start_size: 100
 - name: C
   ancestors: [A]
   epochs:
    - start_size: 100
      end_time: 49
 - name: D
   ancestors: [C, B]
   proportions: [0.5, 0.5]
   start_time: 49
   epochs:
    - start_size: 50
";
        demes::loads(yaml).unwrap()
    }

    pub fn one_generation_model() -> demes::Graph {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 50
      end_time: 1
    - start_size: 100
";
        demes::loads(yaml).unwrap()
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
    fn one_deme_two_epochs() {
        let demes_graph = two_epoch_model();
        let mut graph = ForwardGraph::new_discrete_time(demes_graph, 100_u32).unwrap();
        assert!(graph.update_state(-1.0).is_err());
        assert!(graph.update_state(f64::INFINITY).is_err());
        graph.update_state(125_i32).unwrap();
        assert_eq!(graph.num_extant_parental_demes(), 1);
        assert_eq!(
            graph.parental_deme_sizes().unwrap().first(),
            Some(&CurrentSize::try_from(100.0).unwrap())
        );
        graph.update_state(75_i32).unwrap();
        // assert_eq!(graph.parental_demes().unwrap().iter().count(), 1);
        assert_eq!(graph.num_extant_parental_demes(), 1);
        assert_eq!(
            graph.parental_deme_sizes().unwrap().first(),
            Some(&CurrentSize::try_from(200.0).unwrap())
        );

        // The last generation
        graph.update_state(150_i32).unwrap();
        assert_eq!(graph.num_extant_parental_demes(), 1);
        assert_eq!(graph.num_extant_offspring_demes(), 0);
        assert!(graph.ancestry_proportions(0).is_none());

        // One past the last generation
        graph.update_state(151_i32).unwrap();
        //assert!(graph.parental_demes().is_none());
        assert_eq!(graph.num_extant_parental_demes(), 0);
        assert_eq!(graph.num_extant_offspring_demes(), 0);
        assert!(graph.offspring_deme_sizes().is_none());

        // Test what happens as we "evolve through"
        // an epoch boundary.
        let expected_sizes = |generation: i32| -> (f64, f64) {
            if generation < 100 {
                (200.0, 200.0)
            } else if generation < 101 {
                (200.0, 100.0)
            } else {
                (100.0, 100.0)
            }
        };
        for generation in [99_i32, 100, 101, 102] {
            graph.update_state(generation).unwrap();
            let expected = expected_sizes(generation);

            assert!(graph
                .parental_deme_sizes()
                .unwrap()
                .iter()
                .all(|size| size == &expected.0));

            assert!(graph
                .offspring_deme_sizes()
                .unwrap()
                .iter()
                .all(|size| size == &expected.1));
        }
    }

    #[test]
    fn invalid_conversion_error() {
        let demes_graph = two_epoch_model_invalid_conversion_to_generations();
        let result = ForwardGraph::new_discrete_time(demes_graph, 100.0);
        assert!(matches!(
            result,
            Err(crate::DemesForwardError::DemesError(
                demes::DemesError::EpochError(_)
            ))
        ));
    }

    #[test]
    fn invalid_forward_time() {
        {
            let x = ForwardTime::new(-1_i32);
            assert!(!x.valid());
        }
        {
            let x = ForwardTime::from(-1_f64);
            assert!(!x.valid());
        }
        {
            let x = ForwardTime::from(f64::INFINITY);
            assert!(!x.valid());
        }
        {
            let x = ForwardTime::from(f64::NAN);
            assert!(!x.valid());
            let graph = two_epoch_model();
            assert!(ForwardGraph::new_discrete_time(graph, x).is_err());
        }
    }

    #[test]
    fn test_one_generation_model() {
        {
            let demes_graph = graphs_for_testing::one_generation_model();
            let mut graph = ForwardGraph::new_discrete_time(demes_graph, 0).unwrap();
            assert_eq!(graph.end_time(), 2.0.into());
            for i in graph.time_iterator() {
                graph.update_state(i).unwrap();
                assert!(graph
                    .parental_deme_sizes()
                    .unwrap()
                    .iter()
                    .any(|size| size > &0.0));
                if i == graph.end_time() - 1.0.into() {
                    assert!(graph.offspring_deme_sizes().is_none(), "time = {i:?}");
                } else {
                    assert!(graph.offspring_deme_sizes().is_some(), "time = {i:?}");
                }
            }
        }
    }
}

#[cfg(test)]
mod test_nonlinear_size_changes {
    use super::*;

    fn two_epoch_model_linear_growth() -> demes::Graph {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 200
      end_time: 50
    - start_size: 100
      end_size: 200
      size_function: linear
";
        demes::loads(yaml).unwrap()
    }

    fn two_epoch_model_linear_decline() -> demes::Graph {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 200
      end_time: 50
    - start_size: 200
      end_size: 100
      size_function: linear
";
        demes::loads(yaml).unwrap()
    }

    fn two_epoch_model_exponential_growth() -> demes::Graph {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 200
      end_time: 50
    - start_size: 100
      end_size: 200
      size_function: exponential
";
        demes::loads(yaml).unwrap()
    }

    fn two_deme_split_with_ancestral_size_change() -> demes::Graph {
        let yaml = "
description: Two deme model with migration and size changes.
time_units: generations
demes:
- name: ancestral
  description: ancestral deme, two epochs
  epochs:
  - {end_time: 20, start_size: 100}
  - {end_time: 10, start_size: 200}
- name: deme1
  description: child 1
  epochs:
  - {start_size: 250, end_size: 500, end_time: 0}
  ancestors: [ancestral]
- name: deme2
  description: child 2
  epochs:
  - {start_size: 50, end_size: 200, end_time: 0}
  ancestors: [ancestral]
migrations:
- {demes: [deme1, deme2], rate: 1e-3}
";
        demes::loads(yaml).unwrap()
    }

    // This test is lifted from fwdpy11 and is what brought up
    // the need for PR #235
    #[test]
    fn test_size_history_two_deme_split_with_ancestral_size_change() {
        let demes_graph = two_deme_split_with_ancestral_size_change();
        let mut graph = ForwardGraph::new_discrete_time(demes_graph, 100_u32).unwrap();

        // Manually iterate graph until we hit deme 1 for the first time as a child.
        graph.update_state(0.0).unwrap();
        let mut found = false;
        for time in graph.time_iterator() {
            graph.update_state(time).unwrap();
            let o = graph.offspring_deme_sizes().unwrap();
            if o[1] > 0.0 {
                // Then we make sure it is the right size
                assert_eq!(time.value(), 110.0);
                let g = (500_f64 / 250.).powf(1. / 10.) - 1.;
                assert_eq!(o[1], (250_f64 * (1. + g).powf(1.0)).round());
                found = true;
                break;
            }
        }
        assert!(found);
        found = false;
        // Manually iterate graph until we hit deme 2 for the first time as a child.
        graph.update_state(0.0).unwrap();
        for time in graph.time_iterator() {
            graph.update_state(time).unwrap();
            if let Some(o) = graph.offspring_deme_sizes() {
                if o[2] > 0.0 {
                    // Then we make sure it is the right size
                    assert_eq!(time.value(), 110.0);
                    let g = (200_f64 / 50.).powf(1. / 10.) - 1.;
                    assert_eq!(o[2], (50_f64 * (1. + g).powf(1.0)).round());
                    found = true;
                    break;
                }
            }
        }
        assert!(found);
    }

    #[test]
    fn test_two_epoch_model_linear_growth() {
        let demes_graph = two_epoch_model_linear_growth();
        let mut graph = ForwardGraph::new_discrete_time(demes_graph, 100_u32).unwrap();

        let slope: f64 = (200. - 100.) / 50.;
        graph.update_state(100).unwrap(); // last generation of the 1st epoch
        if let Some(deme) = graph.parent_demes.first() {
            assert_eq!(
                deme.current_size().unwrap(),
                Some(CurrentSize::try_from(200.).unwrap())
            );
        } else {
            panic!();
        }
        if let Some(deme) = graph.child_demes.first() {
            assert_eq!(
                deme.current_size().unwrap(),
                Some(CurrentSize::try_from((100. + slope).round()).unwrap())
            );
        } else {
            panic!();
        }
        // one generation before end
        graph.update_state(149).unwrap();
        if let Some(deme) = graph.child_demes.first() {
            assert_eq!(
                deme.current_size().unwrap(),
                Some(CurrentSize::try_from(200.).unwrap())
            );
        } else {
            panic!();
        }
        graph.update_state(150).unwrap(); // last gen
        assert!(graph.child_demes.first().is_none());
        if let Some(deme) = graph.parent_demes.first() {
            assert_eq!(
                deme.current_size().unwrap(),
                Some(CurrentSize::try_from(200.).unwrap())
            );
        } else {
            panic!();
        }

        // 1/2-way into the final epoch
        graph.update_state(125).unwrap();
        let expected_size: f64 = (100.0 + 25_f64 * slope).round();
        let expected_size = CurrentSize::try_from(expected_size.round()).unwrap();
        assert_eq!(
            graph.parental_deme_sizes().unwrap().first(),
            Some(&expected_size)
        );

        let expected_size: f64 = (100.0 + 26_f64 * slope).round();
        let expected_size = CurrentSize::try_from(expected_size.round()).unwrap();
        assert_eq!(
            graph.offspring_deme_sizes().unwrap().first(),
            Some(&expected_size)
        );
    }

    #[test]
    fn test_two_epoch_model_linear_decline() {
        let demes_graph = two_epoch_model_linear_decline();
        let mut graph = ForwardGraph::new_discrete_time(demes_graph, 100_u32).unwrap();
        graph.update_state(100).unwrap(); // last generation of the 1st epoch
        if let Some(deme) = graph.parent_demes.first() {
            assert_eq!(
                deme.current_size().unwrap(),
                Some(CurrentSize::try_from(200.).unwrap())
            );
        } else {
            panic!();
        }
        let slope: f64 = (100. - 200.) / 50.;
        if let Some(deme) = graph.child_demes.first() {
            assert_eq!(
                deme.current_size().unwrap(),
                Some(CurrentSize::try_from((200. + slope).round()).unwrap())
            );
        } else {
            panic!();
        }
        // one generation before end
        graph.update_state(149).unwrap();
        if let Some(deme) = graph.child_demes.first() {
            assert_eq!(
                deme.current_size().unwrap(),
                Some(CurrentSize::try_from(100.).unwrap())
            );
        } else {
            panic!();
        }
        graph.update_state(150).unwrap(); // last gen
        assert!(graph.child_demes.first().is_none());
        if let Some(deme) = graph.parent_demes.first() {
            assert_eq!(
                deme.current_size().unwrap(),
                Some(CurrentSize::try_from(100.).unwrap())
            );
        } else {
            panic!();
        }

        // 1/2-way into the final epoch
        graph.update_state(125).unwrap();
        let expected_size: CurrentSize = (200_f64 + 25.0 * slope).round().try_into().unwrap();
        assert_eq!(
            graph.parental_deme_sizes().unwrap().first(),
            Some(&expected_size)
        );
        let expected_size = CurrentSize::try_from((200_f64 + 26.0 * slope).round()).unwrap();
        assert_eq!(
            graph.offspring_deme_sizes().unwrap().first(),
            Some(&expected_size)
        );
    }

    #[test]
    fn test_two_epoch_model_exponential_growth() {
        let demes_graph = two_epoch_model_exponential_growth();
        let mut graph = ForwardGraph::new_discrete_time(demes_graph, 100_u32).unwrap();
        graph.update_state(100).unwrap(); // last generation of the 1st epoch
        if let Some(deme) = graph.parent_demes.first() {
            assert_eq!(
                deme.current_size().unwrap(),
                Some(CurrentSize::try_from(200.).unwrap())
            );
        } else {
            panic!();
        }
        let g = (200_f64 / 100.).powf(1. / 50.) - 1.;
        if let Some(deme) = graph.child_demes.first() {
            assert_eq!(
                deme.current_size().unwrap(),
                Some(CurrentSize::try_from((100. * (1. + g)).round()).unwrap())
            );
        } else {
            panic!();
        }
        // one generation before end
        graph.update_state(149).unwrap();
        if let Some(deme) = graph.child_demes.first() {
            assert_eq!(
                deme.current_size().unwrap(),
                Some(CurrentSize::try_from(200.).unwrap())
            );
        } else {
            panic!();
        }
        graph.update_state(150).unwrap(); // last gen
        assert!(graph.child_demes.first().is_none());
        if let Some(deme) = graph.parent_demes.first() {
            assert_eq!(
                deme.current_size().unwrap(),
                Some(CurrentSize::try_from(200.).unwrap())
            );
        } else {
            panic!();
        }

        // 1/2-way into the final epoch
        graph.update_state(125).unwrap();
        let g = (200_f64 / 100_f64).powf(1. / 50.0) - 1.;
        let expected_size = CurrentSize::try_from((100.0 * ((1. + g).powf(25.))).round()).unwrap();
        assert_eq!(
            graph.parental_deme_sizes().unwrap().first(),
            Some(&expected_size)
        );
        let expected_size = CurrentSize::try_from((100.0 * ((1. + g).powf(26.0))).round()).unwrap();
        assert_eq!(
            graph.offspring_deme_sizes().unwrap().first(),
            Some(&expected_size)
        );
        crate::test_functions::test_model_duration(&mut graph);
    }
}

#[cfg(test)]
mod test_deme_ancestors {
    use super::*;
    use crate::test_functions::test_model_duration;

    #[test]
    fn test_four_deme_model() {
        let demes_graph = graphs_for_testing::four_deme_model();
        let mut graph = ForwardGraph::new_discrete_time(demes_graph, 100).unwrap();

        {
            graph.update_state(0).unwrap();
            for (i, deme) in graph.child_demes.iter().enumerate() {
                if i < 1 {
                    assert!(deme.is_extant());
                    assert!(graph
                        .ancestry_proportions(i)
                        .unwrap()
                        .iter()
                        .any(|p| p > &0.0));
                } else {
                    assert!(!graph
                        .ancestry_proportions(i)
                        .unwrap()
                        .iter()
                        .any(|p| p > &0.0));
                }
            }
        }

        {
            graph.update_state(100).unwrap();
            let deme = graph.parent_demes.first().unwrap();
            assert!(deme.is_extant());
            assert_eq!(deme.ancestors().len(), 0);

            for descendant_deme in [1_usize, 2] {
                let deme = graph.child_demes.get(descendant_deme).unwrap();
                assert!(deme.is_extant());
                assert_eq!(deme.ancestors().len(), 1);
                assert_eq!(deme.ancestors()[0], 0_usize);
                match graph.ancestry_proportions(descendant_deme) {
                    Some(ancestry) => {
                        for ancestor in deme.ancestors() {
                            assert_eq!(ancestry[*ancestor], 1.0);
                        }
                    }
                    None => panic!("expected Some(ancestry)"),
                }
            }
        }
        {
            graph.update_state(101).unwrap();
            let deme = graph.parent_demes.first().unwrap();
            assert_eq!(deme.ancestors().len(), 0);

            for descendant_deme in [1_usize, 2] {
                let deme = graph.child_demes.get(descendant_deme).unwrap();
                if descendant_deme == 2 {
                    assert_eq!(
                        graph.offspring_deme_sizes().unwrap().get(2),
                        Some(&CurrentSize::try_from(0.0).unwrap())
                    );
                } else {
                    assert!(deme.is_extant());
                }
                assert_eq!(deme.ancestors().len(), 0);
            }
            let deme = graph.child_demes.get(3).unwrap();
            assert!(deme.is_extant());
            assert_eq!(deme.ancestors().len(), 2);
            assert_eq!(deme.ancestors(), &[2, 1]);
            assert_eq!(deme.proportions().len(), 2);
            assert_eq!(deme.proportions(), &[0.5, 0.5]);

            for ancestor in deme.ancestors() {
                assert!(graph.parent_demes.get(*ancestor).unwrap().is_extant());
            }
        }

        {
            graph.update_state(102).unwrap();
            for deme in graph.child_demes.iter() {
                assert!(deme.ancestors().is_empty());
                assert!(deme.proportions().is_empty());
            }
        }
        test_model_duration(&mut graph);
    }

    #[test]
    fn test_four_deme_model_duration() {
        let demes_graph = graphs_for_testing::four_deme_model();
        let mut graph = ForwardGraph::new_discrete_time(demes_graph, 73).unwrap();
        test_model_duration(&mut graph);
    }
}

#[cfg(test)]
mod test_pulses {
    use super::*;

    fn model_with_pulses() -> demes::Graph {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 50
 - name: B
   epochs:
    - start_size: 50
pulses:
 - sources: [A]
   dest: B
   time: 100
   proportions: [0.5]
";
        demes::loads(yaml).unwrap()
    }

    #[test]
    fn test_pulses() {
        let demes_g = model_with_pulses();
        let mut g = ForwardGraph::new_discrete_time(demes_g, 200.).unwrap();

        for time in [199, 200] {
            g.update_state(time).unwrap();
            for deme in 0..g.num_demes_in_model() {
                let ap = crate::test_functions::ancestry_proportions_from_graph(&g, deme).unwrap();
                let p = g.ancestry_proportions(deme).unwrap();
                for pi in p {
                    assert!(pi <= &1.0);
                }
                p.iter()
                    .zip(ap.iter())
                    .for_each(|(a, b)| assert!((a - b).abs() <= 1e-9, "{a} {b}"));
            }
        }
    }
}

#[cfg(test)]
mod test_migrations {
    use super::*;
    fn model_with_migrations() -> demes::Graph {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 50
 - name: B
   epochs:
    - start_size: 50
migrations:
 - source: A
   dest: B
   rate: 0.25
   start_time: 50
   end_time: 25
 - source: B
   dest: A
   rate: 0.1
   start_time: 40
   end_time: 20
 - demes: [A, B]
   rate: 0.05
   start_time: 15
";
        demes::loads(yaml).unwrap()
    }

    #[test]
    fn test_migration_rate_changes_1() {
        let demes_g = model_with_migrations();
        let mut g = ForwardGraph::new_discrete_time(demes_g, 200.).unwrap();

        // Making sure ;)
        // g.update_state(250).unwrap();
        // assert!(g.child_demes().is_none());
        // assert!(g.parental_demes().is_some());

        // One gen before everyone starts migrating
        g.update_state(199).unwrap();
        assert_eq!(g.migrations.len(), 0);
        // At forward time 201, we are at the
        // start of the first migration epoch,
        // meaning that children born at 201 can be migrants
        g.update_state(200).unwrap();
        assert_eq!(g.migrations.len(), 1);

        g.update_state(209).unwrap();
        assert_eq!(g.migrations.len(), 1);

        g.update_state(210).unwrap();
        assert_eq!(g.migrations.len(), 2);

        g.update_state(229).unwrap();
        assert_eq!(g.migrations.len(), 1);

        g.update_state(230).unwrap();
        assert_eq!(g.migrations.len(), 0);

        // Symmetric mig, so 2 Asymmetric deals...
        g.update_state(235).unwrap();
        assert_eq!(g.migrations.len(), 2);
    }

    // Another test from fwdpy11 that caught a bug here.
    #[test]
    fn test_migration_rate_changes_2() {
        let yaml = "
time_units: generations
generation_time: 1
demes:
- name: A
  epochs:
  - {end_time: 0, start_size: 100}
- name: B
  epochs:
  - {end_time: 0, start_size: 100}
migrations:
- demes: [A, B]
  rate: 0.0
  end_time: 30
- demes: [A, B]
  rate: 0.5
  start_time: 30
  end_time: 20
- demes: [A, B]
  rate: 0.0
  start_time: 20
  end_time: 10
- demes: [A, B]
  rate: 0.5
  start_time: 10
";
        let demes_g = demes::loads(yaml).unwrap();
        let mut g = ForwardGraph::new_discrete_time(demes_g, 10_u32).unwrap();

        //All ancestor demes are constant size.
        //Therefore, at the end of burn-in, the child generation
        //will have nonzero ancestry from each deme.
        g.update_state(10.0).unwrap();
        for deme in [0_usize, 1] {
            assert!(g
                .ancestry_proportions(deme)
                .unwrap()
                .iter()
                .all(|x| x > &0.0));
        }

        // Advance to the final parental generation
        g.update_state(g.end_time() - 2_f64.into()).unwrap();
        for deme in [0_usize, 1] {
            assert!(g
                .ancestry_proportions(deme)
                .unwrap()
                .iter()
                .all(|x| x > &0.0));
        }
    }
}

#[cfg(test)]
mod test_fractional_times {
    use super::*;

    fn bad_epoch_end_time() -> demes::Graph {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 100
      end_time: 10.2
";
        demes::loads(yaml).unwrap()
    }

    fn bad_deme_start_time() -> demes::Graph {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 100
      end_time: 10
 - name: B
   start_time: 50.1
   ancestors: [A]
   epochs:
    - start_size: 50
";
        demes::loads(yaml).unwrap()
    }

    fn bad_pulse_time() -> demes::Graph {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 100
      end_time: 10
 - name: B
   start_time: 50
   ancestors: [A]
   epochs:
    - start_size: 50
pulses:
 - sources: [A]
   dest: B
   proportions: [0.5]
   time: 30.2
";
        demes::loads(yaml).unwrap()
    }

    fn bad_migration_start_time() -> demes::Graph {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 100
      end_time: 10
 - name: B
   start_time: 50
   ancestors: [A]
   epochs:
    - start_size: 50
migrations:
 - source: A
   dest: B
   rate: 0.5
   start_time: 30.2
   end_time: 10
";
        demes::loads(yaml).unwrap()
    }

    fn bad_migration_end_time() -> demes::Graph {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 100
      end_time: 10
 - name: B
   start_time: 50
   ancestors: [A]
   epochs:
    - start_size: 50
migrations:
 - source: A
   dest: B
   rate: 0.5
   start_time: 30
   end_time: 10.2
";
        demes::loads(yaml).unwrap()
    }

    fn run_invalid_model(f: fn() -> demes::Graph) {
        let demes_graph = f();
        assert!(ForwardGraph::new_discrete_time(demes_graph, 1).is_ok());
    }

    #[test]
    fn test_invalid_models() {
        run_invalid_model(bad_epoch_end_time);
        run_invalid_model(bad_deme_start_time);
        run_invalid_model(bad_pulse_time);
        run_invalid_model(bad_migration_start_time);
        run_invalid_model(bad_migration_end_time);
    }
}

#[cfg(test)]
mod test_ancestry_proportions {
    use super::*;
    use crate::test_functions::*;

    #[test]
    fn sequential_pulses_at_same_time_two_demes() {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 1000
 - name: B
   epochs:
    - start_size: 1000
 - name: C
   epochs:
    - start_size: 1000
pulses:
- sources: [A]
  dest: C
  proportions: [0.33]
  time: 10
- sources: [B]
  dest: C
  proportions: [0.25]
  time: 10
";
        let demes_graph = demes::loads(yaml).unwrap();
        let mut graph = ForwardGraph::new_discrete_time(demes_graph, 50).unwrap();
        let index_a: usize = 0;
        let index_b: usize = 1;
        let index_c: usize = 2;
        let mut ancestry_proportions = vec![0.0; 3];
        ancestry_proportions[index_c] = 1.0;
        update_ancestry_proportions(&[index_a], &[0.33], &mut ancestry_proportions);
        update_ancestry_proportions(&[index_b], &[0.25], &mut ancestry_proportions);
        graph.update_state(49.0).unwrap();
        assert_eq!(graph.ancestry_proportions(2).unwrap().len(), 3);
        graph
            .ancestry_proportions(2)
            .unwrap()
            .iter()
            .zip(ancestry_proportions.iter())
            .for_each(|(a, b)| assert!((a - b).abs() <= 1e-9));
    }

    #[test]
    fn sequential_pulses_at_same_time_two_demes_reverse_pulse_order() {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 1000
 - name: B
   epochs:
    - start_size: 1000
 - name: C
   epochs:
    - start_size: 1000
pulses:
- sources: [B]
  dest: C
  proportions: [0.25]
  time: 10
- sources: [A]
  dest: C
  proportions: [0.33]
  time: 10
";
        let demes_graph = demes::loads(yaml).unwrap();
        let mut graph = ForwardGraph::new_discrete_time(demes_graph, 50).unwrap();
        graph.update_state(50.0).unwrap();
        assert_eq!(graph.ancestry_proportions(2).unwrap().len(), 3);
        let ancestry_proportions = ancestry_proportions_from_graph(&graph, 2).unwrap();
        assert_eq!(ancestry_proportions.len(), 3);
        graph
            .ancestry_proportions(2)
            .unwrap()
            .iter()
            .zip(ancestry_proportions.iter())
            .for_each(|(a, b)| assert!((a - b).abs() <= 1e-9, "{a} {b}"));

        for child_deme in [0, 1] {
            let independent = ancestry_proportions_from_graph(&graph, child_deme).unwrap();
            assert_eq!(
                graph.ancestry_proportions(child_deme),
                Some(independent.as_slice())
            );
        }
    }

    #[test]
    fn test_pulse_that_does_nothing() {
        let yaml = "
time_units: generations
description:
 The pulse does nothing because
 demes B and C are new after time 50.
 Demes with finite start_time must
 have 100% of their ancestry accounted
 for in the ancestors field.
demes:
 - name: A
   epochs:
    - start_size: 100
      end_time: 50
 - name: B
   ancestors: [A]
   epochs:
    - start_size: 100
 - name: C
   ancestors: [A]
   epochs:
    - start_size: 100
pulses:
 - sources: [A]
   dest: B
   time: 50
   proportions: [0.5]
";
        let demes_graph = demes::loads(yaml).unwrap();
        let mut graph = ForwardGraph::new_discrete_time(demes_graph, 0).unwrap();
        graph.update_state(0.0).unwrap();
        for child_deme in [1, 2] {
            match graph.child_demes[child_deme].current_size() {
                Ok(value) => assert!(value.is_some()),
                Err(_) => panic!("unexpected Error"),
            }
            assert_eq!(graph.child_demes[child_deme].ancestors().len(), 1);
            assert_eq!(
                graph.ancestry_proportions(child_deme),
                Some([1., 0., 0.].as_slice())
            );
        }
    }
    #[test]
    fn sequential_pulses_at_same_time_two_demes_with_symmetric_migration() {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 1000
 - name: B
   epochs:
    - start_size: 1000
 - name: C
   epochs:
    - start_size: 1000
pulses:
- sources: [A]
  dest: C
  proportions: [0.33]
  time: 10
- sources: [B]
  dest: C
  proportions: [0.25]
  time: 10
migrations:
 - demes: [A, B, C]
   rate: 1e-3
   start_time: 11
   end_time: 0
";
        let demes_graph = demes::loads(yaml).unwrap();
        let mut graph = ForwardGraph::new_discrete_time(demes_graph, 50).unwrap();
        for parental_generation_time in [49.0, 50.0, 51.0, 52.0] {
            graph.update_state(parental_generation_time).unwrap();

            for child_deme in 0..3 {
                let a = graph.ancestry_proportions(child_deme).unwrap();
                let e = ancestry_proportions_from_graph(&graph, child_deme).unwrap();
                assert_eq!(a.len(), e.len());
                a.iter().zip(e.iter()).for_each(|(a, b)| {
                    assert!(
                        (a - b).abs() <= 1e-9,
                        "failure at {parental_generation_time}, {:?}",
                        graph.migrations
                    )
                });
            }
        }
    }

    #[test]
    fn test_ancestry_proportions_from_ancestors() {
        let yaml = "
time_units: generations
demes:
 - name: A1
   epochs:
    - end_time: 50
      start_size: 100
 - name: A2
   epochs:
    - end_time: 50
      start_size: 100
 - name: D
   start_time: 50
   ancestors: [A1, A2]
   proportions: [0.5, 0.5]
   epochs:
    - start_size: 100
";
        let graph = demes::loads(yaml).unwrap();
        let mut fgraph = ForwardGraph::new_discrete_time(graph, 1.).unwrap();
        let at = fgraph.time_to_forward(50.0).unwrap().unwrap();
        // We are updating to a PARENTAL generation
        fgraph.update_state(at).unwrap();
        // So the ancestry proportions of the OFFSPRING should be:
        let aprops = fgraph.ancestry_proportions(2).unwrap().to_owned();
        assert_eq!(aprops, vec![0.5, 0.5, 0.0]);
        let ap = ancestry_proportions_from_graph(&fgraph, 2).unwrap();
        assert_eq!(aprops, ap);

        let at = fgraph.time_to_forward(49.0).unwrap().unwrap();
        fgraph.update_state(at).unwrap();
        let aprops = fgraph.ancestry_proportions(2).unwrap().to_owned();
        assert_eq!(aprops, vec![0., 0., 1.]);
        let ap = ancestry_proportions_from_graph(&fgraph, 2).unwrap();
        assert_eq!(aprops, ap);

        let at = fgraph.time_to_forward(51.0).unwrap().unwrap();
        fgraph.update_state(at).unwrap();
        let aprops = fgraph.ancestry_proportions(2).unwrap().to_owned();
        assert_eq!(aprops, vec![0., 0., 0.]);
        let ap = ancestry_proportions_from_graph(&fgraph, 2).unwrap();
        assert_eq!(ap, vec![0., 0., 0.]);
    }
}

#[cfg(test)]
mod migration_matrix {
    use super::*;
    use crate::test_functions::*;

    #[test]
    fn simple_two_way_migration() {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 1000
 - name: B
   epochs:
    - start_size: 1000
 - name: C
   epochs:
    - start_size: 1000
migrations:
- source: B
  dest: C
  rate: 0.25
- source: A
  dest: C
  rate: 0.25 
";
        let demes_graph = demes::loads(yaml).unwrap();
        let mut graph = ForwardGraph::new_discrete_time(demes_graph, 10).unwrap();
        // Parental generations 0-9 have no migration,
        // which starts at generation 10
        for generation in 0..9 {
            graph.update_state(generation).unwrap();
            assert_eq!(
                graph.ancestry_proportions(2),
                Some([0.25, 0.25, 0.5].as_slice())
            );
            assert_eq!(
                graph.ancestry_proportions(2),
                Some([0.25, 0.25, 0.5].as_slice())
            );
        }
    }

    #[test]
    fn simple_two_way_symmetric_migration() {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 1000
 - name: B
   epochs:
    - start_size: 1000
 - name: C
   epochs:
    - start_size: 1000
migrations:
- demes: [A, B, C]
  rate: 0.25
";
        let demes_graph = demes::loads(yaml).unwrap();
        let mut graph = ForwardGraph::new_discrete_time(demes_graph, 10).unwrap();
        for generation in 0..10 {
            graph.update_state(generation).unwrap();
            for deme in 0..3 {
                let expected = ancestry_proportions_from_graph(&graph, deme).unwrap();
                assert_eq!(
                    graph.ancestry_proportions(deme),
                    Some(expected.as_slice()),
                    "{deme} {expected:?}",
                );
            }
        }
    }
}

#[cfg(test)]
mod test_cloning_selfing_rates {
    use super::*;

    #[test]
    fn test_rate_changes() {
        let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - selfing_rate: 0.5
      end_time: 10
      start_size: 50
    - cloning_rate: 0.25
";
        let demes_graph = demes::loads(yaml).unwrap();
        let mut graph = ForwardGraph::new_discrete_time(demes_graph, 10).unwrap();
        graph.update_state(0.0).unwrap();
        match graph.selfing_rates() {
            Some(rates) => assert_eq!(rates[0], 0.5),
            None => panic!(),
        }
        match graph.cloning_rates() {
            Some(rates) => assert_eq!(rates[0], 0.0),
            None => panic!(),
        }
        graph.update_state(10.0).unwrap();
        match graph.selfing_rates() {
            Some(rates) => assert_eq!(rates[0], 0.0),
            None => panic!(),
        }
        match graph.cloning_rates() {
            Some(rates) => assert_eq!(rates[0], 0.25),
            None => panic!(),
        }
    }
}
