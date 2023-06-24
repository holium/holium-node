use warp::Filter;

pub fn passport_route(
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["POST"]);

    let get_passports = warp::path!("hol" / "passports")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_get_passports);

    get_passports.with(cors)
}

pub async fn handle_get_passports(
    phone_numbers: Vec<String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("phone_numbers: {:?}", phone_numbers);
    Ok(warp::reply::json(&"hello"))
}
