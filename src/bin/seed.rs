use bulb_api::db;

// Sample dev data. Uses ON CONFLICT so re-running the seeder is safe.
const SEED_USERS: &[(&str, &str, Option<i32>)] = &[
    ("Alice", "alice@example.com", Some(28)),
    ("Bob", "bob@example.com", Some(35)),
    ("Carol", "carol@example.com", None),
];

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let pool = db::create_pool();
    let client = pool
        .get()
        .await
        .expect("failed to get a connection from the pool");

    for (name, email, age) in SEED_USERS {
        client
            .execute(
                "INSERT INTO users (name, email, age) VALUES ($1, $2, $3)
                 ON CONFLICT (email) DO NOTHING",
                &[name, email, age],
            )
            .await
            .expect("failed to seed user");
    }

    println!("seed complete ({} rows attempted)", SEED_USERS.len());
}
