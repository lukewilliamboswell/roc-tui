use roc_std::RocStr;

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
    let window_bounds = crate::glue::Bounds {
        height: size.height,
        width: size.width,
    };

    // Initialise Roc app
    let mut elems : roc_std::RocList<crate::glue::Elem>;
    let (mut model, _) = crate::roc::init_and_render(window_bounds);

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
                    let event = crate::glue::Event::KeyPressed(kc);
                    model = crate::roc::update(model, event);
                }
            }
            InputEvent::FocusGained => {
                let event = crate::glue::Event::FocusGained;
                model = crate::roc::update(model, event);
            }
            InputEvent::FocusLost => {
                let event = crate::glue::Event::FocusLost;
                model = crate::roc::update(model, event);
            }
            InputEvent::Paste(contents) => {
                let roc_string = roc_std::RocStr::from(&contents[..]);
                let event = crate::glue::Event::Paste(roc_string);
                model = crate::roc::update(model, event);
            }
            InputEvent::Resize(column, row) => {
                let window_bounds = crate::glue::Bounds {
                    height: column,
                    width: row,
                };
                let event = crate::glue::Event::Resize(window_bounds);
                model = crate::roc::update(model, event);
            }
            InputEvent::Tick => {
                let event = crate::glue::Event::Tick;
                (model, elems) = crate::roc::update_and_render(model, event);

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

fn get_cursor(cursor : crate::glue::Cursor) -> Option<(u16, u16)> {
    match cursor.discriminant() {
        crate::glue::discriminant_Cursor::Hidden => None,
        crate::glue::discriminant_Cursor::At => {
            let cp =  unsafe { cursor.as_At() };
            Some((cp.col,cp.row))
        },
    }
}

fn render_widget<B: tui::backend::Backend>(
    f: &mut tui::Frame<B>,
    area: tui::layout::Rect,
    elem: &crate::glue::Elem,
) {
    match elem.discriminant() {
        crate::glue::discriminant_Elem::Paragraph => render_paragraph(f, area, elem),
        crate::glue::discriminant_Elem::Layout => render_layout(f, area, elem),
        crate::glue::discriminant_Elem::Block => render_block(f, area, elem),
        crate::glue::discriminant_Elem::ListItems => render_list(f, area, elem),
    }
}

fn render_layout<B: tui::backend::Backend>(
    f: &mut tui::Frame<B>,
    area: tui::layout::Rect,
    layout: &crate::glue::Elem,
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
    paragraph: &crate::glue::Elem,
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
    block: &crate::glue::Elem,
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
    list: &crate::glue::Elem,
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

fn get_style(roc_style: &crate::glue::Style) -> tui::style::Style {
    let mut style = tui::style::Style::default();

    if roc_style.bg.discriminant() != crate::glue::discriminant_Color::Default {
        style = style.bg(get_color(roc_style.bg));
    }

    if roc_style.fg.discriminant() != crate::glue::discriminant_Color::Default {
        style = style.fg(get_color(roc_style.fg));
    }

    let mut modifiers = tui::style::Modifier::empty();
    for modifier in &roc_style.modifiers {
        match modifier {
            crate::glue::TextModifier::Bold => {
                modifiers.insert(tui::style::Modifier::BOLD);
            }
            crate::glue::TextModifier::CrossedOut => {
                modifiers.insert(tui::style::Modifier::CROSSED_OUT);
            }
            crate::glue::TextModifier::Dim => {
                modifiers.insert(tui::style::Modifier::DIM);
            }
            crate::glue::TextModifier::Hidden => {
                modifiers.insert(tui::style::Modifier::HIDDEN);
            }
            crate::glue::TextModifier::Italic => {
                modifiers.insert(tui::style::Modifier::ITALIC);
            }
            crate::glue::TextModifier::RapidBlink => {
                modifiers.insert(tui::style::Modifier::RAPID_BLINK);
            }
            crate::glue::TextModifier::Reversed => {
                modifiers.insert(tui::style::Modifier::REVERSED);
            }
            crate::glue::TextModifier::SlowBlink => {
                modifiers.insert(tui::style::Modifier::SLOW_BLINK);
            }
            crate::glue::TextModifier::Underlined => {
                modifiers.insert(tui::style::Modifier::UNDERLINED);
            }
        }
    }
    style = style.add_modifier(modifiers);

    style
}

fn get_color(color: crate::glue::Color) -> tui::style::Color {
    match color.discriminant() {
        crate::glue::discriminant_Color::Black => tui::style::Color::Black,
        crate::glue::discriminant_Color::Blue => tui::style::Color::Blue,
        crate::glue::discriminant_Color::Cyan => tui::style::Color::Cyan,
        crate::glue::discriminant_Color::DarkGray => tui::style::Color::DarkGray,
        crate::glue::discriminant_Color::Default => tui::style::Color::Reset,
        crate::glue::discriminant_Color::Gray => tui::style::Color::Gray,
        crate::glue::discriminant_Color::Green => tui::style::Color::Green,
        crate::glue::discriminant_Color::Indexed => {
            let color = unsafe { color.into_Indexed() };
            tui::style::Color::Indexed(color)
        },
        crate::glue::discriminant_Color::LightBlue => tui::style::Color::LightBlue,
        crate::glue::discriminant_Color::LightCyan => tui::style::Color::LightCyan,
        crate::glue::discriminant_Color::LightGreen  => tui::style::Color::LightGreen,
        crate::glue::discriminant_Color::LightMagenta  => tui::style::Color::LightMagenta,
        crate::glue::discriminant_Color::LightRed  => tui::style::Color::LightRed,
        crate::glue::discriminant_Color::LightYellow  => tui::style::Color::LightYellow,
        crate::glue::discriminant_Color::Magenta  => tui::style::Color::Magenta,
        crate::glue::discriminant_Color::Red  => tui::style::Color::Red,
        crate::glue::discriminant_Color::Rgb  => {
            let color = unsafe { color.into_Rgb() };
            tui::style::Color::Rgb(color.0, color.1, color.2)
        },
        crate::glue::discriminant_Color::White  => tui::style::Color::White,
        crate::glue::discriminant_Color::Yellow  => tui::style::Color::Yellow,
    }
}

fn get_alignment(roc_alignment: crate::glue::Alignment) -> tui::layout::Alignment {
    match roc_alignment {
        crate::glue::Alignment::Left => tui::layout::Alignment::Left,
        crate::glue::Alignment::Center => tui::layout::Alignment::Center,
        crate::glue::Alignment::Right => tui::layout::Alignment::Right,
    }
}

fn get_border_type(rbt: crate::glue::BorderType) -> tui::widgets::BorderType {
    match rbt {
        crate::glue::BorderType::Plain => tui::widgets::BorderType::Plain,
        crate::glue::BorderType::Rounded => tui::widgets::BorderType::Rounded,
        crate::glue::BorderType::Double => tui::widgets::BorderType::Double,
        crate::glue::BorderType::Thick => tui::widgets::BorderType::Thick,
    }
}

fn get_borders(rb: &roc_std::RocList<crate::glue::BorderModifier>) -> tui::widgets::Borders {
    let mut borders = tui::widgets::Borders::empty();
    for border in rb {
        match border {
            crate::glue::BorderModifier::All => borders.insert(tui::widgets::Borders::ALL),
            crate::glue::BorderModifier::Bottom => borders.insert(tui::widgets::Borders::BOTTOM),
            crate::glue::BorderModifier::Left => borders.insert(tui::widgets::Borders::LEFT),
            crate::glue::BorderModifier::None => borders.insert(tui::widgets::Borders::NONE),
            crate::glue::BorderModifier::Right => borders.insert(tui::widgets::Borders::RIGHT),
            crate::glue::BorderModifier::Top => borders.insert(tui::widgets::Borders::TOP),
        }
    }
    borders
}

fn get_constraints(rc: &roc_std::RocList<crate::glue::Constraint>) -> Vec<tui::layout::Constraint> {
    let mut constraints: Vec<tui::layout::Constraint> = Vec::with_capacity(rc.len());
    for constraint in rc {
        match constraint.discriminant() {
            crate::glue::discriminant_Constraint::Length => {
                let l = unsafe { constraint.as_Length() };
                constraints.push(tui::layout::Constraint::Length(*l));
            }
            crate::glue::discriminant_Constraint::Max => {
                let l = unsafe { constraint.as_Max() };
                constraints.push(tui::layout::Constraint::Max(*l));
            }
            crate::glue::discriminant_Constraint::Min => {
                let l = unsafe { constraint.as_Min() };
                constraints.push(tui::layout::Constraint::Min(*l));
            }
            crate::glue::discriminant_Constraint::Percentage => {
                let l = unsafe { constraint.as_Percentage() };
                constraints.push(tui::layout::Constraint::Percentage(*l));
            }
            crate::glue::discriminant_Constraint::Ratio => {
                let (r1, r2) = unsafe { constraint.as_Ratio() };
                constraints.push(tui::layout::Constraint::Ratio(*r1, *r2));
            }
        }
    }
    constraints
}

fn get_layout_direction(direction: crate::glue::LayoutDirection) -> tui::layout::Direction {
    match direction {
        crate::glue::LayoutDirection::Horizontal => tui::layout::Direction::Horizontal,
        crate::glue::LayoutDirection::Vertical => tui::layout::Direction::Vertical,
    }
}

fn get_key_code(event: crossterm::event::KeyCode) -> crate::glue::KeyCode {
    match event {
        crossterm::event::KeyCode::BackTab => crate::glue::KeyCode::BackTab,
        crossterm::event::KeyCode::Backspace => crate::glue::KeyCode::Backspace,
        crossterm::event::KeyCode::CapsLock => crate::glue::KeyCode::CapsLock,
        crossterm::event::KeyCode::Delete => crate::glue::KeyCode::Delete,
        crossterm::event::KeyCode::Down => crate::glue::KeyCode::Down,
        crossterm::event::KeyCode::End => crate::glue::KeyCode::End,
        crossterm::event::KeyCode::Enter => crate::glue::KeyCode::Enter,
        crossterm::event::KeyCode::Esc => crate::glue::KeyCode::Esc,
        crossterm::event::KeyCode::F(number) => crate::glue::KeyCode::Function(number),
        crossterm::event::KeyCode::Home => crate::glue::KeyCode::Home,
        crossterm::event::KeyCode::Insert => crate::glue::KeyCode::Insert,
        crossterm::event::KeyCode::KeypadBegin => crate::glue::KeyCode::KeypadBegin,
        crossterm::event::KeyCode::Left => crate::glue::KeyCode::Left,
        crossterm::event::KeyCode::Media(mk) => {
            crate::glue::KeyCode::Media(get_key_media(mk))
        }
        crossterm::event::KeyCode::Menu => crate::glue::KeyCode::Menu,
        crossterm::event::KeyCode::Modifier(km) => {
            crate::glue::KeyCode::Modifier(get_key_modifier(km))
        }
        crossterm::event::KeyCode::Null => crate::glue::KeyCode::Null,
        crossterm::event::KeyCode::NumLock => crate::glue::KeyCode::NumLock,
        crossterm::event::KeyCode::PageDown => crate::glue::KeyCode::PageDown,
        crossterm::event::KeyCode::PageUp => crate::glue::KeyCode::PageUp,
        crossterm::event::KeyCode::Pause => crate::glue::KeyCode::Pause,
        crossterm::event::KeyCode::PrintScreen => crate::glue::KeyCode::PrintScreen,
        crossterm::event::KeyCode::Right => crate::glue::KeyCode::Right,
        crossterm::event::KeyCode::Char(ch) => {
            let string = String::from(ch);
            let roc_string = roc_std::RocStr::from(&string[..]);
            crate::glue::KeyCode::Scalar(roc_string)
        }
        crossterm::event::KeyCode::ScrollLock => crate::glue::KeyCode::ScrollLock,
        crossterm::event::KeyCode::Tab => crate::glue::KeyCode::Tab,
        crossterm::event::KeyCode::Up => crate::glue::KeyCode::Up,
    }
}

fn get_key_modifier(km: crossterm::event::ModifierKeyCode) -> crate::glue::ModifierKeyCode {
    match km {
        crossterm::event::ModifierKeyCode::IsoLevel3Shift => {
            crate::glue::ModifierKeyCode::IsoLevel3Shift
        }
        crossterm::event::ModifierKeyCode::IsoLevel5Shift => {
            crate::glue::ModifierKeyCode::IsoLevel5Shift
        }
        crossterm::event::ModifierKeyCode::LeftAlt => crate::glue::ModifierKeyCode::LeftAlt,
        crossterm::event::ModifierKeyCode::LeftControl => crate::glue::ModifierKeyCode::LeftControl,
        crossterm::event::ModifierKeyCode::LeftHyper => crate::glue::ModifierKeyCode::LeftHyper,
        crossterm::event::ModifierKeyCode::LeftMeta => crate::glue::ModifierKeyCode::LeftMeta,
        crossterm::event::ModifierKeyCode::LeftShift => crate::glue::ModifierKeyCode::LeftShift,
        crossterm::event::ModifierKeyCode::LeftSuper => crate::glue::ModifierKeyCode::LeftSuper,
        crossterm::event::ModifierKeyCode::RightAlt => crate::glue::ModifierKeyCode::RightAlt,
        crossterm::event::ModifierKeyCode::RightControl => {
            crate::glue::ModifierKeyCode::RightControl
        }
        crossterm::event::ModifierKeyCode::RightHyper => crate::glue::ModifierKeyCode::RightHyper,
        crossterm::event::ModifierKeyCode::RightMeta => crate::glue::ModifierKeyCode::RightMeta,
        crossterm::event::ModifierKeyCode::RightShift => crate::glue::ModifierKeyCode::RightShift,
        crossterm::event::ModifierKeyCode::RightSuper => crate::glue::ModifierKeyCode::RightSuper,
    }
}

fn get_key_media(mk: crossterm::event::MediaKeyCode) -> crate::glue::MediaKeyCode {
    match mk {
        crossterm::event::MediaKeyCode::Play => crate::glue::MediaKeyCode::Play,
        crossterm::event::MediaKeyCode::Pause => crate::glue::MediaKeyCode::Pause,
        crossterm::event::MediaKeyCode::PlayPause => crate::glue::MediaKeyCode::PlayPause,
        crossterm::event::MediaKeyCode::Reverse => crate::glue::MediaKeyCode::Reverse,
        crossterm::event::MediaKeyCode::Stop => crate::glue::MediaKeyCode::Stop,
        crossterm::event::MediaKeyCode::FastForward => crate::glue::MediaKeyCode::FastForward,
        crossterm::event::MediaKeyCode::Rewind => crate::glue::MediaKeyCode::Rewind,
        crossterm::event::MediaKeyCode::TrackNext => crate::glue::MediaKeyCode::TrackNext,
        crossterm::event::MediaKeyCode::TrackPrevious => crate::glue::MediaKeyCode::TrackPrevious,
        crossterm::event::MediaKeyCode::Record => crate::glue::MediaKeyCode::Record,
        crossterm::event::MediaKeyCode::LowerVolume => crate::glue::MediaKeyCode::LowerVolume,
        crossterm::event::MediaKeyCode::RaiseVolume => crate::glue::MediaKeyCode::RaiseVolume,
        crossterm::event::MediaKeyCode::MuteVolume => crate::glue::MediaKeyCode::MuteVolume,
    }
}

fn get_croll(scroll: u16) -> (u16, u16) {

    // TODO the following will crash if you give it too large a value
    // this is a workaround to stop rust panicking if the scroll is too large
    // happens at the following location in tui-rs
    // Line 192 https://github.com/fdehau/tui-rs/src/widgets/paragraph.rs
    if scroll > 65000 {
        return (0, 0);
    }

    // TODO investigate why scrolling the column doesn't do anything.. 
    (scroll, 0)
}

fn get_corner(corner : &crate::glue::Corner) -> tui::layout::Corner {
    match corner {
        crate::glue::Corner::BottomLeft => tui::layout::Corner::BottomLeft,
        crate::glue::Corner::BottomRight => tui::layout::Corner::BottomRight,
        crate::glue::Corner::TopLeft => tui::layout::Corner::TopLeft,
        crate::glue::Corner::TopRight => tui::layout::Corner::TopRight,
    }
}

fn get_list_selection(selection : &crate::glue::ListSelection) -> Option<usize> {
    match selection.discriminant() {
        crate::glue::discriminant_ListSelection::None => None,
        crate::glue::discriminant_ListSelection::Selected =>{
            let s = unsafe { selection.into_Selected() };
            Some(s as usize)
        },
    }
}

fn get_popup(popup : &crate::glue::PopupConfig) -> Option<(u16,u16)>{
    match popup.discriminant() {
        crate::glue::discriminant_PopupConfig::None => None,
        crate::glue::discriminant_PopupConfig::Centered =>{
            let pos = unsafe { popup.into_Centered() };
            Some((pos.percentX, pos.percentY))
        },
    }
}
