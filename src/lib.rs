use http::StatusCode;
use matchit::{Params, Router};

type Request = http::Request<()>;
type Response = http::Response<&'static str>;

#[cfg(test)]
mod tests {
	use futures::executor::block_on;
	use http::StatusCode;

	use crate::{Request, RouterStatic};

	#[test]
	fn test() {
		block_on(async {
			let router = RouterStatic::new();
			let req = Request::get("/").body(()).unwrap();
			assert_eq!(router.route(req).await.status(), StatusCode::OK);

			let req = Request::get("/abcd").body(()).unwrap();
			assert_eq!(router.route(req).await.status(), StatusCode::NOT_FOUND);

			let req = Request::get("/abcd").body(()).unwrap();
			assert_eq!(router.route(req).await.status(), StatusCode::NOT_FOUND);

			let req = Request::get("/api/v1/user").body(()).unwrap();
			assert_eq!(router.route(req).await.status(), StatusCode::NOT_FOUND);

			let req = Request::post("/api/v1/user").body(()).unwrap();
			assert_eq!(router.route(req).await.status(), StatusCode::OK);
		});
	}
}

#[inline(never)]
async fn not_found(req: Request) -> Response {
	let req = async move { req }.await;
	drop(req);
	http::Response::builder()
		.status(StatusCode::NOT_FOUND)
		.body("")
		.unwrap()
}

// #[get("/")]
#[inline(never)]
async fn route_1<'a, 'b>(req: &'a Request, _: Params<'a, 'b>) -> Response {
	let _ = async move { req }.await;
	http::Response::builder()
		.status(StatusCode::OK)
		.body("")
		.unwrap()
}

// #[post("/api/v1/user")]
#[inline(never)]
async fn route_2<'a, 'b>(req: &'a Request, _: Params<'a, 'b>) -> Response {
	let _ = async move { req }.await;
	http::Response::builder()
		.status(StatusCode::OK)
		.body("")
		.unwrap()
}


// --- generated code --- //

pub struct RouterStatic {
	get: Router<u8>,
	post: Router<u8>,
}

impl RouterStatic {
	pub fn new() -> Self {
		let mut get = Router::new();
		get.insert("/", 0).unwrap();
		let mut post = Router::new();
		post.insert("/api/v1/user", 0).unwrap();
		Self {
			get,
			post,
		}
	}

	pub async fn route(&self, req: Request) -> Response {
		match req.method().as_str() {
			"GET" => {
				self.route_get(req).await
			}
			"POST" => {
				self.route_post(req).await
			}
			// remaining method generate base on user requirement
			_ => {
				not_found(req).await
			}
		}
	}

	// this function will generate based on method required with above method routing
	#[inline]
	async fn route_get(&self, req: Request) -> Response {
		let path = req.uri().path();
		match self.get.at(path) {
			Ok(item) => {
				match *item.value {
					0 => route_1(&req, item.params).await,
					_ => not_found(req).await,
				}
			}
			Err(_) => {
				not_found(req).await
			}
		}
	}

	// this function will generate based on method required with above method routing
	#[inline]
	async fn route_post(&self, req: Request) -> Response {
		let path = req.uri().path();
		match self.post.at(path) {
			Ok(item) => {
				match *item.value {
					0 => route_2(&req, item.params).await,
					_ => not_found(req).await,
				}
			}
			Err(_) => {
				not_found(req).await
			}
		}
	}
}