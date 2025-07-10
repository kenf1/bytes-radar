pub mod archive;
pub mod azure_devops;
pub mod bitbucket;
pub mod codeberg;
pub mod gitea;
pub mod github;
pub mod gitlab;
pub mod sourceforge;

pub use archive::ArchiveProvider;
pub use azure_devops::AzureDevOpsProvider;
pub use bitbucket::BitbucketProvider;
pub use codeberg::CodebergProvider;
pub use gitea::GiteaProvider;
pub use github::GitHubProvider;
pub use gitlab::GitLabProvider;
pub use sourceforge::SourceForgeProvider;
