use dotenvy::dotenv;
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: admin-cli <username>");
        eprintln!("Example: cargo run --bin admin-cli -- john_doe");
        std::process::exit(1);
    }
    
    let target_username = &args[1];

    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("Error: DATABASE_URL must be set in .env or environment variables");
        std::process::exit(1);
    });

    println!("Connecting to the database...");
    let db = match Database::connect(&db_url).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to connect to database: {:?}", e);
            std::process::exit(1);
        }
    };

    println!("Promoting user '{}' to admin...", target_username);
    
    let sql = "UPDATE users SET role = 'admin' WHERE username = $1";
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        vec![target_username.into()]
    );

    match db.execute(stmt).await {
        Ok(exec_res) => {
            if exec_res.rows_affected() > 0 {
                println!("✅ Successfully promoted user '{}' to admin.", target_username);
            } else {
                println!("❌ User '{}' not found.", target_username);
            }
        },
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            std::process::exit(1);
        }
    }
}
