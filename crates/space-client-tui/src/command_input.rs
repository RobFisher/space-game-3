use std::time::Instant;

use space_game_protocol::{
    CompletionCandidateDto, CompletionCandidateKindDto, CompletionRequestDto,
    CompletionResponseDto, ReplacementSpanDto,
};
use tui_input::{Input, InputRequest};

use crate::history::DEFAULT_MAX_HISTORY;

const LOCAL_COMMANDS: &[&str] = &["exit", "quit"];
const COMPLETION_SPINNER_DELAY: std::time::Duration = std::time::Duration::from_millis(200);

#[derive(Debug, Clone)]
pub struct CommandInputController {
    input: Input,
    history: Vec<String>,
    history_cursor: Option<usize>,
    history_draft: String,
    reverse_search: Option<ReverseSearchState>,
    pending_completion: Option<PendingCompletionState>,
    completion_candidates: Vec<CompletionCandidateDto>,
    max_history: usize,
    history_dirty: bool,
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
            max_history: DEFAULT_MAX_HISTORY,
            history_dirty: false,
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
        self.pending_completion = None;
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
        self.pending_completion = None;
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
        self.pending_completion = None;
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
        self.pending_completion = None;
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
        self.reverse_search
            .as_ref()
            .map(|search| ReverseSearchView {
                query: search.query.clone(),
                current_match: search.current_match.clone(),
            })
    }

    pub fn complete_local_command(&mut self) -> bool {
        self.completion_candidates.clear();
        self.pending_completion = None;
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

    pub fn cancel_pending_completion(&mut self) -> bool {
        let was_pending = self.pending_completion.is_some();
        self.pending_completion = None;
        was_pending
    }

    pub fn completion_request(&mut self, seq: u64, requested_at: Instant) -> CompletionRequestDto {
        self.set_pending_completion(seq, requested_at);
        self.completion_candidates.clear();
        CompletionRequestDto {
            seq,
            input: self.input.value().to_string(),
            cursor: self.cursor_byte(),
        }
    }

    pub fn apply_completion_response(&mut self, response: CompletionResponseDto) -> bool {
        let Some(pending) = &self.pending_completion else {
            return false;
        };
        if pending.seq != response.seq {
            return false;
        }
        self.pending_completion = None;
        self.completion_candidates.clear();

        match response.candidates.len() {
            0 => true,
            1 => {
                let candidate = &response.candidates[0];
                self.replace_span(&response.replacement, &candidate.insertion);
                true
            }
            _ => {
                let replacement_text = self
                    .input
                    .value()
                    .get(response.replacement.start..response.replacement.end)
                    .unwrap_or_default()
                    .to_string();
                let common_prefix = longest_common_prefix(
                    response
                        .candidates
                        .iter()
                        .map(|candidate| candidate.insertion.as_str()),
                );
                if common_prefix.len() > replacement_text.len()
                    && common_prefix.starts_with(&replacement_text)
                {
                    self.replace_span(&response.replacement, &common_prefix);
                }
                self.completion_candidates = response.candidates;
                true
            }
        }
    }

    pub fn show_completion_pending(&self, now: Instant) -> bool {
        self.pending_completion.as_ref().is_some_and(|pending| {
            now.duration_since(pending.requested_at) > COMPLETION_SPINNER_DELAY
        })
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

    pub fn set_history(&mut self, history: Vec<String>) {
        self.history = history;
        self.bound_history();
        self.history_cursor = None;
        self.history_draft.clear();
        self.history_dirty = false;
    }

    pub fn take_history_dirty(&mut self) -> bool {
        let dirty = self.history_dirty;
        self.history_dirty = false;
        dirty
    }

    fn record_history(&mut self, text: &str) {
        if matches!(text, "quit" | "exit") {
            return;
        }
        if self.history.last().is_some_and(|last| last == text) {
            return;
        }
        self.history.push(text.to_string());
        self.bound_history();
        self.history_dirty = true;
    }

    fn bound_history(&mut self) {
        if self.history.len() > self.max_history {
            let start = self.history.len() - self.max_history;
            self.history.drain(0..start);
        }
    }

    fn replace_span(&mut self, replacement: &ReplacementSpanDto, insertion: &str) {
        let value = self.input.value();
        if replacement.start > replacement.end
            || replacement.end > value.len()
            || !value.is_char_boundary(replacement.start)
            || !value.is_char_boundary(replacement.end)
        {
            return;
        }

        let mut next = String::with_capacity(
            value.len() - (replacement.end - replacement.start) + insertion.len(),
        );
        next.push_str(&value[..replacement.start]);
        next.push_str(insertion);
        next.push_str(&value[replacement.end..]);
        let cursor = byte_to_char(&next, replacement.start + insertion.len());
        self.input = Input::new(next).with_cursor(cursor);
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

fn byte_to_char(value: &str, byte_index: usize) -> usize {
    value[..byte_index].chars().count()
}

fn longest_common_prefix<'a>(mut values: impl Iterator<Item = &'a str>) -> String {
    let Some(first) = values.next() else {
        return String::new();
    };
    let mut prefix = first.to_string();
    for value in values {
        while !value.starts_with(&prefix) {
            let Some((index, _)) = prefix.char_indices().last() else {
                return String::new();
            };
            prefix.truncate(index);
        }
    }
    prefix
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn candidate(text: &str) -> CompletionCandidateDto {
        CompletionCandidateDto {
            insertion: text.to_string(),
            display: text.to_string(),
            kind: CompletionCandidateKindDto::Object,
        }
    }

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
            input
                .reverse_search_view()
                .unwrap()
                .current_match
                .as_deref(),
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

    #[test]
    fn adjacent_repeated_commands_are_deduplicated() {
        let mut input = CommandInputController::default();
        for _ in 0..2 {
            input.set_value("objects".to_string());
            assert_eq!(input.submit().as_deref(), Some("objects"));
        }

        assert_eq!(input.history(), &["objects".to_string()]);
    }

    #[test]
    fn loaded_history_is_bounded() {
        let mut input = CommandInputController::default();
        input.set_history((0..1_005).map(|index| format!("cmd-{index}")).collect());

        assert_eq!(input.history().len(), 1_000);
        assert_eq!(input.history().first().unwrap(), "cmd-5");
    }

    #[test]
    fn completion_request_includes_input_and_cursor_byte_offset() {
        let mut input = CommandInputController::default();
        input.set_value("distance ma".to_string());
        let now = Instant::now();

        let request = input.completion_request(9, now);

        assert_eq!(request.seq, 9);
        assert_eq!(request.input, "distance ma");
        assert_eq!(request.cursor, 11);
        assert_eq!(input.pending_completion().unwrap().seq, 9);
    }

    #[test]
    fn single_completion_candidate_replaces_span() {
        let mut input = CommandInputController::default();
        input.set_value("distance ma".to_string());
        let _ = input.completion_request(10, Instant::now());

        assert!(input.apply_completion_response(CompletionResponseDto {
            seq: 10,
            replacement: ReplacementSpanDto { start: 9, end: 11 },
            candidates: vec![candidate("Mars")],
        }));

        assert_eq!(input.value(), "distance Mars");
        assert_eq!(input.cursor_byte(), "distance Mars".len());
    }

    #[test]
    fn multi_candidate_response_applies_longest_common_prefix() {
        let mut input = CommandInputController::default();
        input.set_value("distance mar".to_string());
        let _ = input.completion_request(11, Instant::now());

        assert!(input.apply_completion_response(CompletionResponseDto {
            seq: 11,
            replacement: ReplacementSpanDto { start: 9, end: 12 },
            candidates: vec![candidate("martian-base"), candidate("martian-station")],
        }));

        assert_eq!(input.value(), "distance martian-");
        assert_eq!(input.completion_candidates().len(), 2);
    }

    #[test]
    fn completion_pending_indicator_obeys_threshold() {
        let mut input = CommandInputController::default();
        let now = Instant::now();
        let _ = input.completion_request(12, now);

        assert!(!input.show_completion_pending(now + Duration::from_millis(200)));
        assert!(input.show_completion_pending(now + Duration::from_millis(201)));
    }

    #[test]
    fn cancellation_and_stale_completion_responses_are_ignored() {
        let mut input = CommandInputController::default();
        input.set_value("distance ma".to_string());
        let _ = input.completion_request(13, Instant::now());
        assert!(input.cancel_pending_completion());

        assert!(!input.apply_completion_response(CompletionResponseDto {
            seq: 13,
            replacement: ReplacementSpanDto { start: 9, end: 11 },
            candidates: vec![candidate("Mars")],
        }));
        assert_eq!(input.value(), "distance ma");

        let _ = input.completion_request(14, Instant::now());
        assert!(!input.apply_completion_response(CompletionResponseDto {
            seq: 99,
            replacement: ReplacementSpanDto { start: 9, end: 11 },
            candidates: vec![candidate("Mars")],
        }));
        assert_eq!(input.value(), "distance ma");
    }
}
