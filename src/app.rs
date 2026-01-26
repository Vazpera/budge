use itertools::multiunzip;
use ratatui::{
    Terminal, crossterm::event::{self, Event, KeyCode}, layout::Margin, macros::{horizontal, vertical}, prelude::Backend, widgets::{Row, ScrollbarOrientation, ScrollbarState, Table, TableState}
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Gauge, Paragraph, Scrollbar},
    Frame,
};

use sqlx::{query, query_as, Pool, Sqlite};
use tui_input::{backend::crossterm::EventHandler, Input};

#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct Payment {
    pub id: i64,
    pub amount: f64,
    pub budget_id: i64,
    pub kind: String,
    pub day_of: String,
}
#[derive(Debug, Clone, Default)]
pub struct Budget {
    pub id: i64,
    pub amount: f64,
    pub month: String,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum InputMode {
    Normal,
    Editing,
    Deleting,
    NewBudget,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputLocation {
    Type,
    Amount,
    Budget,
    Month,
}

pub struct App {
    pub pool: Pool<Sqlite>,
    pub scroll: usize,
    pub scroll_state: ScrollbarState,
    pub current_budget_id: i64,
    pub payments: Vec<Payment>,
    pub budget: Option<Budget>,
    pub payment_input: (Input, Input),
    pub deletion_id: Input,
    pub mode: InputMode,
    pub location: InputLocation,
    pub new_budget: (Input, Input),
}

impl App {
    pub fn new(pool: Pool<Sqlite>, id: i64) -> App {
        App {
            pool,
            scroll: usize::default(),
            scroll_state: ScrollbarState::default(),
            current_budget_id: id,
            payments: Vec::new(),
            budget: None,
            payment_input: (Input::default(), Input::default()),
            deletion_id: Input::default(),
            mode: InputMode::Normal,
            location: InputLocation::Type,
            new_budget: (Input::default(), Input::default()),
        }
    }

    pub async fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match query_as!(
            Budget,
            "SELECT * FROM budget WHERE id = ?",
            self.current_budget_id
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(budget) => self.budget = Some(budget),
            Err(_) => self.budget = None,
        }
        match query_as!(
            Payment,
            "SELECT * FROM payments WHERE budget_id = ? ORDER BY day_of DESC",
            self.current_budget_id
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(payments) => self.payments = payments,
            Err(_) => {}
        }

        Ok(())
    }
    pub async fn add_payment(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let ty = self.payment_input.0.value();
        let amount = self.payment_input.1.value().parse::<f64>()?;
        query!(
            r#"INSERT INTO payments (amount, budget_id, kind) VALUES (?, ?, ?)"#,
            amount,
            self.current_budget_id,
            ty
        )
        .execute(&self.pool)
        .await?;

        self.payment_input = (Input::default(), Input::default());

        Ok(())
    }
    pub async fn add_budget(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let amount = self.new_budget.0.value().parse::<f64>()?;
        let month = self.new_budget.1.value();

        query!(
            "INSERT INTO budget (amount, month) VALUES (?,?)",
            amount,
            month
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    pub async fn delete(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let id = self.deletion_id.value().parse::<i64>()?;

        query!("DELETE FROM payments WHERE id = ?", id)
            .execute(&self.pool)
            .await?;

        self.deletion_id = Input::default();
        Ok(())
    }
    pub async fn run<B: Backend>(
        mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.load().await {
            Ok(_) => {}
            Err(_) => {
                query!(
                    "INSERT INTO budget (amount, month) VALUES (?, ?)",
                    1000,
                    202501
                )
                .execute(&self.pool)
                .await?;
            }
        }
        loop {
            terminal.draw(|f| self.draw(f)).unwrap();
            let evt = event::read()?;
            if let Event::Key(key) = evt {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }
                match self.mode {
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            self.add_payment().await?;
                            self.mode = InputMode::Normal;
                            self.load().await?
                        }
                        KeyCode::Esc => self.mode = InputMode::Normal,
                        KeyCode::Tab => {
                            self.location = match self.location {
                                InputLocation::Type => InputLocation::Amount,
                                InputLocation::Amount => InputLocation::Type,
                                _ => unreachable!(),
                            }
                        }
                        _ => match self.location {
                            InputLocation::Type => {
                                self.payment_input.0.handle_event(&evt);
                            }
                            InputLocation::Amount => {
                                self.payment_input.1.handle_event(&evt);
                            }
                            _ => unreachable!(),
                        },
                    },

                    InputMode::NewBudget => match key.code {
                        KeyCode::Esc => {
                            self.new_budget = (Input::default(), Input::default());
                            self.mode = InputMode::Normal;
                        }
                        KeyCode::Enter => {
                            self.add_budget().await?;
                            self.load().await?;
                            self.mode = InputMode::Normal
                        }
                        KeyCode::Tab => {
                            self.location = match self.location {
                                InputLocation::Budget => InputLocation::Month,
                                InputLocation::Month => InputLocation::Budget,
                                _ => unreachable!(),
                            };
                        }
                        _ => match self.location {
                            InputLocation::Budget => {
                                self.new_budget.0.handle_event(&evt);
                            }
                            InputLocation::Month => {
                                self.new_budget.1.handle_event(&evt);
                            }
                            _ => unreachable!(),
                        },
                    },
                    InputMode::Deleting => match key.code {
                        KeyCode::Esc => {
                            self.deletion_id = Input::default();
                            self.mode = InputMode::Normal;
                        }
                        KeyCode::Enter => {
                            self.delete().await?;
                            self.load().await?;
                            self.mode = InputMode::Normal
                        }
                        _ => {
                            self.deletion_id.handle_event(&evt);
                        }
                    },

                    InputMode::Normal => match key.code {
                        KeyCode::Delete => self.mode = InputMode::Deleting,
                        KeyCode::Char('b') => {
                            self.mode = InputMode::NewBudget;
                            self.location = InputLocation::Budget
                        }
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('j') => {
                            self.scroll = self.scroll.saturating_add(1).clamp(0, self.payments.len() - 1);
                            self.scroll_state = self.scroll_state.position(self.scroll)
                        }
                        KeyCode::Char('k') => {
                            self.scroll = self.scroll.saturating_sub(1).clamp(0, self.payments.len() - 1);
                            self.scroll_state = self.scroll_state.position(self.scroll)
                        }
                        KeyCode::Char('a') => {
                            self.mode = InputMode::Editing;
                            self.location = InputLocation::Type
                        }
                        _ => {}
                    },
                }
            }
        }
    }
    pub fn render_add_payment_textbox(&self, frame: &mut Frame, area: Rect) {
        let input_scroll = self
            .payment_input
            .0
            .visual_scroll((area.width.max(3) - 3) as usize);
        let [ty, amount] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(3), Constraint::Fill(1)])
            .split(area)[..]
        else {
            panic!()
        };

        let in_type = Paragraph::new(self.payment_input.0.value()).block(
            Block::bordered()
                .title(" kind ")
                .title_style(match (self.location, self.mode) {
                    (InputLocation::Type, InputMode::Editing) => Style::default().yellow(),
                    (_, _) => Style::default().white(),
                }).border_style(Style::default().red())
        );
        let in_amount = Paragraph::new(self.payment_input.1.value()).block(
            Block::bordered()
                .title(" amount ")
                .title_style(match (self.location, self.mode) {
                    (InputLocation::Amount, InputMode::Editing) => Style::default().yellow(),
                    (_, _) => Style::default().white(),
                }).border_style(Style::default().red())
        );
        if self.mode == InputMode::Editing {
            match self.location {
                InputLocation::Type => {
                    let x =
                        self.payment_input.0.visual_cursor().max(input_scroll) - input_scroll + 1;
                    frame.set_cursor_position((ty.x + x as u16, ty.y + 1))
                }
                InputLocation::Amount => {
                    let x =
                        self.payment_input.1.visual_cursor().max(input_scroll) - input_scroll + 1;
                    frame.set_cursor_position((amount.x + x as u16, amount.y + 1))
                }
                _ => unreachable!(),
            }
        }
        frame.render_widget(in_type, ty);
        frame.render_widget(in_amount, amount);
    }
    pub fn render_deletion(&self, frame: &mut Frame, area: Rect) {
        let in_del =
            Paragraph::new(self.deletion_id.value()).block(Block::bordered().title(" deleting id: "));

        let input_scroll = self
            .deletion_id
            .visual_scroll((area.width.max(3) - 3) as usize);

        let x = self.deletion_id.visual_cursor().max(input_scroll) - input_scroll + 1;
        frame.render_widget(in_del, area);
        frame.set_cursor_position((area.x + x as u16, area.y + 1));
    }
    pub fn render_adding_budget(&self, frame: &mut Frame, area: Rect) {
        let in_amount = Paragraph::new(self.new_budget.0.value())
            .block(Block::bordered().title(" budget amount ".white()).border_style(Style::default().red()));
        let in_month = Paragraph::new(self.new_budget.1.value())
            .block(Block::bordered().title(" budget month ".white()).border_style(Style::default().red()));

        let input_scroll = self
            .new_budget
            .0
            .visual_scroll((area.width.max(3) - 3) as usize);
        let [amount, month] = horizontal![*=2, *=1].split(area)[..] else {
            unreachable!()
        };

        if self.mode == InputMode::NewBudget {
            match self.location {
                InputLocation::Budget => {
                    let x = self.new_budget.0.visual_cursor().max(input_scroll) - input_scroll + 1;
                    frame.set_cursor_position((amount.x + x as u16, amount.y + 1))
                }
                InputLocation::Month => {
                    let x = self.new_budget.1.visual_cursor().max(input_scroll) - input_scroll + 1;
                    frame.set_cursor_position((month.x + x as u16, month.y + 1))
                }
                _ => unreachable!(),
            }
        }
        frame.render_widget(in_amount, amount);
        frame.render_widget(in_month, month);
    }
    pub fn render_budget(&self, frame: &mut Frame, area: Rect) {
        let total_payout: f64 = self.payments.iter().map(|x| x.amount).sum();
        let budget = match self.budget.clone() {
            Some(b) => b,
            None => Budget::default(),
        };
        let ratio = match self.budget.clone() {
            Some(b) => total_payout.abs() / b.amount,
            None => 1.0,
        };
        let budget_visualizer = Gauge::default()
            .block(
                Block::bordered()
                    .title(" budget ".fg(Color::White))
                    .border_style(Style::default().fg(Color::Red)),
            )
            .ratio(ratio.abs().clamp(0.0, 1.0))
            .gauge_style(match total_payout.signum() {
                0.0 => Style::default(),
                1.0 => {
                    if total_payout > budget.amount {
                        Style::default().red()
                    } else {
                        Style::default().yellow()
                    }
                }
                -1.0 => Style::default().green(),
                _ => unreachable!(),
            })
            .label(match self.budget.clone() {
                Some(_b) => format!("{:.2}/{} $", total_payout, budget.amount.abs()).black(),
                None => "No budget loaded!".black(),
            });
        frame.render_widget(budget_visualizer, area);
    }
    pub fn _render_main(&self, frame: &mut Frame, area: Rect) {
        let main_info = Block::default()
            .title(" info ".fg(Color::White))
            .title_alignment(ratatui::layout::HorizontalAlignment::Left)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));
        frame.render_widget(main_info, area);
    }
    pub fn render_payments(&self, frame: &mut Frame, area: Rect) {
        let (ids, kinds, amounts, days): (Vec<_>, Vec<_>, Vec<_>, Vec<_>) =
            multiunzip(self.payments.iter().map(|x| {
                (
                    x.id.to_string(),
                    x.kind.clone(),
                    x.amount.to_string(),
                    x.day_of.clone(),
                )
            }));

        let rows = self.payments.iter().enumerate().map(|(i, x)| {
            let s_1 = x.id.to_string();
            let s_2 = x.kind.clone();
            let s_3 = x.amount.to_string();
            let s_4 = x.day_of.clone();

            Row::new([s_1, s_2, s_3, s_4]).style(match i % 2 {
                0 => Style::default().on_black(),
                1 => Style::default(),
                _ => unreachable!(),
            })
        });
        let table = Table::new(
            rows.clone(),
            [
                Constraint::Min(
                    ids.iter().max().unwrap_or(&"".to_string()).len() as u16 + 1,
                ),
                Constraint::Min(
                    kinds.iter().max().unwrap_or(&"".to_string()).len() as u16 + 1,
                ),
                Constraint::Min(
                    amounts.iter().max().unwrap_or(&"".to_string()).len() as u16 + 1,
                ),
                Constraint::Min(
                    days.iter().max().unwrap_or(&"".to_string()).len() as u16 + 1,
                ),
            ]
        ).block(Block::bordered().title(" payments ".white()).border_style(Style::default().red()));

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .style(Style::default().fg(Color::Gray));
        let mut scr = self.scroll_state.content_length(rows.len());

        frame.render_stateful_widget(table, area, &mut TableState::default().with_offset(self.scroll));
        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut scr,
        );

    }
    pub fn draw(&mut self, frame: &mut Frame) {
        let _layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Min(10)])
            .split(frame.area());

        // self.render_main(frame, layout[0]);

        let right_bar = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Min(3),
                Constraint::Percentage(100),
            ])
            .split(frame.area());

        self.render_budget(frame, right_bar[0]);
        self.render_add_payment_textbox(frame, right_bar[1]);
        self.render_payments(frame, right_bar[2]);
        if self.mode == InputMode::Deleting {
            let center_of_right_bar = centered_rect(50, 50, right_bar[2]);
            let mid = vertical![*=1, ==5, *= 1].split(center_of_right_bar);
            self.render_deletion(frame, centered_rect(50, 50, mid[1]));
        }
        if self.mode == InputMode::NewBudget {
            let center_of_right_bar = centered_rect(50, 50, frame.area());
            let mid = vertical![*=1, ==5, *= 1].split(center_of_right_bar);
            self.render_adding_budget(frame, centered_rect(50, 50, mid[1]));
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
