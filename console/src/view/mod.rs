use crate::input;
use std::borrow::Cow;
use tui::{
    layout,
    style::{self, Style},
    text::Span,
};

mod task;
mod tasks;

pub struct View {
    /// The tasks list is stored separately from the currently selected state,
    /// because it serves as the console's "home screen".
    ///
    /// When we return to the tasks list view (such as by exiting the task
    /// details view), we want to leave the task list's state the way we left it
    /// --- e.g., if the user previously selected a particular sorting, we want
    /// it to remain sorted that way when we return to it.
    list: tasks::List,
    state: ViewState,
}
enum ViewState {
    /// The table list of all tasks.
    TasksList,
    /// Inspecting a single task instance.
    TaskInstance(self::task::TaskView),
}

macro_rules! key {
    ($code:ident) => {
        input::Event::Key(input::KeyEvent {
            code: input::KeyCode::$code,
            ..
        })
    };
}

impl View {
    pub(crate) fn update_input(&mut self, event: input::Event) {
        use ViewState::*;
        match self.state {
            TasksList => {
                // The enter key changes views, so handle here since we can
                // mutate the currently selected view.
                match event {
                    key!(Enter) => {
                        if let Some(task) = self.list.selected_task().upgrade() {
                            self.state = TaskInstance(self::task::TaskView::new(task));
                        }
                    }
                    _ => {
                        // otherwise pass on to view
                        self.list.update_input(event);
                    }
                }
            }
            TaskInstance(ref mut view) => {
                // The escape key changes views, so handle here since we can
                // mutate the currently selected view.
                match event {
                    key!(Esc) => {
                        self.state = TasksList;
                    }
                    _ => {
                        // otherwise pass on to view
                        view.update_input(event);
                    }
                }
            }
        }
    }

    pub(crate) fn render<B: tui::backend::Backend>(
        &mut self,
        frame: &mut tui::terminal::Frame<B>,
        area: layout::Rect,
        tasks: &mut crate::tasks::State,
    ) {
        match self.state {
            ViewState::TasksList => {
                self.list.render(frame, area, tasks);
            }
            ViewState::TaskInstance(ref mut view) => {
                let now = tasks
                    .last_updated_at()
                    .expect("task view implies we've received an update");
                view.render(frame, area, now);
            }
        }

        tasks.retain_active();
    }
}

impl Default for View {
    fn default() -> Self {
        Self {
            state: ViewState::TasksList,
            list: tasks::List::default(),
        }
    }
}

pub(crate) fn bold<'a>(text: impl Into<Cow<'a, str>>) -> Span<'a> {
    Span::styled(text, Style::default().add_modifier(style::Modifier::BOLD))
}
