use roc_std::{RocStr, RocList};
use crate::glue;
use crate::roc;

const SCREEN_DRAW_RATE_MS: u64 = 50;

pub fn run_event_loop() {

    // Setup terminal
    crossterm::terminal::enable_raw_mode().expect("TODO handle enabling Raw mode on terminal");
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )
    .expect("TODO handle entering alternate screen and enabling mouse capture on terminal");
    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal =
        tui::Terminal::new(backend).expect("TODO handle unable to create crossterm backend");
    let tick_rate = std::time::Duration::from_millis(SCREEN_DRAW_RATE_MS);
    let events = Events::new(tick_rate);
    let size = terminal.size().expect("TODO unable to get frame size");
    let window_bounds = glue::Bounds {
        height: size.height,
        width: size.width,
    };

    // Initialise Roc app
    let mut elems : RocList<glue::Elem>;
    let (mut model, _) = roc::init_and_render(window_bounds);

    loop {
        let mut app_return = false;

        // Handle any events
        match events
            .next()
            .expect("TODO handle unable to spawn event thread")
        {
            InputEvent::KeyPressed(key) => {
                if key.code == crossterm::event::KeyCode::Esc {
                    // TODO don't hardcode the escape
                    app_return = true;
                } else {
                    let kc = get_key_code(key.code);
                    let event = glue::Event::KeyPressed(kc);
                    model = roc::update(model, event);
                }
            }
            InputEvent::FocusGained => {
                let event = glue::Event::FocusGained;
                model = roc::update(model, event);
            }
            InputEvent::FocusLost => {
                let event = glue::Event::FocusLost;
                model = roc::update(model, event);
            }
            InputEvent::Paste(contents) => {
                let roc_string = roc_std::RocStr::from(&contents[..]);
                let event = glue::Event::Paste(roc_string);
                model = roc::update(model, event);
            }
            InputEvent::Resize(column, row) => {
                let window_bounds = glue::Bounds {
                    height: column,
                    width: row,
                };
                let event = glue::Event::Resize(window_bounds);
                model = roc::update(model, event);
            }
            InputEvent::Tick => {
                let event = glue::Event::Tick;
                (model, elems) = roc::update_and_render(model, event);

                // Draw the widgets
                terminal
                    .draw(|f| {
                        for elem in &elems {
                            render_widget(f, f.size(), &elem)
                        }
                    })
                    .expect("Err: Unable to draw to terminal.");
                }
        };

        if app_return {
            break;
        }
    }

    // restore terminal
    crossterm::terminal::disable_raw_mode()
        .expect("TODO handle unable to disable Raw mode on terminal");
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )
    .expect("TODO handle unable to leave alternate screen or disable mouse capture");

}

pub enum InputEvent {
    KeyPressed(crossterm::event::KeyEvent),
    FocusGained,
    FocusLost,
    // TODO Mouse(MouseEvent),
    Paste(String),
    Resize(u16, u16), // column, row
    Tick,
}

pub struct Events {
    rx: std::sync::mpsc::Receiver<InputEvent>,
    _tx: std::sync::mpsc::Sender<InputEvent>,
}

impl Events {
    pub fn new(tick_rate: std::time::Duration) -> Events {
        let (tx, rx) = std::sync::mpsc::channel();

        let event_tx = tx.clone(); // the thread::spawn own event_tx
        std::thread::spawn(move || {
            loop {
                // poll for tick rate duration, if no event, sent tick event.
                if crossterm::event::poll(tick_rate)
                    .expect("TODO handle unable to poll for crossterm events")
                {
                    match crossterm::event::read().expect(
                        "TODO handle unable to read crossterm events, this shouldn't happen",
                    ) {
                        crossterm::event::Event::Key(key) => {
                            let key = crossterm::event::KeyEvent::from(key);
                            event_tx
                                .send(InputEvent::KeyPressed(key))
                                .expect("TODO hangle unable to send keypress event to channel");
                        }
                        crossterm::event::Event::FocusGained => {
                            event_tx
                                .send(InputEvent::FocusGained)
                                .expect("TODO hangle unable to send focus gained event to channel");
                        }
                        crossterm::event::Event::FocusLost => {
                            event_tx
                                .send(InputEvent::FocusLost)
                                .expect("TODO hangle unable to send focus lost event to channel");
                        }
                        crossterm::event::Event::Mouse(_) => {
                            // TODO support mouse stuff
                        }
                        crossterm::event::Event::Paste(contents) => {
                            event_tx
                                .send(InputEvent::Paste(contents))
                                .expect("TODO hangle unable to send paste event to channel");
                        }
                        crossterm::event::Event::Resize(column, row) => {
                            event_tx
                                .send(InputEvent::Resize(column, row))
                                .expect("TODO hangle unable to send resize event to channel");
                        }
                    }
                }
                event_tx
                    .send(InputEvent::Tick)
                    .expect("TODO hangle unable to send tick event to channel");
            }
        });

        Events { rx, _tx: tx }
    }

