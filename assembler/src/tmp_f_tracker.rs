use crate::misc::Mode;

use std::collections::{HashMap, HashSet};

#[derive(Default, Debug)]
pub struct TmpFTracker {
    // Label -> lines that have a forward ref to it.
    need: HashMap<u16, HashSet<usize>>,

    // Line -> label -> (addr, Mode) of next such declaration.
    found: HashMap<usize, HashMap<u16, (u16, Mode)>>,
}

impl TmpFTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&mut self, line: usize, label: u16) -> Option<(u16, Mode)> {
        self.found.get(&line).and_then(|hm| hm.get(&label)).copied()
    }

    pub fn need(&mut self, line: usize, label: u16) {
        self.need.entry(label).or_default().insert(line);
    }

    pub fn found(&mut self, label: u16, loc: u16, mode: Mode) {
        let Some(need) = self.need.remove(&label) else {
            return;
        };

        for line in need {
            self.found
                .entry(line)
                .or_default()
                .insert(label, (loc, mode));
        }
    }
}
