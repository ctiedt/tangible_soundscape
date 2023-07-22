use std::time::Duration;

use tangible_soundscape_common::{FigureInfo, FigureInfoProvider, MockFigureProvider};

fn main() {
    let fig1 = FigureInfo::new("0", "building", "rural", "farm");
    let fig2 = FigureInfo::new("1", "plant", "tree", "oak");
    let fig3 = FigureInfo::new("2", "plant", "tree", "oak");
    let fig4 = FigureInfo::new("3", "creature", "goblin", "barbarian");

    let mut provider = MockFigureProvider::new()
        .add(vec![fig1.clone(), fig2.clone()], Duration::from_secs(1))
        .add(
            vec![fig1.clone(), fig2.clone(), fig3.clone()],
            Duration::from_secs(1),
        )
        .add(
            vec![fig1.clone(), fig2.clone(), fig3.clone(), fig4.clone()],
            Duration::from_secs(1),
        );

    while let Ok(info) = provider.get_figures() {
        dbg!(info);
    }
}
