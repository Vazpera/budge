use std::io;

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Margin},
    prelude::Backend,
    text::ToLine,
    widgets::{Padding, ScrollbarOrientation, ScrollbarState},
    Terminal,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Scrollbar, Wrap},
    Frame,
};
use sqlx::{query, query_as, Pool, Sqlite};

use crate::ui;

#[derive(Debug)]
pub enum Screen {
    PaymentLogger,
}
#[derive(Default, Debug)]
pub struct Payment {
    pub id: i64,
    pub amount: i64,
    pub budget_id: i64,
    pub kind: String,
    pub day_of: String,
}
#[derive(Debug, Clone, Default)]
pub struct Budget {
    pub id: i64,
    pub amount: i64,
    pub month: String,
}

pub struct PaymentBuilder {
    pub id: String,
    pub amount: String,
    pub budget_id: String,
    pub kind: String,
    pub day_of: String,
}

pub struct App {
    pub pool: Pool<Sqlite>,
    pub scroll: usize,
    pub scroll_state: ScrollbarState,
    pub current_budget_id: i64,
    pub payments: Vec<Payment>,
    pub budget: Option<Budget>,
    pub adding_payment: bool,
}

impl App {
    pub fn new(pool: Pool<Sqlite>) -> App {
        App {
            pool,
            scroll: usize::default(),
            scroll_state: ScrollbarState::default(),
            current_budget_id: 1,
            payments: Vec::new(),
            budget: None,
            adding_payment: false,
        }
    }
    pub async fn run<B: Backend>(
        mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let budget = query_as!(
            Budget,
            "SELECT * FROM budget WHERE id = ?",
            self.current_budget_id
        )
        .fetch_one(&self.pool)
        .await?;
        let payments = query_as!(
            Payment,
            "SELECT * FROM payments WHERE budget_id = ?",
            self.current_budget_id
        )
        .fetch_all(&self.pool)
        .await?;

        self.budget = Some(budget);
        self.payments = payments;
        loop {
            terminal.draw(|f| self.draw(f)).unwrap();
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }
                if !self.adding_payment {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('j') => {
                            self.scroll = self.scroll.saturating_add(1);
                            self.scroll_state = self.scroll_state.position(self.scroll)
                        }
                        KeyCode::Char('k') => {
                            self.scroll = self.scroll.saturating_sub(1);
                            self.scroll_state = self.scroll_state.position(self.scroll)
                        }
                        KeyCode::Enter => {
                            self.adding_payment = true;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    pub fn draw(&mut self, frame: &mut Frame) {
        let total_payout: i64 = self.payments.iter().map(|x| x.amount).sum();
        let budget = match self.budget.clone() {
            Some(b) => b,
            None => Budget::default(),
        };
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Min(10)])
            .split(frame.area());

        let main_info = Block::default()
            .title(" info ".fg(Color::White))
            .title_alignment(ratatui::layout::HorizontalAlignment::Left)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));
        frame.render_widget(main_info, layout[0]);

        let right_bar = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Percentage(100)])
            .split(layout[1]);

        let budget_visualizer = Gauge::default()
            .block(
                Block::bordered()
                    .title(" budget ".fg(Color::White))
                    .border_style(Style::default().fg(Color::Red)),
            )
            .ratio(total_payout as f64 / budget.amount as f64)
            .gauge_style(match total_payout.signum() {
                0 => Style::default(),
                1 => {
                    if total_payout > budget.amount {
                        Style::default().red()
                    } else {
                        Style::default().yellow()
                    }
                }
                -1 => Style::default().green(),
                _ => unreachable!(),
            })
            .label(
                format!(
                    "{}/{} $",
                    total_payout,
                    budget.amount.abs()
                )
                .black(),
            );

        let (type_lines, payment_lines): (Vec<Line>, Vec<Line>) = self
            .payments
            .iter()
            .map(|x| (x.kind.to_line(), x.amount.to_line()))
            .collect();

        let types = Paragraph::new(type_lines.clone())
            .block(
                Block::bordered()
                    .title(" payments ".fg(Color::White))
                    .border_style(Style::default().fg(Color::Red))
                    .padding(Padding::new(1, 1, 0, 0)),
            )
            .scroll((self.scroll as u16, 0));
        let payment_amounts = Paragraph::new(payment_lines.clone())
            .block(
                Block::bordered()
                    .title(" payments ".fg(Color::White))
                    .border_style(Style::default().fg(Color::Red))
                    .padding(Padding::new(1, 1, 0, 0)),
            )
            .alignment(Alignment::Right)
            .scroll((self.scroll as u16, 0));

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .style(Style::default().fg(Color::Gray));
        let mut scr = self.scroll_state.content_length(type_lines.len());

        frame.render_widget(budget_visualizer, right_bar[0]);

        frame.render_widget(payment_amounts, right_bar[1]);
        frame.render_widget(types, right_bar[1]);

        frame.render_stateful_widget(
            scrollbar,
            right_bar[1].inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut scr,
        );

        if self.adding_payment {

        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
