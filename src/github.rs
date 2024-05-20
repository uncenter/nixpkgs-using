use chrono::prelude::*;
use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use reqwest::{blocking::Client, Error};

use self::pull_requests::{PullRequestsRepositoryPullRequests, PullRequestsRepositoryPullRequestsNodes};

#[allow(clippy::upper_case_acronyms)]
type URI = String;
type DateTime = chrono::DateTime<Utc>;

#[derive(GraphQLQuery)]
#[graphql(schema_path = "src/schema.graphql", query_path = "src/query.graphql", response_derives = "Debug")]
struct PullRequests;

pub fn fetch_pull_requests(client: Client, owner: String, name: String, cursor: std::option::Option<std::string::String>) -> PullRequestsRepositoryPullRequests {
	let variables = pull_requests::Variables { owner, name, cursor };

	let response_body = post_graphql::<PullRequests, _>(&client, "https://api.github.com/graphql", variables).unwrap();

	let response_data: pull_requests::ResponseData = response_body
		.data
		.expect("missing response data");

	return response_data
		.repository
		.expect("missing repository")
		.pull_requests;
}

pub fn paginate_pull_requests(owner: String, name: String, token: String) -> Result<Vec<Option<PullRequestsRepositoryPullRequestsNodes>>, Error> {
	let client = Client::builder()
		.user_agent("graphql-rust/0.10.0")
		.default_headers(std::iter::once((reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap())).collect())
		.build()?;

	let mut cursor = None;
	let mut prs: Vec<Option<PullRequestsRepositoryPullRequestsNodes>> = vec![];

	loop {
		let data = fetch_pull_requests(client.clone(), owner.clone(), name.clone(), cursor);

		prs.extend(
			data.nodes
				.expect("pull requests nodes is null"),
		);

		if !data.page_info.has_next_page {
			break;
		}
		cursor = data.page_info.end_cursor;
	}

	return Ok(prs);
}
