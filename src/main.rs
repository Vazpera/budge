mod app;
mod ui;
use app::Payment;
use ratatui::{init, restore};
use ratatui::widgets::{Scrollbar, ScrollbarState};
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = "sqlite:///home/vazpera/.local/share/test.db";
    let pool = create_database_pool(db_url).await?;
    println!(
        "{:?}",
        query_as!(Budget, "SELECT * FROM budget")
            .fetch_all(&pool)
            .await?
    );

    let mut terminal = init();

    let res = App::new(pool).run(&mut terminal).await;

    restore();
    res?;

    Ok(())
}
