extern crate termion;
extern crate tui;

extern crate eksicli;

use std::io;
use std::thread;
use std::sync::mpsc;

use termion::event::Key;
use termion::input::TermRead;

use tui::Terminal;
use tui::backend::MouseBackend;
use tui::widgets::{Block, Borders, Paragraph, SelectableList, Widget};
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Modifier, Style};

use eksicli::eksi;
use eksicli::endpoints::title::Title;
use eksicli::endpoints::entry::Entry;


const ENTRY_HEIGHT: u16 = 6;
const LOGO: &str = "
        __           .__                 .__  .__
  ____ |  | __  _____|__|           ____ |  | |__|
_/ __ \\|  |/ / /  ___/  |  ______ _/ ___\\|  | |  |
\\  ___/|    <  \\___ \\|  | /_____/ \\  \\___|  |_|  |
 \\___  >__|_ \\/____  >__|          \\___  >____/__|
     \\/     \\/     \\/                  \\/
";

enum Event {
    Input(Key),
}

enum ShowMode {
    SingleEntry,
    EntryList,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    CommandInput,
}

enum Panel {
    Titles,
    Entries,
}

// TODO: add command history
pub struct App {
    size: Rect,
    /// Vector of titles that are shown in title panel
    titles: Vec<Title>,
    /// Vector of entries that are shown in entry panel
    entries: Vec<Entry>,
    /// Show gundem entries?
    popular: bool,
    /// Currently selected title object
    title: Option<Title>,
    /// Index of the currently hovered title
    title_selected: usize,
    /// Index of the currently open title
    title_current: usize,
    /// Current page of the left frame
    title_page: usize,
    /// Index of the currently hovered
    entry_current: usize,
    /// Current page of the currently open title
    entry_page: usize,
    show_mode: ShowMode,
    mode: Mode,
    panel_current: Panel,
    command_buffer: String,
    info_buffer: String,
}

impl App {
    fn focus_entries(&mut self) {
        self.panel_current = Panel::Entries;
    }

    fn focus_titles(&mut self) {
        self.panel_current = Panel::Titles;
    }

    fn update_entries(&mut self, entries: Vec<Entry>) {
        self.entry_current = 0;
        self.entry_page = 0;

        self.entries = entries;
    }
    // TODO: maybe replace matching with slice pattern syntax
    fn execute_command(&mut self) {
        if self.command_buffer.is_empty() {
            self.mode = Mode::Normal;
            return;
        }

        let first = self.command_buffer.chars().next();
        match first {
            Some('/') => {
                // Search
                let search_result = eksi::search(&self.command_buffer[1..]);
                match search_result {
                    Some((title, tentries)) => {
                        // Update with new data
                        self.title = Some(title);
                        self.update_entries(tentries);
                    }
                    _ => {
                        // Show error
                        self.info_buffer = "Can't find that Baslik.".to_string();
                    }
                }
            }
            Some(_) => {}
            None => {}
        }

        self.mode = Mode::Normal;
    }

    fn update_size(&mut self, t: &mut Terminal<MouseBackend>) {
        let size = t.size().unwrap();
        if size != self.size {
            t.resize(size).unwrap();
            self.size = size;
        }
    }

    fn draw_ui(&self, t: &mut Terminal<MouseBackend>) -> Result<(), io::Error> {
        let size = t.size()?;

        Group::default()
            .direction(Direction::Vertical)
            .sizes(&[Size::Percent(99), Size::Fixed(1)])
            .render(t, &size, |t, chunks| {
                self.draw_content(t, &chunks[0]);
                self.draw_footer(t, &chunks[1]);
            });

        t.draw()
    }

