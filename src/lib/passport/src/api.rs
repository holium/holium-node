use warp::Filter;

use bedrock_db::api::{db_get_contact_passports, db_insert_passport};
use bedrock_db::models::Passport;

pub fn get_contact_passports(
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["POST"]);

    let get_passports = warp::path!("hol" / "passports")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_get_contact_passports);

    get_passports.with(cors)
}

async fn handle_get_contact_passports(
    phone_numbers: Vec<String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&db_get_contact_passports(phone_numbers)))
}

pub fn insert_passport(
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["POST"]);

    let insert_passport = warp::path!("hol" / "passport")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_insert_passport);

    insert_passport.with(cors)
}

pub async fn handle_insert_passport(
    passport: Passport,
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("inserting passport: {:?}", passport);
    let result = &db_insert_passport(passport);
    // TODO: normalize phone number format if necessary
    Ok(warp::reply::json(result))
}
