use std::{borrow::Cow, env};

use anyhow::{Context as _, Result};
use gitlab::{
    api::{
        self,
        common::NameOrId,
        endpoint_prelude::Method,
        projects::{
            jobs::JobScope,
            pipelines::{PipelineJobs, Pipelines},
        },
        Endpoint, Pagination, Query as _,
    },
    Gitlab,
};
use log::{debug, info, warn};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(transparent)]
struct JobId(u64);

#[derive(Deserialize)]
#[serde(transparent)]
struct PipelineId(u64);

#[derive(Deserialize)]
struct Job {
    id: JobId,
    name: String,
}

#[derive(Deserialize)]
struct Pipeline {
    id: PipelineId,
}

pub struct Api<'a> {
    gitlab: Gitlab,
    host: &'a str,
    project: &'a str,
    job: &'a str,
    artifact: &'a str,
}

impl<'a> Api<'a> {
    pub fn new(host: &'a str, project: &'a str, job: &'a str, artifact: &'a str) -> Result<Self> {
        let token = env::var("GITLAB_API_TOKEN")
            .context("missing Gitlab API access token -- set GITLAB_API_TOKEN")?;
        let gitlab = Gitlab::new(host, token).context("failed to create Gitlab API instance")?;
        Ok(Self {
            gitlab,
            host,
            project,
            job,
            artifact,
        })
    }

    pub fn get_artifact(&self, commit: &str) -> Result<String> {
        debug!(
            "Searching artifacts for commit {commit} in Gitlab project {} on {}",
            self.project, self.host
        );
        let pipelines = self.get_pipelines_for_commit(commit)?;
        let mut jobs = Vec::new();
        for pipeline in pipelines {
            jobs.extend(self.get_jobs_for_pipeline(pipeline)?);
        }
        if jobs.len() > 1 {
            warn!("Found multiple matching jobs for commit {commit} in Gitlab project {} on {}, using the first one", self.project, self.host);
        }
        let job = jobs.pop().with_context(|| {
            format!("no matching artifacts found for commit {commit} on Gitlab")
        })?;
        let artifact = self.get_artifact_for_job(job)?;
        info!(
            "Fetched metrics for commit {commit} from Gitlab project {} on {}",
            self.project, self.host
        );
        Ok(artifact)
    }

    fn get_pipelines_for_commit(&self, commit: &str) -> Result<Vec<PipelineId>> {
        debug!("Fetching pipelines for commit {commit}");
        let query = Pipelines::builder()
            .project(self.project)
            .sha(commit)
            .build()
            .context("failed to fetch pipelines from Gitlab")?;
        api::paged(query, Pagination::All)
            .iter(&self.gitlab)
            .map(|result| result.context("failed to parse pipeline returned by Gitlab"))
            .map(|result| result.map(|pipeline: Pipeline| pipeline.id))
            .collect()
    }

    fn get_jobs_for_pipeline(&self, pipeline: PipelineId) -> Result<Vec<JobId>> {
        debug!("Fetching jobs for pipeline {}", pipeline.0);
        let query = PipelineJobs::builder()
            .project(self.project)
            .pipeline(pipeline.0)
            .scope(JobScope::Success)
            .build()
            .context("failed to fetch jobs from Gitlab")?;
        api::paged(query, Pagination::All)
            .iter(&self.gitlab)
            .map(|result| result.context("failed to parse pipeline returned by Gitlab"))
            .filter(|result| {
                result
                    .as_ref()
                    .map(|job: &Job| job.name == self.job)
                    .unwrap_or(true)
            })
            .map(|result| result.map(|job| job.id))
            .collect()
    }

    fn get_artifact_for_job(&self, job: JobId) -> Result<String> {
        debug!("Fetching artifact {} for job {}", self.artifact, job.0);
        let query = JobArtifact {
            project: NameOrId::from(self.project),
            job,
            artifact: self.artifact,
        };
        let data = api::raw(query)
            .query(&self.gitlab)
            .context("failed to fetch artifact from Gitlab")?;
        String::from_utf8(data).context("failed to decode artifact returned by Gitlab as UTF-8")
    }
}

struct JobArtifact<'a> {
    project: NameOrId<'a>,
    job: JobId,
    artifact: &'a str,
}

impl Endpoint for JobArtifact<'_> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/jobs/{}/artifacts/{}",
            self.project, self.job.0, self.artifact
        )
        .into()
    }
}