    /// Attempts to read an event.
    /// This function block the current thread.
    pub fn next(&self) -> Result<InputEvent, std::sync::mpsc::RecvError> {
        self.rx.recv()
    }
}

fn get_cursor(cursor : glue::Cursor) -> Option<(u16, u16)> {
    match cursor.discriminant() {
        glue::discriminant_Cursor::Hidden => None,
        glue::discriminant_Cursor::At => {
            let cp =  unsafe { cursor.as_At() };
            Some((cp.col,cp.row))
        },
    }
}

fn render_widget<B: tui::backend::Backend>(
    f: &mut tui::Frame<B>,
    area: tui::layout::Rect,
    elem: &glue::Elem,
) {
    match elem.discriminant() {
        glue::discriminant_Elem::Paragraph => render_paragraph(f, area, elem),
        glue::discriminant_Elem::Layout => render_layout(f, area, elem),
        glue::discriminant_Elem::Block => render_block(f, area, elem),
        glue::discriminant_Elem::ListItems => render_list(f, area, elem),
    }
}

fn render_layout<B: tui::backend::Backend>(
    f: &mut tui::Frame<B>,
    area: tui::layout::Rect,
    layout: &glue::Elem,
) {
    let (elems, config) = unsafe { layout.as_Layout() };
    let layout_direction = get_layout_direction(config.direction);
    let mut constraints = get_constraints(&config.constraints);

    // Handle popup behaviour
    let popup = get_popup(&config.popup);
    let area2 : tui::layout::Rect;
    match popup {
        None => {
            area2 = area;
        },
        Some ((x,y)) => {

            // calculate popup from area
            area2 = centered_rect(x, y, area);

            // clear the background
            f.render_widget(tui::widgets::Clear, area2); 
        },
    }

    // check we have enough constriants otherwise add some default to stop tui from crashing
    while constraints.len() < elems.len() {
        constraints.push(tui::layout::Constraint::Ratio(1, 1));
    }

    let chunks = tui::layout::Layout::default()
        .direction(layout_direction)
        .horizontal_margin(config.hMargin)
        .vertical_margin(config.vMargin)
        .constraints(constraints)
        .split(area2);

    let mut chunk_index = 0;
    for elem in elems {
        render_widget(f, chunks[chunk_index], elem);
        chunk_index += 1;
    }

}


