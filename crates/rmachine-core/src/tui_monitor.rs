use std::{collections::HashSet, time::Duration};

use crate::machine::Machine;
use anyhow::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Running,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Turbo,
}

pub struct TuiMonitor<'a> {
    mode: Mode,
    state: State,
    machine: &'a mut Machine,
    changed_addresses: HashSet<u16>,
}

impl<'a> TuiMonitor<'a> {
    /// Create a new TUI monitor.
    pub fn new(machine: &'a mut Machine) -> Self {
        Self {
            machine,
            mode: Mode::Normal,
            state: State::Paused,
            changed_addresses: HashSet::new(),
        }
    }

    /// Run the TUI monitor.
    ///
    /// # Errors
    ///
    /// Returns an error if the TUI monitor fails to run.
    pub fn run(&mut self) -> Result<()> {
        ratatui::run(|terminal| app(terminal, self))?;
        Ok(())
    }

    fn find_changed_addresses(&mut self, previous_memory: &[u8]) {
        self.changed_addresses.clear();
        for (addr, &value) in self.machine.memory.iter().enumerate() {
            if previous_memory[addr] != value {
                self.changed_addresses
                    .insert(u16::try_from(addr).expect("overflow"));
            }
        }
    }

    fn step(&mut self) {
        let previous_memory = self.machine.memory.clone();
        self.machine.step();
        self.find_changed_addresses(&previous_memory);
    }

    fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            Mode::Normal => Mode::Turbo,
            Mode::Turbo => Mode::Normal,
        };
    }

    fn reset(&mut self) {
        self.machine.reset();
        self.state = State::Paused;
        self.changed_addresses.clear();
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        match (code, modifiers) {
            (KeyCode::Char('q'), _) => return false,
            (KeyCode::Char('n'), _) => self.step(),
            (KeyCode::Char('t'), _) => self.toggle_mode(),
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => self.reset(),
            (KeyCode::Char('r'), KeyModifiers::NONE) => self.state = State::Running,
            _ => {}
        }
        true
    }

    fn tick(&mut self) {
        if self.machine.exception.is_some() {
            self.state = State::Paused;
            return;
        }
        if self.state == State::Running {
            match self.mode {
                Mode::Normal => self.step(),
                Mode::Turbo => self.machine.run(),
            }
        }
    }
}

fn app(terminal: &mut DefaultTerminal, monitor: &mut TuiMonitor) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| render(frame, monitor))?;
        if event::poll(Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && !monitor.handle_key(key.code, key.modifiers)
            {
                break;
            }
        } else {
            monitor.tick();
        }
    }
    Ok(())
}

fn render(frame: &mut Frame, monitor: &mut TuiMonitor) {
    let app_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let bottom_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(28), Constraint::Percentage(72)])
        .split(app_layout[1]);

    render_registers(monitor, frame, app_layout[0]);
    render_disassembly(monitor, frame, bottom_layout[0]);
    render_memory(monitor, frame, bottom_layout[1]);
    render_menu(frame, app_layout[2]);
}

fn render_menu(frame: &mut Frame, area: Rect) {
    let spans = vec![
        "<r> - run / ".fg(Color::Gray),
        "<n> - step / ".fg(Color::Gray),
        "<t> - turbo mode / ".fg(Color::Gray),
        "<ctrl+r> - reset / ".fg(Color::Gray),
        "<q> - quit ".fg(Color::Gray),
    ];
    let p = Paragraph::new(Text::from(vec![Line::from(spans).centered()]));
    frame.render_widget(p, area);
}

