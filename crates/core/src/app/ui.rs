#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentFilterMode {
    #[default]
    All,
    Busy,
    Active,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Grid,
    Focus,
}

#[derive(Default)]
pub struct UiState {
    pub should_quit: bool,
    pub filter_mode: AgentFilterMode,
    pub search_query: String,
    pub search_mode: bool,
    pub view_mode: ViewMode,
    pub selected_agent_idx: usize,
    pub selected_terminal_idx: usize,
}

impl UiState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cycle_filter_mode(&mut self) {
        self.filter_mode = match self.filter_mode {
            AgentFilterMode::All => AgentFilterMode::Busy,
            AgentFilterMode::Busy => AgentFilterMode::Active,
            AgentFilterMode::Active => AgentFilterMode::Offline,
            AgentFilterMode::Offline => AgentFilterMode::All,
        };
    }

    pub fn filter_label(&self) -> &'static str {
        match self.filter_mode {
            AgentFilterMode::All => "all",
            AgentFilterMode::Busy => "busy",
            AgentFilterMode::Active => "active",
            AgentFilterMode::Offline => "offline",
        }
    }

    pub fn begin_search(&mut self) {
        self.search_mode = true;
    }

    pub fn end_search(&mut self) {
        self.search_mode = false;
    }

    pub fn clear_search_query(&mut self) {
        self.search_query.clear();
    }

    pub fn append_search_char(&mut self, ch: char) {
        self.search_query.push(ch);
    }

    pub fn pop_search_char(&mut self) {
        self.search_query.pop();
    }

    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Grid => ViewMode::Focus,
            ViewMode::Focus => ViewMode::Grid,
        };
    }
}
