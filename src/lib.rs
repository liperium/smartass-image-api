pub mod models;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::env;

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let result = PgConnection::establish(&database_url);

    match result {
        Ok(res) => res,
        Err(error) => {
            println!("{}", error);
            panic!("{}", error);
        }
    }
}
pub fn get_filename(task_unique_string: &str, user_id: &str, task_type: i32) -> String {
    return format!(
        "{}_{}_{}.jpg",
        user_id,
        task_type.to_string(),
        task_unique_string,
    );
}