fn render_registers(monitor: &mut TuiMonitor, frame: &mut Frame, area: Rect) {
    let block = Block::new().borders(Borders::ALL).title("Status");
    frame.render_widget(block.clone(), area);

    let inner = block.inner(area);
    let num_registers = monitor.machine.register_list.len();
    let total = num_registers.checked_add(3).expect("overflow");

    let cells = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Ratio(
                1,
                u32::try_from(total).expect("too large")
            );
            total
        ])
        .split(inner);

    let pc = Line::from(vec![
        Span::styled("PC: ", Style::default().fg(Color::Blue).bold()),
        Span::styled(
            format!("{:04X} ", monitor.machine.pc()),
            Style::default().fg(Color::Yellow),
        ),
    ]);
    frame.render_widget(Text::from(vec![pc]), cells[0]);

    let cycles = Line::from(vec![
        Span::styled("Cycles: ", Style::default().fg(Color::Blue).bold()),
        Span::styled(
            monitor.machine.cycles.to_string(),
            Style::default().fg(Color::Yellow),
        ),
    ]);
    frame.render_widget(Text::from(vec![cycles]), cells[1]);

    let speed = Line::from(vec![
        Span::styled("Speed: ", Style::default().fg(Color::Blue).bold()),
        Span::styled(
            format!("{} mhz", monitor.machine.speed_mhz()),
            Style::default().fg(Color::Yellow),
        ),
    ]);
    frame.render_widget(Text::from(vec![speed]), cells[2]);

    for (i, reg) in monitor.machine.register_list.iter().enumerate() {
        let mut spans = vec![];
        let value = monitor.machine.reg(reg);
        spans.push(Span::styled(
            format!("{reg}: "),
            Style::default().fg(Color::Blue).bold(),
        ));
        spans.push(Span::styled(
            format!("{value:02X} "),
            Style::default().fg(Color::Yellow),
        ));
        let bits = (0..8)
            .rev()
            .map(|n| {
                let color = if (value >> n) & 1 == 1 {
                    Color::Rgb(255, 80, 80)
                } else {
                    Color::Rgb(100, 0, 0)
                };
                Span::styled("●", Style::default().fg(color))
            })
            .collect::<Vec<_>>();
        spans.extend(bits);
        let line = Line::from(spans);
        let cell_index = i.checked_add(3).expect("overflow");
        frame.render_widget(Text::from(vec![line]), cells[cell_index]);
    }
}

fn render_disassembly(monitor: &mut TuiMonitor, frame: &mut Frame, area: Rect) {
    let mut disassembled_program = vec![];

    let mut pc = 0u16;
    while let Some((instruction, bytes)) = monitor.machine.disassemble_at(pc) {
        let mut spans = vec![];
        if pc == monitor.machine.pc() {
            spans.push("> ".into());
            spans.push(Span::styled(
                format!("${pc:04x}: "),
                Style::default().fg(Color::Gray).bg(Color::DarkGray),
            ));
            spans.push(instruction.fg(Color::Yellow).bg(Color::DarkGray));
        } else {
            spans.push(Span::styled(
                format!("  ${pc:04x}: "),
                Style::default().fg(Color::Gray),
            ));
            spans.push(instruction.fg(Color::Yellow));
        }
        disassembled_program.push(Line::from(spans));
        pc = pc.checked_add(bytes).expect("pc overflow");
    }

    let p = Paragraph::new(Text::from(disassembled_program));
    frame.render_widget(
        p.block(Block::new().borders(Borders::ALL).title("Disassembly")),
        area,
    );
}

fn render_memory(monitor: &mut TuiMonitor, frame: &mut Frame, area: Rect) {
    let mut lines = vec![];

    for row in 0..16_u16 {
        let mut hex_spans = vec![];
        let mut ascii_spans = vec![];
        let row_offset = row.checked_mul(16).expect("address out of range");

        for col in 0..16 {
            let addr = row_offset.checked_add(col).expect("address out of range");
            let value = monitor.machine.get8(addr);
            let changed = monitor.changed_addresses.contains(&addr);
            let color = if changed {
                Color::Yellow
            } else if value != 0 {
                Color::Gray
            } else {
                Color::DarkGray
            };
            hex_spans.push(Span::styled(
                format!("{value:02x} "),
                Style::default().fg(color),
            ));

            let display_char = if value.is_ascii_graphic() || value == b' ' {
                char::from(value).to_string()
            } else {
                ".".to_string()
            };
            ascii_spans.push(Span::styled(display_char, Style::default().fg(color)));
        }

        let mut spans = vec![format!("{row_offset:#06x}: ").fg(Color::Gray)];
        spans.extend(hex_spans);
        spans.push("|".fg(Color::DarkGray));
        spans.extend(ascii_spans);
        spans.push("|".fg(Color::DarkGray));
        lines.push(Line::from(spans));
    }

    let page = Paragraph::new(Text::from(lines));
    frame.render_widget(
        page.block(Block::new().borders(Borders::ALL).title("Memory")),
        area,
    );
}
