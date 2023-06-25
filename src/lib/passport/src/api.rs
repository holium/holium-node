use tokio::join;
use warp::Filter;

use bedrock_db::api::{db_get_contact_passports, db_insert_passport};
use bedrock_db::models::Passport;
use chrono::NaiveDateTime;
use reqwest::Client;
use serde::Deserialize;

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

#[derive(Deserialize)]
struct GetContactPassports {
    phone_numbers: Option<Vec<String>>,
    twitter: Option<String>,
    since: Option<i64>,
}

async fn handle_get_contact_passports(
    GetContactPassports {
        phone_numbers,
        twitter,
        since,
    }: GetContactPassports,
) -> Result<impl warp::Reply, warp::Rejection> {
    // If since is unset, defaults to T=0
    let since_time: NaiveDateTime =
        NaiveDateTime::from_timestamp_millis(since.unwrap_or_else(|| 0)).unwrap();

    // TODO: normalize phone number format if necessary
    let phone_numbers = phone_numbers.unwrap_or_else(|| vec![]);

    let twitter_handles = vec![];
    if twitter.is_some() {
        let endpoint = format!("https://api.twitter.com/2/users/{}", twitter.unwrap());
        let client = Client::new();
        // TODO: hit twitter API with twitter to get all followers and following, if given

        // let (followers, following) = join!(
        //     client.get(concat!(endpoint, "/followers")).send(),
        //     client.get(concat!(endpoint, "/following")).send()
        // );
    }

    Ok(warp::reply::json(&db_get_contact_passports(
        phone_numbers,
        twitter_handles,
        since_time,
    )))
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
