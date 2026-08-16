//! The live knowledge service. Serves the *same* canonical profile data that
//! the Rust/WASM OS core embeds — one source of truth (`knowledge/data/profile.json`),
//! two consumers. Parsed once, never at request time.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

const PROFILE_JSON: &str = include_str!("../../knowledge/data/profile.json");

/// Full profile. Field names mirror `knowledge/data/profile.json` exactly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub person: Person,
    pub highlights: Vec<String>,
    pub skills: Vec<Skill>,
    pub technologies: Vec<String>,
    pub projects: Vec<Project>,
    pub experience: Vec<Experience>,
    pub education: Vec<Education>,
    pub certifications: Vec<Certification>,
    pub achievements: Vec<Achievement>,
    pub contributions: Vec<Contribution>,
    #[serde(default)]
    pub claims: Vec<Claim>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
    pub role: String,
    pub location: String,
    pub summary: String,
    pub contact: Contact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub email: String,
    pub github: String,
    pub linkedin: String,
    pub website: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub category: String,
    pub level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub title: String,
    pub category: String,
    pub summary: String,
    pub description: String,
    pub architecture: String,
    pub technologies: Vec<String>,
    pub highlights: Vec<String>,
    pub repo: String,
    pub demo: String,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub role: String,
    pub organization: String,
    pub period: String,
    pub summary: String,
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Education {
    pub degree: String,
    pub institution: String,
    pub period: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certification {
    pub name: String,
    pub issuer: String,
    pub year: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    pub repo: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub claim: String,
    pub evidence: Vec<String>,
    pub confidence: f32,
}

/// Parses the canonical profile once. A parse failure is a build-time-class bug:
/// it panics loudly on first use rather than silently serving empty data.
pub fn load() -> &'static Profile {
    static PROFILE: OnceLock<Profile> = OnceLock::new();
    PROFILE.get_or_init(|| {
        serde_json::from_str(PROFILE_JSON)
            .expect("knowledge/data/profile.json must parse against the backend Profile schema")
    })
}

/// A stable ETag derived from the raw profile bytes, so CDNs/proxies and the
/// admin panel can cache and 304 efficiently.
pub fn etag() -> &'static str {
    static ETAG: OnceLock<String> = OnceLock::new();
    ETAG.get_or_init(|| {
        let mut hasher = DefaultHasher::new();
        PROFILE_JSON.hash(&mut hasher);
        format!("\"{:016x}\"", hasher.finish())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_parses_and_has_expected_shape() {
        let p = load();
        assert!(!p.person.name.is_empty());
        assert_eq!(p.skills.len(), 23);
        assert!(p.projects.iter().any(|x| x.id == "nikhil-os"));
        assert!(p.claims.iter().any(|c| c.confidence > 0.0));
    }

    #[test]
    fn etag_is_stable() {
        assert_eq!(etag(), etag());
        assert!(etag().starts_with('"') && etag().ends_with('"'));
    }
}
