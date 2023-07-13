use warp::{http::StatusCode, reject, reply, Filter, Rejection, Reply};

#[derive(Debug)]
struct InvalidParameter;
#[derive(Debug)]
struct DbError;

impl reject::Reject for InvalidParameter {}
impl reject::Reject for DbError {}

// Custom rejection handler that maps rejections into responses.
async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    if err.is_not_found() {
        Ok(reply::with_status("NOT_FOUND", StatusCode::NOT_FOUND))
    } else if let Some(_) = err.find::<InvalidParameter>() {
        Ok(reply::with_status("BAD_REQUEST", StatusCode::BAD_REQUEST))
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        Ok(reply::with_status(
            "INTERNAL_SERVER_ERROR",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}

pub fn chat_router() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET"]); // , "POST", "DELETE"]);

    // /db/messages/start-ms/{}
    let chat_routes = warp::path!("hol" / "chat" / "messages" / "start-ms")
        .and(warp::path::param())
        .and_then(|param: String| async { handle_chat_messages(param).await })
        .recover(handle_rejection);

    chat_routes.with(cors)
}

pub async fn handle_chat_messages(param: String) -> Result<impl warp::Reply, warp::Rejection> {
    let timestamp = i64::from_str_radix(&param, 10);
    if timestamp.is_err() {
        println!(
            "chat: [handle_chat_messages] invalid start-ms parameter {}",
            param
        );
        return Err(reject::custom(InvalidParameter));
    }
    let data = {
        let data = super::data::query_messages(timestamp.unwrap()).await;
        if data.is_err() {
            println!("chat: [handle_chat_messages] query_messages failed");
            return Err(reject::custom(DbError));
        }
        data.unwrap()
    };
    Ok(warp::reply::json(&data))
}
