use std::time::Instant;

use space_game_protocol::{CompletionCandidateDto, CompletionCandidateKindDto};
use tui_input::{Input, InputRequest};

const LOCAL_COMMANDS: &[&str] = &["exit", "quit"];

#[derive(Debug, Clone)]
pub struct CommandInputController {
    input: Input,
    history: Vec<String>,
    history_cursor: Option<usize>,
    history_draft: String,
    reverse_search: Option<ReverseSearchState>,
    pending_completion: Option<PendingCompletionState>,
    completion_candidates: Vec<CompletionCandidateDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReverseSearchView {
    pub query: String,
    pub current_match: Option<String>,
}

#[derive(Debug, Clone)]
struct ReverseSearchState {
    draft: String,
    query: String,
    current_match: Option<String>,
    current_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct PendingCompletionState {
    pub seq: u64,
    pub requested_at: Instant,
}

impl Default for CommandInputController {
    fn default() -> Self {
        Self {
            input: Input::default(),
            history: Vec::new(),
            history_cursor: None,
            history_draft: String::new(),
            reverse_search: None,
            pending_completion: None,
            completion_candidates: Vec::new(),
        }
    }
}

impl CommandInputController {
    pub fn value(&self) -> &str {
        self.input.value()
    }

    pub fn cursor_byte(&self) -> usize {
        char_to_byte(self.input.value(), self.input.cursor())
    }

    pub fn visual_cursor(&self) -> usize {
        self.input.visual_cursor()
    }

    pub fn visual_scroll(&self, width: usize) -> usize {
        self.input.visual_scroll(width)
    }

    pub fn set_value(&mut self, value: String) {
        self.input = Input::new(value);
        self.history_cursor = None;
        self.completion_candidates.clear();
    }

    pub fn insert_char(&mut self, ch: char) {
        if let Some(search) = &mut self.reverse_search {
            search.query.push(ch);
            self.update_reverse_search_match();
            return;
        }
        self.leave_history_browse_for_edit();
        self.completion_candidates.clear();
        self.input.handle(InputRequest::InsertChar(ch));
    }

    pub fn backspace(&mut self) {
        if let Some(search) = &mut self.reverse_search {
            search.query.pop();
            self.update_reverse_search_match();
            return;
        }
        self.leave_history_browse_for_edit();
        self.completion_candidates.clear();
        self.input.handle(InputRequest::DeletePrevChar);
    }

    pub fn move_left(&mut self) {
        self.input.handle(InputRequest::GoToPrevChar);
    }

    pub fn move_right(&mut self) {
        self.input.handle(InputRequest::GoToNextChar);
    }

    pub fn history_previous(&mut self) {
        if self.history.is_empty() || self.reverse_search.is_some() {
            return;
        }

        let next_index = match self.history_cursor {
            Some(index) => index.saturating_sub(1),
            None => {
                self.history_draft = self.input.value().to_string();
                self.history.len() - 1
            }
        };
        self.history_cursor = Some(next_index);
        self.input = Input::new(self.history[next_index].clone());
        self.completion_candidates.clear();
    }

    pub fn history_next(&mut self) {
        let Some(index) = self.history_cursor else {
            return;
        };
        if self.reverse_search.is_some() {
            return;
        }

        if index + 1 < self.history.len() {
            let next_index = index + 1;
            self.history_cursor = Some(next_index);
            self.input = Input::new(self.history[next_index].clone());
        } else {
            self.history_cursor = None;
            self.input = Input::new(self.history_draft.clone());
            self.history_draft.clear();
        }
        self.completion_candidates.clear();
    }

    pub fn start_reverse_search(&mut self) {
        if self.reverse_search.is_none() {
            self.reverse_search = Some(ReverseSearchState {
                draft: self.input.value().to_string(),
                query: String::new(),
                current_match: None,
                current_index: None,
            });
            self.history_cursor = None;
        }
        self.update_reverse_search_match();
    }

    pub fn repeat_reverse_search(&mut self) {
        if self.reverse_search.is_none() {
            self.start_reverse_search();
            return;
        }
        self.update_reverse_search_match_from_previous();
    }

    pub fn accept_reverse_search(&mut self) -> bool {
        let Some(search) = self.reverse_search.take() else {
            return false;
        };
        if let Some(current_match) = search.current_match {
            self.input = Input::new(current_match);
        } else {
            self.input = Input::new(search.draft);
        }
        true
    }

    pub fn cancel_reverse_search(&mut self) -> bool {
        let Some(search) = self.reverse_search.take() else {
            return false;
        };
        self.input = Input::new(search.draft);
        true
    }

    pub fn reverse_search_view(&self) -> Option<ReverseSearchView> {
        self.reverse_search.as_ref().map(|search| ReverseSearchView {
            query: search.query.clone(),
            current_match: search.current_match.clone(),
        })
    }

