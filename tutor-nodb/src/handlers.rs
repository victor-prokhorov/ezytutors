use super::models::Course;
use super::state::AppState;
use actix_web::{web, HttpResponse};
use chrono::Utc;

pub async fn health_check_handler(app_state: web::Data<AppState>) -> HttpResponse {
    println!("health-check handler");
    let health_check_response = &app_state.health_check_response;
    let mut visit_count = app_state.visit_count.lock().unwrap();
    let response = format!("{} {} times", health_check_response, visit_count);
    *visit_count += 1;
    HttpResponse::Ok().json(&response)
}

pub async fn new_course(
    new_course: web::Json<Course>,
    app_state: web::Data<AppState>,
) -> HttpResponse {
    println!("received new course");
    // TODO: benchmark where it is faster to `lock` `courses` twice
    // or holding the first `lock` for longer period
    let mut course_count_for_user: i32 = app_state
        .courses
        .lock()
        .expect("failed to acquire a mutex")
        .clone()
        .into_iter()
        .filter(|course| course.tutor_id == new_course.tutor_id)
        .count()
        .try_into()
        .expect("wasn't able to fit `uszie` into `i32`");
    course_count_for_user += 1;
    let new_course = Course {
        tutor_id: new_course.tutor_id,
        course_id: Some(course_count_for_user),
        course_name: new_course.course_name.clone(),
        posted_time: Some(Utc::now().naive_utc()),
    };
    app_state
        .courses
        .lock()
        .expect("failed to acquire a mutex")
        .push(new_course);
    HttpResponse::Ok().json(format!(
        "course added, that makes a total of {course_count_for_user} course"
    ))
}

// TODO: try by reference
pub async fn get_courses_for_tutor(
    app_state: web::Data<AppState>,
    params: web::Path<i32>,
) -> HttpResponse {
    let tutor_id: i32 = *params;
    let filtered_courses: Vec<Course> = app_state
        .courses
        .lock()
        .expect("failed to acquire a mutex")
        .clone()
        .into_iter()
        .filter(|course| course.tutor_id == tutor_id)
        .collect();
    if filtered_courses.len() > 0 {
        HttpResponse::Ok().json(filtered_courses)
    } else {
        HttpResponse::Ok().json("no courses found for tutor")
    }
}

pub async fn get_course_detail(
    app_state: web::Data<AppState>,
    params: web::Path<(i32, i32)>,
) -> HttpResponse {
    let (tutor_id, course_id) = params.into_inner();
    let course = app_state
        .courses
        .lock()
        .unwrap()
        .iter()
        .find(|course| course.course_id == Some(course_id) && course.tutor_id == tutor_id)
        // i guess it's better to `clone` only found course
        .cloned();
    if let Some(course) = course {
        HttpResponse::Ok().json(course)
    } else {
        HttpResponse::Ok().json("unable to find specific course for this tutor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{body::to_bytes, http::StatusCode, web::Bytes};
    use std::sync::Mutex;
    trait BodyTest {
        fn as_str(&self) -> &str;
    }

    impl BodyTest for Bytes {
        fn as_str(&self) -> &str {
            std::str::from_utf8(self).unwrap()
        }
    }
    #[actix_rt::test]
    async fn post_course_test() {
        let course = web::Json(Course {
            tutor_id: 1,
            course_name: "course".to_string(),
            course_id: None,
            posted_time: None,
        });
        let app_state = web::Data::new(AppState {
            health_check_response: "".to_string(),
            visit_count: Mutex::new(0),
            courses: Mutex::new(Vec::new()),
        });

        let resp = new_course(course, app_state).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn get_courses() {
        // https://github.com/actix/actix-web/discussions/2640
        // https://stackoverflow.com/questions/63910673/how-to-get-the-body-of-a-response-in-actix-web-unit-test
        // https://github.com/actix/examples/blob/master/forms/form/src/main.rs
        let course = Course {
            tutor_id: 1,
            course_name: "1234".to_string(),
            course_id: None,
            posted_time: None,
        };
        let app_state = web::Data::new(AppState {
            health_check_response: "".to_string(),
            visit_count: Mutex::new(0),
            courses: Mutex::new(vec![course.clone()]),
        });
        let path = web::Path::from(1);

        let resp = get_courses_for_tutor(app_state, path).await;
        let body = to_bytes(resp.into_body()).await.unwrap();
        // OR
        let courses: Vec<Course> =
            serde_json::from_str(body.as_str()).expect("Failed to parse response");

        assert_eq!(
            body.as_str(),
            "[{\"tutor_id\":1,\"course_id\":null,\"course_name\":\"1234\",\"posted_time\":null}]"
        );
        // OR
        assert_eq!(courses, vec![course]);
    }

    #[actix_rt::test]
    async fn get_one_course_success() {
        let app_state: web::Data<AppState> = web::Data::new(AppState {
            health_check_response: "".to_string(),
            visit_count: Mutex::new(0),
            courses: Mutex::new(vec![]),
        });
        let params: web::Path<(i32, i32)> = web::Path::from((1, 1));

        let resp = get_course_detail(app_state, params).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }
}