    fn draw_content(&self, t: &mut Terminal<MouseBackend>, area: &Rect) {
        let titles_str: Vec<_> = self.titles.iter().map(|x| format!("{}", x)).collect();

        Group::default()
            .direction(Direction::Horizontal)
            .sizes(&[Size::Fixed(60), Size::Percent(100)])
            .render(t, area, |t, chunks| {
                // Title group
                SelectableList::default()
                    .block(Block::default())
                    .items(&titles_str)
                    .select(self.title_selected)
                    .highlight_style(Style::default().modifier(Modifier::Bold))
                    .highlight_symbol(">")
                    .render(t, &chunks[0]);


                // Title and entry group
                Group::default()
                    .direction(Direction::Vertical)
                    .sizes(&vec![Size::Fixed(2), Size::Percent(100)])
                    .render(t, &chunks[1], |t, chunks| {
                        // Draw title
                        let normal_style = Style::default()
                            .fg(Color::White)
                            .modifier(Modifier::Bold);
                        Paragraph::default()
                            .wrap(true)
                            .style(normal_style)
                            .text(&self.title.as_ref()
                                  .unwrap_or(&Title {id:1,title: String::new(),popular_count:None}).title)
                            .render(t, &chunks[0]);

                        if self.entries.is_empty() {
                            // Draw logo
                            Paragraph::default()
                                .style(normal_style)
                                .text(LOGO)
                                .render(t, &chunks[1]);
                        } else {
                            // Draw entry group
                            match self.show_mode {
                                ShowMode::SingleEntry => {
                                    Group::default()
                                        .direction(Direction::Vertical)
                                        .sizes(&vec![Size::Percent(100)])
                                        .render(t, &chunks[1], |t, chunks| {
                                            self.draw_entry(
                                                t,
                                                &chunks[0],
                                                &self.entries[self.entry_current],
                                                self.entry_current,
                                            );
                                        });
                                },
                                ShowMode::EntryList => {
                                    let content_height = chunks[1].height;
                                    let display_count = (content_height / ENTRY_HEIGHT) as usize;

                                    Group::default()
                                        .direction(Direction::Vertical)
                                        .sizes(&vec![Size::Fixed(ENTRY_HEIGHT); display_count])
                                        .render(t, &chunks[1], |t, chunks| {
                                            for i in 0..self.entries.iter().len() {
                                                let mut offset = 0;
                                                if self.entry_current > display_count {
                                                    offset = self.entry_current - (display_count - 1);
                                                }

                                                if i < display_count {
                                                    let index = i + offset;
                                                    self.draw_entry(t, &chunks[i], &self.entries[index], index);
                                                }
                                            }
                                        });
                                }
                            }
                        }
                    });
            });
    }

    fn draw_footer(&self, t: &mut Terminal<MouseBackend>, area: &Rect) {
        match self.mode {
            Mode::CommandInput => {
                // Display command input
                let input_style = Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow);

                Paragraph::default()
                    .wrap(true)
                    .style(input_style)
                    .text(&self.command_buffer)
                    .render(t, area);
            }
            Mode::Normal if !self.info_buffer.is_empty() => {
                // Display info/error
                let info_style = Style::default()
                    .fg(Color::Black)
                    .bg(Color::Red); // Dark red

                Paragraph::default()
                    .wrap(true)
                    .style(info_style)
                    .text(&self.info_buffer)
                    .render(t, area);
            }
            Mode::Normal => {
                // Display simple help
                let normal_style = Style::default()
                    .fg(Color::White)
                    .bg(Color::DarkGray); // Dark blueish

                Paragraph::default()
                    .wrap(true)
                    .style(normal_style)
                    .text(&format!(
                        "/: arama, esc,h: iptal/geri, j: sonraki entry/baslik, k: onceki entry/baslik, enter,l: secim"
                    ))
                    .render(t, area);
            }
        }
    }

    fn draw_entry(&self, t: &mut Terminal<MouseBackend>, area: &Rect, entry: &Entry, index: usize) {
        let title_style = if self.entry_current == index {
            Style::default().fg(Color::Magenta).modifier(Modifier::Bold)
        } else {
            Style::default().fg(Color::Magenta)
        };

        Block::default()
            .borders(Borders::BOTTOM)
            .title(&format!("{}.", index + 1))
            .title_style(title_style)
            .render(t, area);

        Group::default()
            .direction(Direction::Vertical)
            .sizes(&vec![Size::Percent(99), Size::Fixed(1)])
            .margin(1)
            .render(t, area, |t, chunks| {
                // Entry
                Paragraph::default()
                    .wrap(true)
                    .text(&entry.text)
                    .render(t, &chunks[0]);

                Paragraph::default()
                    .wrap(false)
                    .text(&text_right_aligned(
                        &format!(
                            "({author}, {date})",
                            author = entry.author.name,
                            date = entry.date
                        ),
                        &chunks[1],
                    ))
                    .render(t, &chunks[1]);
            });
    }
}

fn text_right_aligned(text: &str, rect: &Rect) -> String {
    let width = rect.width;
    let spaces = width as usize - text.len();

    (" ".repeat(spaces) + &text)
}

fn init_events() -> (mpsc::Sender<Event>, mpsc::Receiver<Event>) {
    let (sender, receiver) = mpsc::channel();
    let input_sender = sender.clone();

    thread::spawn(move || {
        for c in io::stdin().keys() {
            let key = c.unwrap();
            input_sender.send(Event::Input(key)).unwrap();
        }
    });

    (sender, receiver)
}

