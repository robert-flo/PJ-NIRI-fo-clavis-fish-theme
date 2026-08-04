pub mod duration;
pub mod git;
pub mod language;
pub mod path;
pub mod time;

use crate::config::Config;
use crate::prompt::PromptState;
use crate::segment::Segment;

#[derive(Default)]
pub struct RenderedModules {
    /// The duration module has one representation; it is deliberately the
    /// last informational module removed by the layout fitter.
    pub duration: Option<Segment>,
    /// Path variants are ordered from longest/context-rich to shortest.
    pub path: Vec<Segment>,
    /// Git variants are [full, branch-only] when both differ.
    pub git: Vec<Segment>,
    pub language: Option<Segment>,
    pub time: Option<Segment>,
}

pub fn render(config: &Config, state: &PromptState) -> RenderedModules {
    RenderedModules {
        duration: config
            .modules
            .duration
            .then(|| duration::render(config.colors.duration, state))
            .flatten(),
        path: if config.modules.path {
            path::render(config.colors.path)
        } else {
            Vec::new()
        },
        git: if config.modules.git {
            git::render(config.colors.git)
        } else {
            Vec::new()
        },
        language: config
            .modules
            .language
            .then(|| language::render(config.colors.language))
            .flatten(),
        time: config
            .modules
            .time
            .then(|| time::render(config.colors.time)),
    }
}
