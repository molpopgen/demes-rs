#[derive(Debug, Clone)]
pub struct SquareMatrix {
    data: Vec<f64>,
    nrows: usize,
}

impl SquareMatrix {
    pub fn zeros(nrows: usize) -> Self {
        Self {
            data: vec![0.0; nrows * nrows],
            nrows,
        }
    }

    pub fn fill(&mut self, value: f64) {
        self.data.fill(value)
    }

    fn get_element_mut(&mut self, row: usize, column: usize) -> &mut f64 {
        &mut self.data[row * self.nrows + column]
    }

    pub fn set(&mut self, row: usize, column: usize, value: f64) {
        *self.get_element_mut(row, column) = value;
    }

    pub fn row(&self, row: usize) -> &[f64] {
        let start = row * self.nrows;
        let end = start + self.nrows;
        &self.data[start..end]
    }

    pub fn row_mut(&mut self, row: usize) -> &mut [f64] {
        let start = row * self.nrows;
        let end = start + self.nrows;
        &mut self.data[start..end]
    }

    pub fn nrows(&self) -> usize {
        self.nrows
    }
}
