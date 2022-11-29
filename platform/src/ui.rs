const SCREEN_DRAW_RATE_MS: u64 = 50;

pub fn run_event_loop(title: &str) {

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
    let (mut model, mut elems) = crate::roc::init_and_render(window_bounds);

    loop {
        let mut appReturn = false;
        let mut appRedraw = false;

        // Handle any events
        let result = match events
            .next()
            .expect("TODO handle unable to spawn event thread")
        {
            InputEvent::KeyPressed(key) => {
                if key.code == crossterm::event::KeyCode::Esc {
                    // TODO don't hardcode the escape
                    appReturn = true;
                } else {
                    let keyCode = getKeyCode(key.code);
                    let event = crate::glue::Event::KeyPressed(keyCode);
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
                // TODO add ticks?
                // Are these needed in the app??
                appRedraw = true;
            }
        };

        if appReturn {
            break;
        }

        if appRedraw {
            elems = crate::roc::render(model);

            // Draw the widgets
            terminal
                .draw(|f| {
                    for elem in &elems {
                        renderWidget(f, f.size(), &elem)
                    }
                })
                .expect("Err: Unable to draw to terminal.");
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
    terminal
        .show_cursor()
        .expect("TODO handle unable to show cursor in terminal");
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

fn getCursor(cursor : crate::glue::Cursor) -> Option<(u16, u16)> {
    match cursor.discriminant() {
        crate::glue::discriminant_Cursor::Hidden => None,
        crate::glue::discriminant_Cursor::At => {
            let cursorPos =  unsafe { cursor.as_At() };
            Some((cursorPos.col,cursorPos.row))
        },
    }
}

fn renderWidget<B: tui::backend::Backend>(
    f: &mut tui::Frame<B>,
    area: tui::layout::Rect,
    elem: &crate::glue::Elem,
) {
    match elem.discriminant() {
        crate::glue::discriminant_Elem::Paragraph => renderParagraph(f, area, elem),
        crate::glue::discriminant_Elem::Layout => renderLayout(f, area, elem),
        crate::glue::discriminant_Elem::Block => renderBlock(f, area, elem),
        crate::glue::discriminant_Elem::ListItems => {} //TODO implement renderBlock(f, area, elem),
    }
}

fn renderLayout<B: tui::backend::Backend>(
    f: &mut tui::Frame<B>,
    area: tui::layout::Rect,
    layout: &crate::glue::Elem,
) {
    let (mut elems, mut config) = unsafe { layout.as_Layout() };
    let layoutDirection = getLayoutDirection(config.direction);
    let mut constraints = getConstraints(&config.constraints);

    // check we have enough constriants otherwise add some default to stop tui from crashing
    while constraints.len() < elems.len() {
        constraints.push(tui::layout::Constraint::Ratio(1, 1));
    }

    let chunks = tui::layout::Layout::default()
        .direction(layoutDirection)
        .horizontal_margin(config.hMargin)
        .vertical_margin(config.vMargin)
        .constraints(constraints)
        .split(area);

    let mut chunkIndex = 0;
    for elem in elems {
        renderWidget(f, chunks[chunkIndex], elem);
        chunkIndex += 1;
    }
}

fn renderParagraph<B: tui::backend::Backend>(
    f: &mut tui::Frame<B>,
    area: tui::layout::Rect,
    paragraph: &crate::glue::Elem,
) {
    let config = unsafe { paragraph.as_Paragraph() };

    // Block window for the paragraph text to live in
    let borders = getBorders(&config.block.borders);
    let borderType = getBorderType(config.block.borderType);
    let borderStyle = getStyle(&config.block.borderStyle);
    let title = tui::text::Span::styled(config.block.title.text.as_str(), getStyle(&config.block.title.style));
    let titleAlignment = getAlignment(config.block.titleAlignment);
    let style = &config.block.style;
    let block = tui::widgets::Block::default()
        .title(title)
        .title_alignment(titleAlignment)
        .borders(borders)
        .border_style(borderStyle)
        .border_type(borderType)
        .style(getStyle(style));

    // Build pargraph up from nested Span(s)
    let mut text = Vec::with_capacity(config.text.len());
    let mut spansElements = Vec::with_capacity(1);
    for span in &config.text {
        let s = tui::text::Span::styled(span.text.as_str(), getStyle(&span.style));
        spansElements.push(s);
    }
    text.push(tui::text::Spans::from(spansElements));

    // Create the paragraph
    let textAlignment = getAlignment(config.textAlignment);
    let scroll = getScroll(config.scroll);
    let p = tui::widgets::Paragraph::new(text)
        .block(block)
        .scroll(scroll)
        .wrap(tui::widgets::Wrap { trim: true })
        .alignment(textAlignment);

    // Show the cursor if required
    let cursor = getCursor(config.cursor);
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

fn renderBlock<B: tui::backend::Backend>(
    f: &mut tui::Frame<B>,
    area: tui::layout::Rect,
    block: &crate::glue::Elem,
) {
    let config = unsafe { block.as_Block() };

    // Block window for the paragraph text to live in
    let borders = getBorders(&config.borders);
    let borderStyle = getStyle(&config.borderStyle);
    let borderType = getBorderType(config.borderType);
    let title = tui::text::Span::styled(config.title.text.as_str(), getStyle(&config.title.style));
    let titleAlignment = getAlignment(config.titleAlignment);
    let style = &config.style;
    let block = tui::widgets::Block::default()
        .title(title)
        .title_alignment(titleAlignment)
        .borders(borders)
        .border_style(borderStyle)
        .border_type(borderType)
        .style(getStyle(style));

    // Render to the frame
    f.render_widget(block, area);

    // TODO Can we refactor out Block to re-use in all the other widgets?
    // this will require creating it separately to rendering. Might change the
    // way we recursively do layouts. Requires further investigation.
}

fn getStyle(rocStyle: &crate::glue::Style) -> tui::style::Style {
    let mut style = tui::style::Style::default();

    if rocStyle.bg.discriminant() != crate::glue::discriminant_Color::Default {
        style = style.bg(getColor(rocStyle.bg));
    }

    if rocStyle.fg.discriminant() != crate::glue::discriminant_Color::Default {
        style = style.fg(getColor(rocStyle.fg));
    }

    let mut modifiers = tui::style::Modifier::empty();
    for modifier in &rocStyle.modifiers {
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

fn getColor(color: crate::glue::Color) -> tui::style::Color {
    match color.discriminant() {
        crate::glue::discriminant_Color::Default => tui::style::Color::Reset,
        crate::glue::discriminant_Color::Rgb => {
            let (r, g, b) = unsafe { color.as_Rgb() };
            tui::style::Color::Rgb(*r, *g, *b)
        }
        crate::glue::discriminant_Color::White => tui::style::Color::White,
        crate::glue::discriminant_Color::Black => tui::style::Color::Black,
        crate::glue::discriminant_Color::Red => tui::style::Color::Red,
        crate::glue::discriminant_Color::Green => tui::style::Color::Green,
        crate::glue::discriminant_Color::Blue => tui::style::Color::Blue,
    }
}

fn getAlignment(rocAlignment: crate::glue::Alignment) -> tui::layout::Alignment {
    match rocAlignment {
        crate::glue::Alignment::Left => tui::layout::Alignment::Left,
        crate::glue::Alignment::Center => tui::layout::Alignment::Center,
        crate::glue::Alignment::Right => tui::layout::Alignment::Right,
    }
}

fn getBorderType(rocBorderType: crate::glue::BorderType) -> tui::widgets::BorderType {
    match rocBorderType {
        crate::glue::BorderType::Plain => tui::widgets::BorderType::Plain,
        crate::glue::BorderType::Rounded => tui::widgets::BorderType::Rounded,
        crate::glue::BorderType::Double => tui::widgets::BorderType::Double,
        crate::glue::BorderType::Thick => tui::widgets::BorderType::Thick,
    }
}

fn getBorders(rocBorders: &roc_std::RocList<crate::glue::BorderModifier>) -> tui::widgets::Borders {
    let mut borders = tui::widgets::Borders::empty();
    for border in rocBorders {
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

fn getConstraints(
    rocConstraints: &roc_std::RocList<crate::glue::Constraint>,
) -> Vec<tui::layout::Constraint> {
    let mut constraints: Vec<tui::layout::Constraint> = Vec::with_capacity(rocConstraints.len());
    for constraint in rocConstraints {
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

fn getLayoutDirection(direction: crate::glue::LayoutDirection) -> tui::layout::Direction {
    match direction {
        crate::glue::LayoutDirection::Horizontal => tui::layout::Direction::Horizontal,
        crate::glue::LayoutDirection::Vertical => tui::layout::Direction::Vertical,
    }
}

fn getKeyCode(event: crossterm::event::KeyCode) -> crate::glue::KeyCode {
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
        crossterm::event::KeyCode::Media(mediaKey) => {
            crate::glue::KeyCode::Media(getKeyMedia(mediaKey))
        }
        crossterm::event::KeyCode::Menu => crate::glue::KeyCode::Menu,
        crossterm::event::KeyCode::Modifier(keyMod) => {
            crate::glue::KeyCode::Modifier(getKeyModifier(keyMod))
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

fn getKeyModifier(keyMod: crossterm::event::ModifierKeyCode) -> crate::glue::ModifierKeyCode {
    match keyMod {
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

fn getKeyMedia(mediaKey: crossterm::event::MediaKeyCode) -> crate::glue::MediaKeyCode {
    match mediaKey {
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

fn getScroll(scroll: u16) -> (u16, u16) {

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
