use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod rule;

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct FigureInfo {
    pub category: String,
    pub subcategory: String,
    pub kind: String,
    pub id: String,
}

impl FigureInfo {
    pub fn new(
        id: impl Into<String>,
        category: impl Into<String>,
        subcategory: impl Into<String>,
        kind: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            category: category.into(),
            subcategory: subcategory.into(),
            kind: kind.into(),
        }
    }
}

pub trait FigureInfoProvider {
    type Error;

    fn get_figures(&mut self) -> Result<Vec<FigureInfo>, Self::Error>;
}

#[derive(Default)]
pub struct MockFigureProvider {
    pattern: Vec<(Vec<FigureInfo>, Duration)>,
    index: usize,
}

impl MockFigureProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(mut self, figures: Vec<FigureInfo>, delay: Duration) -> Self {
        self.pattern.push((figures, delay));
        self
    }
}

impl FigureInfoProvider for MockFigureProvider {
    type Error = String;

    fn get_figures(&mut self) -> Result<Vec<FigureInfo>, Self::Error> {
        let (figures, delay) = self.pattern.get(self.index).unwrap();
        std::thread::sleep(*delay);
        self.index = (self.index + 1) % self.pattern.len();
        Ok(figures.clone())
    }
}
