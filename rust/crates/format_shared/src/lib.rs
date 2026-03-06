use viewer_core::archive::OoxmlArchive;
use viewer_core::xml::parse_document;
use viewer_core::ViewerError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentTypeOverride {
    pub part_name: String,
    pub content_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageRelationship {
    pub id: String,
    pub relationship_type: String,
    pub target: String,
    pub resolved_target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedPackageModel {
    pub content_types: Vec<ContentTypeOverride>,
    pub relationships: Vec<PackageRelationship>,
}

pub fn parse_shared_package(archive: &OoxmlArchive) -> Result<SharedPackageModel, ViewerError> {
    let content_types = parse_content_types(archive)?;
    let relationships = parse_package_relationships(archive)?;

    Ok(SharedPackageModel {
        content_types,
        relationships,
    })
}

pub fn parse_content_types(archive: &OoxmlArchive) -> Result<Vec<ContentTypeOverride>, ViewerError> {
    let xml = archive.read_part("[Content_Types].xml")?;
    let text = String::from_utf8(xml).map_err(|_| ViewerError::InvalidDocument)?;
    let root = parse_document(&text)?;

    let mut overrides = Vec::new();
    for child in &root.children {
        if child.local_name() != "Override" {
            continue;
        }

        overrides.push(ContentTypeOverride {
            part_name: child.required_attribute("PartName")?.to_string(),
            content_type: child.required_attribute("ContentType")?.to_string(),
        });
    }

    Ok(overrides)
}

pub fn parse_package_relationships(
    archive: &OoxmlArchive,
) -> Result<Vec<PackageRelationship>, ViewerError> {
    let xml = archive.read_part("_rels/.rels")?;
    let text = String::from_utf8(xml).map_err(|_| ViewerError::InvalidDocument)?;
    let root = parse_document(&text)?;

    let mut relationships = Vec::new();
    for child in &root.children {
        if child.local_name() != "Relationship" {
            continue;
        }

        let target = child.required_attribute("Target")?.to_string();
        let resolved_target = resolve_relationship_target("/", &target);

        if !archive.contains_part(resolved_target.trim_start_matches('/')) {
            return Err(ViewerError::InvalidDocument);
        }

        relationships.push(PackageRelationship {
            id: child.required_attribute("Id")?.to_string(),
            relationship_type: child.required_attribute("Type")?.to_string(),
            target,
            resolved_target,
        });
    }

    Ok(relationships)
}

pub fn resolve_relationship_target(base: &str, target: &str) -> String {
    if target.starts_with('/') {
        return target.to_string();
    }

    let mut parts: Vec<&str> = base.split('/').filter(|part| !part.is_empty()).collect();
    if !base.ends_with('/') {
        let _ = parts.pop();
    }

    for segment in target.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                let _ = parts.pop();
            }
            part => parts.push(part),
        }
    }

    format!("/{}", parts.join("/"))
}