fn main() {
    let mut term = Terminal::new(MouseBackend::new().unwrap()).unwrap();
    term.clear().unwrap();
    term.hide_cursor().unwrap();

    let mut app = App {
        size: Rect::default(),
        entries: vec![],
        titles: vec![],
        popular: true,
        title: None,
        title_selected: 0,
        title_current: 0,
        title_page: 0,
        entry_current: 0,
        entry_page: 0,
        panel_current: Panel::Titles,
        show_mode: ShowMode::EntryList,
        mode: Mode::Normal,
        command_buffer: String::new(),
        info_buffer: String::new(),
    };

    let (_sender, receiver) = init_events();

    // Load popular titles
    app.title_page += 1;
    app.titles.append(&mut eksi::popular_titles(app.title_page));
    app.draw_ui(&mut term).expect("Something went wrong.");

    loop {
        match app.mode {
            Mode::Normal => {
                match receiver.recv().unwrap() {
                    Event::Input(Key::Char('q')) => {
                        // Simply quit
                        break;
                    },
                    Event::Input(Key::Down) | Event::Input(Key::Char('j')) => {
                        match app.panel_current {
                            Panel::Titles => {
                                // Select next title
                                if app.title_selected >= app.titles.len() {
                                    // Load next titles
                                    app.title_page += 1;
                                    app.titles.append(&mut eksi::popular_titles(app.title_page));
                                }
                                app.title_selected += 1;
                            },
                            Panel::Entries => {
                                // Select next entry (infinitely)
                                app.entry_current += 1; // FIXME: check if we are at the end of that title

                                if app.entry_current >= app.entries.len() {
                                    app.entry_page += 1;
                                    let mut title = app.title
                                        .as_ref()
                                        .expect("app.title is None");

                                    // If the title is accessed trough Popular's
                                    // it will have a popular_count, so we can
                                    // safely get popular entries
                                    let mut entries = if title.popular_count.is_some() {
                                        title.entries(app.entry_page, app.popular)
                                    } else {
                                        title.entries(app.entry_page, false)
                                    };

                                    app.entries.append(&mut entries);
                                }
                            },
                        }
                    },
                    Event::Input(Key::Up) | Event::Input(Key::Char('k')) => {
                        match app.panel_current {
                            Panel::Titles => {
                                // Select prev title
                                app.title_selected -= 1;
                            },
                            Panel::Entries => {
                                // Select prev entry
                                if app.entry_current > 0 {
                                    app.entry_current -= 1;
                                }
                            }
                        }
                    },
                    Event::Input(Key::Right) | Event::Input(Key::Char('\n'))
                        | Event::Input(Key::Char('l')) => {
                        match app.panel_current {
                            Panel::Titles => {
                                // Clean up
                                // Change the title to selected one
                                let entries = app.titles[app.title_selected]
                                    .entries(app.entry_page, app.popular);

                                app.update_entries(entries);

                                app.title_current = app.title_selected;
                                app.title = Some(app.titles[app.title_selected].clone());

                                app.focus_entries();
                            },
                            Panel::Entries => {
                                // Go into single entry mode
                                app.show_mode = ShowMode::SingleEntry;
                            }
                        }
                    },
                    Event::Input(Key::Left) | Event::Input(Key::Esc)
                        | Event::Input(Key::Char('h')) => {
                        match app.panel_current {
                            Panel::Titles => {},
                            Panel::Entries => match app.show_mode {
                                ShowMode::SingleEntry => {
                                    app.show_mode = ShowMode::EntryList;
                                },
                                ShowMode::EntryList => {
                                    app.focus_titles();
                                }
                            }
                        }
                    },
                    Event::Input(Key::Char('/')) => {
                        // Open search
                        app.mode = Mode::CommandInput;
                        app.command_buffer = "/".to_string();
                    },
                    Event::Input(Key::Char('\t')) => {
                        // (Keys::Tab) Cycle trough panels
                        app.panel_current = match app.panel_current {
                            Panel::Titles => Panel::Entries,
                            Panel::Entries => Panel::Titles
                        };
                    },
                    _ => {
                        // FIXME: add not defined warning
                    }
                }
            },
            Mode::CommandInput => {
                match receiver.recv().unwrap() {
                    Event::Input(Key::Esc) => {
                        // Quit CommandInput
                        app.mode = Mode::Normal;
                        app.command_buffer.clear();
                    },
                    Event::Input(Key::Char('\n')) => {
                        // (Key::Enter) Execute command
                        app.execute_command();
                    },
                    Event::Input(Key::Char(ch)) => {
                        // Push chars to command buffer
                        app.command_buffer.push(ch);
                    },
                    Event::Input(Key::Backspace) => {
                        // Pop one char from command buffer
                        app.command_buffer.pop();
                    },
                    _ => {}
                }
            }
        }


        app.update_size(&mut term);
        app.draw_ui(&mut term).expect("Something went wrong.");
        app.info_buffer.clear();
    }

    // Clean up
    term.show_cursor().unwrap();
    term.clear().unwrap();
}
