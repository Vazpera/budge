mod app;
use std::path::PathBuf;

use dirs::data_dir;
use ratatui::{init, restore};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{migrate, Pool, Sqlite, SqlitePool};
use sqlx::{query, query_as};


use crate::app::App;
pub type DbPool = Pool<Sqlite>;

pub async fn create_database_pool(options: &str) -> Result<DbPool, Box<dyn std::error::Error>> {
    let db_opts: SqliteConnectOptions = options.parse()?;

    let pool = SqlitePool::connect_with(db_opts.create_if_missing(true)).await?;

    migrate!().run(&pool).await?;

    Ok(pool)
}
use clap::{Parser, Subcommand};

#[derive(Subcommand, Clone, Debug)]
enum Mode {
    /// Add a new budget
    Create {
        amount: f64,
        month: String
    },
    /// Remove a budget, supplying the ID
    Remove {
        id: i64,
    },
    /// List all budgets
    List,
    /// Load a budget
    Load {
        budget_id: i64,
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
    let mut db_path:PathBuf = data_dir().unwrap();
    db_path.push("budge.db");
    let db_url = format!("{}", db_path.to_string_lossy());

    if !db_path.exists() {
        std::fs::File::create(db_path.clone())?;
    }

    let pool = create_database_pool(&db_url).await?;

    match args.mode {
        Mode::Load { budget_id } => {
            let mut terminal = init();

            let res = App::new(pool, budget_id).run(&mut terminal).await;

            restore();
            res?;
        }
        Mode::Create { amount, month } => {
            query!("INSERT INTO budget (amount, month) VALUES (?, ?)", amount, month).execute(&pool).await?;
            println!("Budget created successfully")
        }
        Mode::List => {
            println!("Hosted at: {}", db_path.to_string_lossy());
            let budgets = query_as!(crate::app::Budget, "SELECT * FROM budget").fetch_all(&pool).await?;
            for budget in budgets {
                println!("{}: {} - {}", budget.id, budget.amount, budget.month);
            }
        }
        Mode::Remove { id } => {
            query!("DELETE FROM budget WHERE id = ?", id).execute(&pool).await?;
            println!("Removed budget with id {id} successfully")
        }
    }

    Ok(())
}
