use super::models::Course;
use crate::errors::EzyTutorError;
use sqlx::postgres::PgPool;

pub async fn get_courses_for_tutor_db(
    pool: &PgPool,
    tutor_id: i32,
) -> Result<Vec<Course>, EzyTutorError> {
    let course_rows = sqlx::query!(
        "SELECT tutor_id, course_id, course_name, posted_time
        FROM ezy_course_c4
        WHERE tutor_id = $1",
        tutor_id
    )
    .fetch_all(pool)
    .await?;
    let courses: Vec<Course> = course_rows
        .iter()
        .map(|course_row| Course {
            course_id: course_row.course_id,
            tutor_id: course_row.tutor_id,
            course_name: course_row.course_name.clone(),
            posted_time: Some(chrono::NaiveDateTime::from(course_row.posted_time.unwrap())),
        })
        .collect();
    if courses.is_empty() {
        Err(EzyTutorError::NotFound(
            "courses not found for tutor".to_string(),
        ))
    } else {
        Ok(courses)
    }
}

pub async fn get_course_details_db(
    pool: &PgPool,
    tutor_id: i32,
    course_id: i32,
) -> Result<Course, EzyTutorError> {
    let course_row = sqlx::query!(
        "SELECT tutor_id, course_id, course_name, posted_time
        FROM ezy_course_c4
        WHERE tutor_id = $1 AND course_id = $2",
        tutor_id,
        course_id,
    )
    .fetch_one(pool)
    .await;
    let Ok(course_row) = course_row else {
        return Err(EzyTutorError::DBError(
            "course id was not found".to_string(),
        ));
    };
    Ok(Course {
        course_id: course_row.course_id,
        tutor_id: course_row.tutor_id,
        course_name: course_row.course_name.clone(),
        posted_time: Some(chrono::NaiveDateTime::from(course_row.posted_time.unwrap())),
    })
}

// NOTE: i guess it will be fixed in the upcoming chapters but so far it feels strange
// to insert `course_id` in the app code while we already set `SERIAL` in the db
pub async fn post_new_course_db(pool: &PgPool, course: Course) -> Result<Course, EzyTutorError> {
    let course_row = sqlx::query!(
        "INSERT INTO ezy_course_c4(tutor_id, course_id, course_name)
        VALUES($1, $2, $3)
        RETURNING tutor_id, course_id, course_name, posted_time",
        course.tutor_id,
        course.course_id,
        course.course_name,
    )
    .fetch_one(pool)
    .await?;
    // TODO: impl `From`, i guess at some point `query_as` will be introdued as well
    Ok(Course {
        course_id: course_row.course_id,
        tutor_id: course_row.tutor_id,
        course_name: course_row.course_name.clone(),
        posted_time: Some(chrono::NaiveDateTime::from(course_row.posted_time.unwrap())),
    })
}
