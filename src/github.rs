use chrono::prelude::*;
use color_eyre::{eyre::bail, Result};
use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use reqwest::blocking::Client;

use self::pull_requests::{PullRequestsRepositoryPullRequests, PullRequestsRepositoryPullRequestsNodes};

#[allow(clippy::upper_case_acronyms)]
type URI = String;
type DateTime = chrono::DateTime<Utc>;

#[derive(GraphQLQuery)]
#[graphql(schema_path = "src/schema.graphql", query_path = "src/query.graphql", response_derives = "Debug")]
struct PullRequests;

pub fn fetch_pull_requests(client: &Client, owner: &str, name: &str, cursor: Option<String>) -> Result<PullRequestsRepositoryPullRequests> {
	let variables = pull_requests::Variables {
		owner: owner.to_owned(),
		name: name.to_owned(),
		cursor,
	};

	let response_body = post_graphql::<PullRequests, _>(client, "https://api.github.com/graphql", variables)?;
	if let Some(errors) = response_body.errors {
		for error in errors {
			bail!(error);
		}
	}
	let response_data: pull_requests::ResponseData = response_body
		.data
		.expect("missing response data");

	Ok(response_data
		.repository
		.expect("missing repository")
		.pull_requests)
}

pub fn paginate_pull_requests(owner: &str, name: &str, token: &str) -> Result<Vec<Option<PullRequestsRepositoryPullRequestsNodes>>> {
	let client = Client::builder()
		.user_agent("graphql-rust/0.10.0")
		.default_headers(std::iter::once((reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(&format!("Bearer {token}")).unwrap())).collect())
		.build()?;

	let mut cursor = None;
	let mut prs: Vec<Option<PullRequestsRepositoryPullRequestsNodes>> = vec![];

	loop {
		let data = fetch_pull_requests(&client, owner, name, cursor)?;

		prs.extend(
			data.nodes
				.expect("pull requests nodes is null"),
		);

		if !data.page_info.has_next_page {
			break;
		}
		cursor = data.page_info.end_cursor;
	}

	Ok(prs)
}
