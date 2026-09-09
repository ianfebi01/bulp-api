use bulb_api::db;

// Embeds every migrations/V{n}__{name}.sql file into the binary at compile time
// and generates a `migrations::runner()` function in this module.
mod embedded {
    refinery::embed_migrations!("./migrations");
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let pool = db::create_pool();
    let mut conn = pool
        .get()
        .await
        .expect("failed to get a connection from the pool");

    // **conn derefs twice: deadpool::Object → ClientWrapper → tokio_postgres::Client
    let report = embedded::migrations::runner()
        .run_async(&mut **conn)
        .await
        .expect("migration failed");

    if report.applied_migrations().is_empty() {
        println!("no new migrations to apply");
    } else {
        for m in report.applied_migrations() {
            println!("applied: V{} {}", m.version(), m.name());
        }
    }
}
