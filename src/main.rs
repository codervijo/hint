use color_eyre::Result;
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

#[tokio::main]
async fn main() -> Result<()> {
    init_debug_log();
    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut hintapp = App::default();

    match hnreader::fetch_top_stories().await {
        Ok(story_ids) => {
            //println!("Top Stories IDs: {:?}", story_ids);

            for (i, sid) in story_ids.iter().enumerate() {
                if i > 5 {
                    break;
                }
                let mut title = String::from("abc");
                let mut url = String::from("hcker");
                match hnreader::fetch_story_details(*sid).await {
                    Ok(story) => {
                        //println!("Story Details: {:?}", story);
                        title = story.title.clone().unwrap_or_else(|| String::from("Untitled"));
                        url = story.url.clone().unwrap_or_else(|| String::from("http://example.com"));
                    }
                    Err(err) => eprintln!("Failed to fetch story details: {}", err),
                }
                //println!("\n");
                hintapp.storylist.append_item(DisplayListItem {
                    title,
                    details: format!("Details From URL: {}", url),
                    status: Status::Unread,
                });
            }
        }
        Err(err) => eprintln!("Failed to fetch top stories: {}", err),
    }

    let hl = hint_hackernews::HnStoryList::new().await;
    println!("hn list: {:?}", hl);
    log_debug_info("HackerNews List:", format_args!("{:?}", hl));

    let res = hintapp.run(terminal);
    ratatui::restore();
    res
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
}

impl App {
    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }

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
        let items: Vec<ListItem> = self
            .storylist
            .items
            .iter()
            .enumerate()
            .map(|(i, storyitem)| {
                let color = alternate_colors(i);
                ListItem::from(storyitem).bg(color)
            })
            .collect();

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