/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: tui::layout::Rect) -> tui::layout::Rect {
    let popup_layout = tui::layout::Layout::default()
        .direction(tui::layout::Direction::Vertical)
        .constraints(
            [
                tui::layout::Constraint::Percentage((100 - percent_y) / 2),
                tui::layout::Constraint::Percentage(percent_y),
                tui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    tui::layout::Layout::default()
        .direction(tui::layout::Direction::Horizontal)
        .constraints(
            [
                tui::layout::Constraint::Percentage((100 - percent_x) / 2),
                tui::layout::Constraint::Percentage(percent_x),
                tui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn render_paragraph<B: tui::backend::Backend>(
    f: &mut tui::Frame<B>,
    area: tui::layout::Rect,
    paragraph: &glue::Elem,
) {
    let config = unsafe { paragraph.as_Paragraph() };

    // Block window for the paragraph text to live in
    let borders = get_borders(&config.block.borders);
    let border_type = get_border_type(config.block.borderType);
    let border_style = get_style(&config.block.borderStyle);
    let title = tui::text::Span::styled(config.block.title.text.as_str(), get_style(&config.block.title.style));
    let title_alignment = get_alignment(config.block.titleAlignment);
    let style = &config.block.style;
    let block = tui::widgets::Block::default()
        .title(title)
        .title_alignment(title_alignment)
        .borders(borders)
        .border_style(border_style)
        .border_type(border_type)
        .style(get_style(style));

    // Build pargraph up from nested Span(s)
    let mut text = Vec::new();
    for line in &config.text {
        let mut spans_elements = Vec::new();
        for span in line {
            let s = tui::text::Span::styled(span.text.as_str(), get_style(&span.style));
            spans_elements.push(s);
        }
        let spans = tui::text::Spans::from(spans_elements);
        text.push(spans);
    }

    // Create the paragraph
    let text_alignment = get_alignment(config.textAlignment);
    let scroll = get_croll(config.scroll);
    let p = tui::widgets::Paragraph::new(text)
        .block(block)
        .scroll(scroll)
        .wrap(tui::widgets::Wrap { trim: true })
        .alignment(text_alignment);

    // Show the cursor if required
    let cursor = get_cursor(config.cursor);
    match cursor {
        None => {
            // will be hidden if not set
        },
        Some((x,y)) => {
            f.set_cursor(area.x + x,area.y + y);
        },
    }

    // Render to the frame
    f.render_widget(p, area);
}

fn render_block<B: tui::backend::Backend>(
    f: &mut tui::Frame<B>,
    area: tui::layout::Rect,
    block: &glue::Elem,
) {
    let config = unsafe { block.as_Block() };

    // Block window for the paragraph text to live in
    let borders = get_borders(&config.borders);
    let border_style = get_style(&config.borderStyle);
    let border_type = get_border_type(config.borderType);
    let title = tui::text::Span::styled(config.title.text.as_str(), get_style(&config.title.style));
    let title_alignment = get_alignment(config.titleAlignment);
    let style = &config.style;
    let block = tui::widgets::Block::default()
        .title(title)
        .title_alignment(title_alignment)
        .borders(borders)
        .border_style(border_style)
        .border_type(border_type)
        .style(get_style(style));

    f.render_widget(block, area);

    // TODO Can we refactor out Block to re-use in all the other widgets?
    // this will require creating it separately to rendering. Might change the
    // way we recursively do layouts. Requires further investigation.
}

fn render_list<B: tui::backend::Backend>(
    f: &mut tui::Frame<B>,
    area: tui::layout::Rect,
    list: &glue::Elem,
) {
    let config = unsafe { list.as_ListItems() };

    // Block window for the list to live in
    let borders = get_borders(&config.block.borders);
    let border_type = get_border_type(config.block.borderType);
    let border_style = get_style(&config.block.borderStyle);
    let title = tui::text::Span::styled(config.block.title.text.as_str(), get_style(&config.block.title.style));
    let title_alignment = get_alignment(config.block.titleAlignment);
    let style = &config.block.style;
    let block = tui::widgets::Block::default()
        .title(title)
        .title_alignment(title_alignment)
        .borders(borders)
        .border_style(border_style)
        .border_type(border_type);

    // Build list items up from nested Span(s)
    let mut items = Vec::new();
    for line in &config.items {
        let mut spans_elements = Vec::new();
        for span in line {
            let s = tui::text::Span::styled(span.text.as_str(), get_style(&span.style));
            spans_elements.push(s);
        }
        let spans = tui::text::Spans::from(spans_elements);
        let list_item = tui::widgets::ListItem::new(spans);
        items.push(list_item);
    }

    let highlight_symbol =  RocStr::as_str(&config.highlightSymbol);
    let start_corner = get_corner(&config.startCorner);
    let list = tui::widgets::List::new(items)
        .block(block)
        .style(get_style(style))
        .highlight_style(get_style(&config.highlightStyle))
        .highlight_symbol(highlight_symbol)
        .repeat_highlight_symbol(config.highlightSymbolRepeat)
        .start_corner(start_corner);

    let selection = get_list_selection(&config.selected);
    let mut list_state = tui::widgets::ListState::default();
    list_state.select(selection);
        
    // Render to the frame
    f.render_stateful_widget(list, area, &mut list_state);

}

fn get_style(roc_style: &glue::Style) -> tui::style::Style {
    let mut style = tui::style::Style::default();

    if roc_style.bg.discriminant() != glue::discriminant_Color::Default {
        style = style.bg(get_color(roc_style.bg));
    }

    if roc_style.fg.discriminant() != glue::discriminant_Color::Default {
        style = style.fg(get_color(roc_style.fg));
    }

    let mut modifiers = tui::style::Modifier::empty();
    for modifier in &roc_style.modifiers {
        match modifier {
            glue::TextModifier::Bold => modifiers.insert(tui::style::Modifier::BOLD),
            glue::TextModifier::CrossedOut => modifiers.insert(tui::style::Modifier::CROSSED_OUT),
            glue::TextModifier::Dim => modifiers.insert(tui::style::Modifier::DIM),
            glue::TextModifier::Hidden => modifiers.insert(tui::style::Modifier::HIDDEN),
            glue::TextModifier::Italic => modifiers.insert(tui::style::Modifier::ITALIC),
            glue::TextModifier::RapidBlink => modifiers.insert(tui::style::Modifier::RAPID_BLINK),
            glue::TextModifier::Reversed => modifiers.insert(tui::style::Modifier::REVERSED),
            glue::TextModifier::SlowBlink => modifiers.insert(tui::style::Modifier::SLOW_BLINK),
            glue::TextModifier::Underlined => modifiers.insert(tui::style::Modifier::UNDERLINED),
        }
    }
    style = style.add_modifier(modifiers);

    style
}

fn get_color(color: glue::Color) -> tui::style::Color {
    match color.discriminant() {
        glue::discriminant_Color::Black => tui::style::Color::Black,
        glue::discriminant_Color::Blue => tui::style::Color::Blue,
        glue::discriminant_Color::Cyan => tui::style::Color::Cyan,
        glue::discriminant_Color::DarkGray => tui::style::Color::DarkGray,
        glue::discriminant_Color::Default => tui::style::Color::Reset,
        glue::discriminant_Color::Gray => tui::style::Color::Gray,
        glue::discriminant_Color::Green => tui::style::Color::Green,
        glue::discriminant_Color::Indexed => {
            let color = unsafe { color.into_Indexed() };
            tui::style::Color::Indexed(color)
        },
        glue::discriminant_Color::LightBlue => tui::style::Color::LightBlue,
        glue::discriminant_Color::LightCyan => tui::style::Color::LightCyan,
        glue::discriminant_Color::LightGreen  => tui::style::Color::LightGreen,
        glue::discriminant_Color::LightMagenta  => tui::style::Color::LightMagenta,
        glue::discriminant_Color::LightRed  => tui::style::Color::LightRed,
        glue::discriminant_Color::LightYellow  => tui::style::Color::LightYellow,
        glue::discriminant_Color::Magenta  => tui::style::Color::Magenta,
        glue::discriminant_Color::Red  => tui::style::Color::Red,
        glue::discriminant_Color::Rgb  => {
            let color = unsafe { color.into_Rgb() };
            tui::style::Color::Rgb(color.0, color.1, color.2)
        },
        glue::discriminant_Color::White  => tui::style::Color::White,
        glue::discriminant_Color::Yellow  => tui::style::Color::Yellow,
    }
}

fn get_alignment(roc_alignment: glue::Alignment) -> tui::layout::Alignment {
    match roc_alignment {
        glue::Alignment::Left => tui::layout::Alignment::Left,
        glue::Alignment::Center => tui::layout::Alignment::Center,
        glue::Alignment::Right => tui::layout::Alignment::Right,
    }
}

fn get_border_type(rbt: glue::BorderType) -> tui::widgets::BorderType {
    match rbt {
        glue::BorderType::Plain => tui::widgets::BorderType::Plain,
        glue::BorderType::Rounded => tui::widgets::BorderType::Rounded,
        glue::BorderType::Double => tui::widgets::BorderType::Double,
        glue::BorderType::Thick => tui::widgets::BorderType::Thick,
    }
}

fn get_borders(rb: &RocList<glue::BorderModifier>) -> tui::widgets::Borders {
    let mut borders = tui::widgets::Borders::empty();
    for border in rb {
        match border {
            glue::BorderModifier::All => borders.insert(tui::widgets::Borders::ALL),
            glue::BorderModifier::Bottom => borders.insert(tui::widgets::Borders::BOTTOM),
            glue::BorderModifier::Left => borders.insert(tui::widgets::Borders::LEFT),
            glue::BorderModifier::None => borders.insert(tui::widgets::Borders::NONE),
            glue::BorderModifier::Right => borders.insert(tui::widgets::Borders::RIGHT),
            glue::BorderModifier::Top => borders.insert(tui::widgets::Borders::TOP),
        }
    }
    borders
}

fn get_constraints(rc: &RocList<glue::Constraint>) -> Vec<tui::layout::Constraint> {
    let mut constraints: Vec<tui::layout::Constraint> = Vec::with_capacity(rc.len());
    for constraint in rc {
        match constraint.discriminant() {
            glue::discriminant_Constraint::Length => {
                let l = unsafe { constraint.as_Length() };
                constraints.push(tui::layout::Constraint::Length(*l));
            }
            glue::discriminant_Constraint::Max => {
                let l = unsafe { constraint.as_Max() };
                constraints.push(tui::layout::Constraint::Max(*l));
            }
            glue::discriminant_Constraint::Min => {
                let l = unsafe { constraint.as_Min() };
                constraints.push(tui::layout::Constraint::Min(*l));
            }
            glue::discriminant_Constraint::Percentage => {
                let l = unsafe { constraint.as_Percentage() };
                constraints.push(tui::layout::Constraint::Percentage(*l));
            }
            glue::discriminant_Constraint::Ratio => {
                let (r1, r2) = unsafe { constraint.as_Ratio() };
                constraints.push(tui::layout::Constraint::Ratio(*r1, *r2));
            }
        }
    }
    constraints
}

fn get_layout_direction(direction: glue::LayoutDirection) -> tui::layout::Direction {
    match direction {
        glue::LayoutDirection::Horizontal => tui::layout::Direction::Horizontal,
        glue::LayoutDirection::Vertical => tui::layout::Direction::Vertical,
    }
}

fn get_key_code(event: crossterm::event::KeyCode) -> glue::KeyCode {
    match event {
        crossterm::event::KeyCode::BackTab => glue::KeyCode::BackTab,
        crossterm::event::KeyCode::Backspace => glue::KeyCode::Backspace,
        crossterm::event::KeyCode::CapsLock => glue::KeyCode::CapsLock,
        crossterm::event::KeyCode::Delete => glue::KeyCode::Delete,
        crossterm::event::KeyCode::Down => glue::KeyCode::Down,
        crossterm::event::KeyCode::End => glue::KeyCode::End,
        crossterm::event::KeyCode::Enter => glue::KeyCode::Enter,
        crossterm::event::KeyCode::Esc => glue::KeyCode::Esc,
        crossterm::event::KeyCode::F(number) => glue::KeyCode::Function(number),
        crossterm::event::KeyCode::Home => glue::KeyCode::Home,
        crossterm::event::KeyCode::Insert => glue::KeyCode::Insert,
        crossterm::event::KeyCode::KeypadBegin => glue::KeyCode::KeypadBegin,
        crossterm::event::KeyCode::Left => glue::KeyCode::Left,
        crossterm::event::KeyCode::Media(mk) => glue::KeyCode::Media(get_key_media(mk)),
        crossterm::event::KeyCode::Menu => glue::KeyCode::Menu,
        crossterm::event::KeyCode::Modifier(km) => glue::KeyCode::Modifier(get_key_modifier(km)),
        crossterm::event::KeyCode::Null => glue::KeyCode::Null,
        crossterm::event::KeyCode::NumLock => glue::KeyCode::NumLock,
        crossterm::event::KeyCode::PageDown => glue::KeyCode::PageDown,
        crossterm::event::KeyCode::PageUp => glue::KeyCode::PageUp,
        crossterm::event::KeyCode::Pause => glue::KeyCode::Pause,
        crossterm::event::KeyCode::PrintScreen => glue::KeyCode::PrintScreen,
        crossterm::event::KeyCode::Right => glue::KeyCode::Right,
        crossterm::event::KeyCode::Char(ch) => {
            let string = String::from(ch);
            let roc_string = roc_std::RocStr::from(&string[..]);
            glue::KeyCode::Scalar(roc_string)
        }
        crossterm::event::KeyCode::ScrollLock => glue::KeyCode::ScrollLock,
        crossterm::event::KeyCode::Tab => glue::KeyCode::Tab,
        crossterm::event::KeyCode::Up => glue::KeyCode::Up,
    }
}

fn get_key_modifier(km: crossterm::event::ModifierKeyCode) -> glue::ModifierKeyCode {
    match km {
        crossterm::event::ModifierKeyCode::IsoLevel3Shift => glue::ModifierKeyCode::IsoLevel3Shift,
        crossterm::event::ModifierKeyCode::IsoLevel5Shift => glue::ModifierKeyCode::IsoLevel5Shift,
        crossterm::event::ModifierKeyCode::LeftAlt => glue::ModifierKeyCode::LeftAlt,
        crossterm::event::ModifierKeyCode::LeftControl => glue::ModifierKeyCode::LeftControl,
        crossterm::event::ModifierKeyCode::LeftHyper => glue::ModifierKeyCode::LeftHyper,
        crossterm::event::ModifierKeyCode::LeftMeta => glue::ModifierKeyCode::LeftMeta,
        crossterm::event::ModifierKeyCode::LeftShift => glue::ModifierKeyCode::LeftShift,
        crossterm::event::ModifierKeyCode::LeftSuper => glue::ModifierKeyCode::LeftSuper,
        crossterm::event::ModifierKeyCode::RightAlt => glue::ModifierKeyCode::RightAlt,
        crossterm::event::ModifierKeyCode::RightControl => glue::ModifierKeyCode::RightControl,
        crossterm::event::ModifierKeyCode::RightHyper => glue::ModifierKeyCode::RightHyper,
        crossterm::event::ModifierKeyCode::RightMeta => glue::ModifierKeyCode::RightMeta,
        crossterm::event::ModifierKeyCode::RightShift => glue::ModifierKeyCode::RightShift,
        crossterm::event::ModifierKeyCode::RightSuper => glue::ModifierKeyCode::RightSuper,
    }
}

fn get_key_media(mk: crossterm::event::MediaKeyCode) -> glue::MediaKeyCode {
    match mk {
        crossterm::event::MediaKeyCode::Play => glue::MediaKeyCode::Play,
        crossterm::event::MediaKeyCode::Pause => glue::MediaKeyCode::Pause,
        crossterm::event::MediaKeyCode::PlayPause => glue::MediaKeyCode::PlayPause,
        crossterm::event::MediaKeyCode::Reverse => glue::MediaKeyCode::Reverse,
        crossterm::event::MediaKeyCode::Stop => glue::MediaKeyCode::Stop,
        crossterm::event::MediaKeyCode::FastForward => glue::MediaKeyCode::FastForward,
        crossterm::event::MediaKeyCode::Rewind => glue::MediaKeyCode::Rewind,
        crossterm::event::MediaKeyCode::TrackNext => glue::MediaKeyCode::TrackNext,
        crossterm::event::MediaKeyCode::TrackPrevious => glue::MediaKeyCode::TrackPrevious,
        crossterm::event::MediaKeyCode::Record => glue::MediaKeyCode::Record,
        crossterm::event::MediaKeyCode::LowerVolume => glue::MediaKeyCode::LowerVolume,
        crossterm::event::MediaKeyCode::RaiseVolume => glue::MediaKeyCode::RaiseVolume,
        crossterm::event::MediaKeyCode::MuteVolume => glue::MediaKeyCode::MuteVolume,
    }
}

// TODO the following will crash if you give it too large a value
// this is a hacky workaround to stop rust panicking if the scroll is too large
// happens at the following location in tui-rs
// Line 192 https://github.com/fdehau/tui-rs/src/widgets/paragraph.rs
fn get_croll(scroll: u16) -> (u16, u16) {
    
    if scroll > 65000 {
        return (0, 0);
    }

    // TODO investigate why scrolling the column doesn't do anything... 
    (scroll, 0)
}

fn get_corner(corner : &glue::Corner) -> tui::layout::Corner {
    match corner {
        glue::Corner::BottomLeft => tui::layout::Corner::BottomLeft,
        glue::Corner::BottomRight => tui::layout::Corner::BottomRight,
        glue::Corner::TopLeft => tui::layout::Corner::TopLeft,
        glue::Corner::TopRight => tui::layout::Corner::TopRight,
    }
}

fn get_list_selection(selection : &glue::ListSelection) -> Option<usize> {
    match selection.discriminant() {
        glue::discriminant_ListSelection::None => None,
        glue::discriminant_ListSelection::Selected =>{
            let s = unsafe { selection.into_Selected() };
            Some(s as usize)
        },
    }
}

fn get_popup(popup : &glue::PopupConfig) -> Option<(u16,u16)>{
    match popup.discriminant() {
        glue::discriminant_PopupConfig::None => None,
        glue::discriminant_PopupConfig::Centered =>{
            let pos = unsafe { popup.into_Centered() };
            Some((pos.percentX, pos.percentY))
        },
    }
}