    pub fn complete_local_command(&mut self) -> bool {
        self.completion_candidates.clear();
        let value = self.input.value();
        if value.split_whitespace().count() > 1 || value.starts_with(char::is_whitespace) {
            return false;
        }
        let prefix = value.trim();
        let matches = LOCAL_COMMANDS
            .iter()
            .copied()
            .filter(|command| command.starts_with(prefix))
            .map(|command| CompletionCandidateDto {
                insertion: command.to_string(),
                display: command.to_string(),
                kind: CompletionCandidateKindDto::LocalCommand,
            })
            .collect::<Vec<_>>();

        match matches.len() {
            0 => false,
            1 => {
                self.input = Input::new(matches[0].insertion.clone());
                true
            }
            _ => {
                self.completion_candidates = matches;
                true
            }
        }
    }

    pub fn completion_candidates(&self) -> &[CompletionCandidateDto] {
        &self.completion_candidates
    }

    pub fn pending_completion(&self) -> Option<&PendingCompletionState> {
        self.pending_completion.as_ref()
    }

    pub fn set_pending_completion(&mut self, seq: u64, requested_at: Instant) {
        self.pending_completion = Some(PendingCompletionState { seq, requested_at });
    }

    pub fn clear_pending_completion(&mut self) {
        self.pending_completion = None;
    }

    pub fn submit(&mut self) -> Option<String> {
        if self.accept_reverse_search() {
            return None;
        }

        let text = self.input.value().trim().to_string();
        self.input.reset();
        self.history_cursor = None;
        self.history_draft.clear();
        self.completion_candidates.clear();

        if text.is_empty() {
            return None;
        }
        self.record_history(&text);
        Some(text)
    }

    pub fn history(&self) -> &[String] {
        &self.history
    }

    fn record_history(&mut self, text: &str) {
        if matches!(text, "quit" | "exit") {
            return;
        }
        if self.history.last().is_some_and(|last| last == text) {
            return;
        }
        self.history.push(text.to_string());
    }

    fn leave_history_browse_for_edit(&mut self) {
        self.history_cursor = None;
        self.history_draft.clear();
    }

    fn update_reverse_search_match(&mut self) {
        let Some(search) = &self.reverse_search else {
            return;
        };
        let start = self.history.len();
        self.set_reverse_search_match_before(start, &search.query.clone());
    }

    fn update_reverse_search_match_from_previous(&mut self) {
        let Some(search) = &self.reverse_search else {
            return;
        };
        let start = search.current_index.unwrap_or(self.history.len());
        self.set_reverse_search_match_before(start, &search.query.clone());
    }

    fn set_reverse_search_match_before(&mut self, start: usize, query: &str) {
        let Some(search) = &mut self.reverse_search else {
            return;
        };
        let match_index = self
            .history
            .iter()
            .take(start)
            .enumerate()
            .rev()
            .find(|(_, entry)| entry.contains(query))
            .map(|(index, _)| index);

        search.current_index = match_index;
        search.current_match = match_index.map(|index| self.history[index].clone());
    }
}

fn char_to_byte(value: &str, char_index: usize) -> usize {
    value
        .char_indices()
        .nth(char_index)
        .map_or(value.len(), |(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recalled_history_entries_are_editable() {
        let mut input = CommandInputController::default();
        input.set_value("distance mars".to_string());
        assert_eq!(input.submit().as_deref(), Some("distance mars"));

        input.history_previous();
        input.backspace();
        input.backspace();
        input.backspace();
        input.backspace();
        for ch in "luna".chars() {
            input.insert_char(ch);
        }

        assert_eq!(input.submit().as_deref(), Some("distance luna"));
    }

    #[test]
    fn history_browsing_restores_draft() {
        let mut input = CommandInputController::default();
        input.set_value("objects".to_string());
        assert_eq!(input.submit().as_deref(), Some("objects"));

        input.set_value("distance ma".to_string());
        input.history_previous();
        assert_eq!(input.value(), "objects");
        input.history_next();
        assert_eq!(input.value(), "distance ma");
    }

    #[test]
    fn reverse_search_accepts_and_cancels() {
        let mut input = CommandInputController::default();
        input.set_value("objects".to_string());
        assert_eq!(input.submit().as_deref(), Some("objects"));
        input.set_value("distance mars".to_string());
        assert_eq!(input.submit().as_deref(), Some("distance mars"));

        input.set_value("draft".to_string());
        input.start_reverse_search();
        for ch in "mars".chars() {
            input.insert_char(ch);
        }
        assert_eq!(
            input.reverse_search_view().unwrap().current_match.as_deref(),
            Some("distance mars")
        );
        assert!(input.accept_reverse_search());
        assert_eq!(input.value(), "distance mars");

        input.set_value("draft".to_string());
        input.start_reverse_search();
        input.insert_char('o');
        assert!(input.cancel_reverse_search());
        assert_eq!(input.value(), "draft");
    }

    #[test]
    fn local_only_command_completion_applies_single_match() {
        let mut input = CommandInputController::default();
        input.set_value("qu".to_string());

        assert!(input.complete_local_command());
        assert_eq!(input.value(), "quit");
    }

    #[test]
    fn normal_submission_records_history_and_returns_text() {
        let mut input = CommandInputController::default();
        input.set_value("objects".to_string());

        assert_eq!(input.submit().as_deref(), Some("objects"));
        assert_eq!(input.value(), "");
        assert_eq!(input.history(), &["objects".to_string()]);
    }
}
