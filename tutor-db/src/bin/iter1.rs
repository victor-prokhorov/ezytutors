use chrono::NaiveDateTime;
use dotenv::dotenv;
use sqlx::PgPool;
use std::{env, io};

#[derive(Debug, Clone)]
pub struct Course {
    pub course_id: i32,
    pub tutor_id: i32,
    pub course_name: String,
    pub posted_time: Option<NaiveDateTime>,
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    // TODO: not sure so far why we transform this into `Option`
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("set DATABASE_URL in .env");
    let db_pool = PgPool::connect(&database_url).await.unwrap();
    let course_rows = sqlx::query!(
        r#"SELECT course_id, tutor_id, course_name, posted_time FROM ezy_course_c4 WHERE course_id = $1"#,
        1
    )
        .fetch_all(&db_pool)
        .await
        .unwrap();
    let mut courses_list = Vec::new();
    for course_row in course_rows {
        courses_list.push(Course {
            course_id: course_row.course_id,
            tutor_id: course_row.tutor_id,
            course_name: course_row.course_name,
            posted_time: Some(chrono::NaiveDateTime::from(course_row.posted_time.unwrap())),
        });
    }
    dbg!(courses_list);
    Ok(())
}
