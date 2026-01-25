mod app;
mod ui;
use app::Payment;
use ratatui::widgets::{Scrollbar, ScrollbarState};
use ratatui::{init, restore};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{migrate, Pool, Sqlite, SqlitePool};
use sqlx::{query, query_as, sqlite::SqlitePoolOptions};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};
use std::{error::Error, io};

use crate::app::{App, Budget};
pub type DbPool = Pool<Sqlite>;

pub async fn create_database_pool(options: &str) -> Result<DbPool, Box<dyn std::error::Error>> {
    let db_opts: SqliteConnectOptions = options.parse()?;

    let pool = SqlitePool::connect_with(db_opts.create_if_missing(true)).await?;

    migrate!().run(&pool).await?;

    Ok(pool)
}
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Subcommand, Clone, Debug)]
enum Mode {
    Create {
        amount: f64,
        month: String
    },
    List,
    Load {
        #[arg(short, long)]
        budget_id: Option<i64>,
    },
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    mode: Mode,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let db_url = "sqlite:///home/vazpera/.local/share/test.db";
    let pool = create_database_pool(db_url).await?;

    match args.mode {
        Mode::Load { budget_id } => {
            let mut terminal = init();

            let res = App::new(pool, budget_id).run(&mut terminal).await;

            restore();
            res?;
        }
        Mode::Create { amount, month } => {
            let x = query!("INSERT INTO budget (amount, month) VALUES (?, ?)", amount, month).execute(&pool).await?;
            println!("{x:?}")
        }
        Mode::List => {
            let budgets = query_as!(crate::app::Budget, "SELECT * FROM budget").fetch_all(&pool).await?;
            for budget in budgets {
                println!("{:?}", budget);
            }
        }
    }

    Ok(())
}
