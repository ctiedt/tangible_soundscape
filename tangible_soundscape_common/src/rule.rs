use crate::FigureInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Rule {
    pub condition: Condition,
    pub sound: String,
}

impl Rule {
    pub fn matches(&self, figures: &[FigureInfo]) -> bool {
        self.condition.matches(figures)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Condition {
    MatchesCategory(String),
    MatchesSubcategory(String),
    MatchesKind(String),
    All(Vec<Condition>),
    Any(Vec<Condition>),
    AtLeast(u32, Box<Condition>),
}

impl Condition {
    pub fn matches(&self, figures: &[FigureInfo]) -> bool {
        match self {
            Condition::MatchesCategory(category) => {
                figures.iter().any(|figure| &figure.category == category)
            }
            Condition::MatchesSubcategory(subcategory) => figures
                .iter()
                .any(|figure| &figure.subcategory == subcategory),
            Condition::MatchesKind(kind) => figures.iter().any(|figure| &figure.kind == kind),
            Condition::All(rules) => rules.iter().map(|rule| rule.matches(figures)).all(|r| r),
            Condition::Any(rules) => rules.iter().map(|rule| rule.matches(figures)).any(|r| r),
            Condition::AtLeast(count, condition) => {
                figures
                    .iter()
                    .filter(|&figure| condition.matches(&[figure.clone()]))
                    .count() as u32
                    >= *count
            }
        }
    }
    pub fn figure(
        category: impl Into<String>,
        subcategory: impl Into<String>,
        kind: impl Into<String>,
    ) -> Condition {
        Condition::All(vec![
            Condition::MatchesCategory(category.into()),
            Condition::MatchesSubcategory(subcategory.into()),
            Condition::MatchesKind(kind.into()),
        ])
    }
}

#[cfg(test)]
mod test {
    use crate::FigureInfo;

    use super::Condition;

    #[test]
    fn test_basic() {
        let figures = vec![
            FigureInfo::new("123", "building", "rural", "farm"),
            FigureInfo::new("456", "plant", "tree", "oak"),
        ];

        let rule_1 = Condition::MatchesCategory("building".into());
        let rule_2 = Condition::MatchesCategory("creature".into());
        let rule_3 = Condition::figure("building", "rural", "farm");
        let rule_4 = Condition::figure("creature", "goblin", "barbarian");

        assert!(rule_1.matches(&figures));
        assert!(!rule_2.matches(&figures));
        assert!(rule_3.matches(&figures));
        assert!(!rule_4.matches(&figures));
    }

    #[test]
    fn test_and_rule() {
        let figures = vec![
            FigureInfo::new("123", "building", "rural", "farm"),
            FigureInfo::new("456", "plant", "tree", "oak"),
        ];

        let rule_1 = Condition::MatchesCategory("building".into());
        let rule_2 = Condition::MatchesCategory("plant".into());
        let rule_3 = Condition::All(vec![rule_1, rule_2]);

        assert!(rule_3.matches(&figures));
    }

    #[test]
    fn test_or_rule() {
        let figures = vec![
            FigureInfo::new("123", "building", "rural", "farm"),
            FigureInfo::new("456", "plant", "tree", "oak"),
        ];

        let rule_1 = Condition::MatchesCategory("building".into());
        let rule_2 = Condition::MatchesCategory("creature".into());
        let rule_3 = Condition::Any(vec![rule_1, rule_2]);

        assert!(rule_3.matches(&figures));
    }
}
