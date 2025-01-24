use color_eyre::Result;
use hint_hackernews::HnStory;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{
        palette::tailwind::{BLUE, GREEN, SLATE, TEAL},
        Color, Modifier, Style, Stylize,
    },
    symbols,
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
    DefaultTerminal,
};
use std::sync::Arc;
mod hnreader;
mod hint_hackernews;
mod hint_log;
use crate::hint_log::init_debug_log;
use crate::hint_log::log_debug_info;

const HEADER_STYLE: Style = Style::new().fg(BLUE.c300).bg(BLUE.c700);
const NORMAL_ROW_BG: Color = BLUE.c950;
const ALT_ROW_BG_COLOR: Color = BLUE.c900;
const SELECTED_STYLE: Style = Style::new().bg(BLUE.c700).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = BLUE.c200;
const COMPLETED_TEXT_FG_COLOR: Color = TEAL.c400; // Slightly shifted for better contrast with blue

use tokio::sync::{Mutex};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    init_debug_log();
    color_eyre::install()?;

    let mut terminal = ratatui::init();
    let mut hintapp = App::default();

    // Create a new HnStoryList wrapped in Arc<Mutex<>>
    let story_list = Arc::new(Mutex::new(hint_hackernews::HnStoryList::new().await));

    // Create an mpsc channel for communication
    let (tx, mut rx) = mpsc::channel::<HnStory>(100);

    for story in story_list.lock().await.iter() {
        hintapp
            .storylist
            .append_item(DisplayListItem::from_hnstory(story.clone()));
    }

    // Start the update thread
    {
        let story_list_clone = Arc::clone(&story_list);
        tokio::spawn(async move {
            let mut locked_list = story_list_clone.lock().await;
            locked_list.start_update_thread_with_callback(tx.clone());
        });
    }

    // Main TUI loop
    loop {
        // Process received updates
        if let Some(updated_story) = rx.recv().await {
            // Add the received story to the display list
            hintapp.storylist.append_item(DisplayListItem::from_hnstory(updated_story));
        }

        terminal.draw(|frame| {
            let size = frame.area();
            hintapp.render(size, frame.buffer_mut());
        })?;

        if let Event::Key(key) = event::read()? {
            hintapp.handle_key(key);
        };

        // Check if the app should exit
        if hintapp.should_exit {
            break;
        }

        // Short delay to prevent excessive CPU usage
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    ratatui::restore();
    Ok(())
}

/// This struct holds the current state of the app. In particular, it has the `list` field
/// which is a wrapper around `ListState`. Keeping track of the state lets us render the
/// associated widget with its state and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events. Check
/// the drawing logic for items on how to specify the highlighting style for selected items.
struct App {
    should_exit: bool,
    show_details: bool,
    storylist: DisplayList,
    tick_count: u32,
}

struct DisplayList {
    items: Vec<DisplayListItem>,
    state: ListState,
}

#[derive(Debug)]
struct DisplayListItem {
    title: String,
    details: String,
    status: Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Status {
    Unread,
    Read,
}

impl Default for App {
    fn default() -> Self {
        Self {
            show_details: false,
            should_exit: false,
            storylist: DisplayList::from_iter([]),
            tick_count: 0,
        }
    }
}

impl DisplayList {
    fn from_iter<I: IntoIterator<Item = (Status, &'static str, &'static str)>>(iter: I) -> Self {
        let items = iter
            .into_iter()
            .map(|(status, title, details)| DisplayListItem::new(status, title, details))
            .collect();
        let state = ListState::default();
        Self { items, state }
    }

    fn append_item(&mut self, item: DisplayListItem) {
        self.items.push(item);
    }
}

impl DisplayListItem {
    fn new(status: Status, title: &str, details: &str) -> Self {
        Self {
            status,
            title:title.to_string(),
            details: details.to_string(),
        }
    }

    #[allow(dead_code)]
    fn from_hnstory(story: HnStory) -> Self {
        Self {
            status: Status::Unread,
            title: story.title().to_string(),
            details: String::from("Tobe filled in later")
        }
    }
}

impl App {
    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                self.toggle_status();
            }
            _ => {}
        }
    }

    fn select_none(&mut self) {
        self.storylist.state.select(None);
    }

    fn select_next(&mut self) {
        self.storylist.state.select_next();
    }
    fn select_previous(&mut self) {
        self.storylist.state.select_previous();
    }

    fn select_first(&mut self) {
        self.storylist.state.select_first();
    }

    fn select_last(&mut self) {
        self.storylist.state.select_last();
    }

    /// Changes the status of the selected list item
    fn toggle_status(&mut self) {
        if let Some(i) = self.storylist.state.selected() {
            self.storylist.items[i].status = match self.storylist.items[i].status {
                Status::Read => Status::Unread,
                Status::Unread => Status::Read,
            };
            self.show_details = match self.show_details {
                true => false,
                false => true,
            }
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [main_area, footer_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let (list_area, item_area);

        if self.show_details {
            let areas: [Rect; 2] = Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);
            list_area = areas[0];
            item_area = areas[1];
        } else {
            let areas: [Rect; 1] = Layout::vertical([Constraint::Fill(1)]).areas(main_area);
            list_area = areas[0];
            item_area = Rect::default(); // Use a default value when not needed
        }

        App::render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        if self.show_details == true {
            self.render_selected_item(item_area, buf);
        }
        self.tick_count += 1;
    }
}

/// Rendering logic for the app
impl App {
    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom.")
            .centered()
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("HackerNews").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all elements in the `items` and stylize them.
        let mut items: Vec<ListItem> = self
            .storylist
            .items
            .iter()
            .enumerate()
            .map(|(i, storyitem)| {
                let color = alternate_colors(i);
                ListItem::from(storyitem).bg(color)
            })
            .collect();

        // Define the spinner frames
        let spinner_frames = vec!["|", "/", "-", "\\"];
        let tick = self.tick_count; // Or you can use a counter from your app logic to track ticks

        // Get the current spinner frame
        let frame = spinner_frames[tick  as usize % (spinner_frames.len() as usize)];

        // Add the spinner as the last item
        items.push(ListItem::from(format!("  Updating... {}", frame)));

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.storylist.state);
    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        if self.show_details == false {
            return;
        }
        // We get the info depending on the item's state.
        let info = if let Some(i) = self.storylist.state.selected() {
            match self.storylist.items[i].status {
                Status::Read => format!("✓ DONE: {}", self.storylist.items[i].details),
                Status::Unread => format!("☐ TOREAD: {}", self.storylist.items[i].details),
            }
        } else {
            "Nothing selected...".to_string()
        };

        // We show the list item's info under the list in this paragraph
        let block = Block::new()
            .title(Line::raw("Story Details").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(HEADER_STYLE)
            .bg(NORMAL_ROW_BG)
            .padding(Padding::horizontal(1));

        // We can now render the item info
        Paragraph::new(info)
            .block(block)
            .fg(TEXT_FG_COLOR)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}

impl From<&DisplayListItem> for ListItem<'_> {
    fn from(value: &DisplayListItem) -> Self {
        let line = match value.status {
            Status::Unread => Line::styled(format!(" ☐ {}", value.title), TEXT_FG_COLOR),
            Status::Read => {
                Line::styled(format!(" ✓ {}", value.title), COMPLETED_TEXT_FG_COLOR)
            }
        };
        ListItem::new(line)
    }
}
