use dealve_core::models::Deal;

pub struct App {
    pub deals: Vec<Deal>,
    pub selected_index: usize,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            deals: vec![],
            selected_index: 0,
            should_quit: false,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn next(&mut self) {
        if !self.deals.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.deals.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.deals.is_empty() {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = self.deals.len() - 1;
            }
        }
    }
}
