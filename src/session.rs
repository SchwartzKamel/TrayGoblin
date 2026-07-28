use std::path::PathBuf;

use serde::Deserialize;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkspaceMetadata {
    pub repository: Option<String>,
    pub active_directory: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
struct WorkspaceDocument {
    #[serde(default)]
    repository: Option<Repository>,
    #[serde(
        default,
        alias = "cwd",
        alias = "directory",
        alias = "workingDirectory"
    )]
    active_directory: Option<PathBuf>,
    #[serde(default)]
    workspace: WorkspaceFields,
}

#[derive(Debug, Default, Deserialize)]
struct WorkspaceFields {
    #[serde(default)]
    repository: Option<Repository>,
    #[serde(
        default,
        alias = "cwd",
        alias = "directory",
        alias = "workingDirectory",
        alias = "path"
    )]
    active_directory: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Repository {
    Name(String),
    Coordinates {
        #[serde(default)]
        owner: Option<String>,
        name: String,
    },
}

impl Repository {
    fn into_name(self) -> Option<String> {
        let name = match self {
            Self::Name(name) => name,
            Self::Coordinates {
                owner: Some(owner),
                name,
            } if !owner.trim().is_empty() => format!("{owner}/{name}"),
            Self::Coordinates { name, .. } => name,
        };
        let name = name.trim();
        (!name.is_empty()).then(|| name.to_owned())
    }
}

pub fn parse_workspace_metadata(input: &str) -> Result<WorkspaceMetadata, serde_yaml::Error> {
    let document: WorkspaceDocument = serde_yaml::from_str(input)?;

    Ok(WorkspaceMetadata {
        repository: document
            .repository
            .or(document.workspace.repository)
            .and_then(Repository::into_name),
        active_directory: document
            .active_directory
            .or(document.workspace.active_directory),
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{WorkspaceMetadata, parse_workspace_metadata};

    const WORKSPACE: &str = include_str!("../tests/fixtures/parser/workspace.yaml");

    // This proves only safe workspace identity crosses the session parser boundary.
    #[test]
    fn reads_workspace_metadata_only() {
        let metadata = parse_workspace_metadata(WORKSPACE).expect("fixture should be valid YAML");

        assert_eq!(
            metadata,
            WorkspaceMetadata {
                repository: Some("octo-org/content-free-demo".to_owned()),
                active_directory: Some(Path::new("C:/fixture/content-free-demo").to_path_buf()),
            }
        );

        let modeled = format!("{metadata:?}");
        assert!(!modeled.contains("SENSITIVE_SENTINEL"));
        assert!(!modeled.contains("prompt"));
        assert!(!modeled.contains("credential"));
    }

    // This protects compatibility with the nested metadata shape used by newer producers.
    #[test]
    fn reads_nested_workspace_metadata() {
        let metadata = parse_workspace_metadata(
            "workspace:\n  repository:\n    owner: octo-org\n    name: nested-demo\n  path: C:/fixture/nested-demo\n",
        )
        .expect("nested metadata should be valid");

        assert_eq!(metadata.repository.as_deref(), Some("octo-org/nested-demo"));
        assert_eq!(
            metadata.active_directory.as_deref(),
            Some(Path::new("C:/fixture/nested-demo"))
        );
    }

    // This keeps extra future workspace fields compatible without broad untyped modeling.
    #[test]
    fn ignores_unknown_workspace_fields() {
        let metadata = parse_workspace_metadata(
            "repository: demo\ncwd: C:/fixture/demo\nfutureMetadata:\n  shape: changed\n",
        )
        .expect("unknown fields should be ignored");

        assert_eq!(metadata.repository.as_deref(), Some("demo"));
    }
}
