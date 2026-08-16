//! Knowledge core.
//!
//! The canonical profile dataset lives in `knowledge/data/profile.json`
//! (a single source of truth; see ADR-0003). It is embedded into the core at
//! build time and served through this typed service. Applications never read
//! profile facts directly — they query this service, so Projects, Resume,
//! Recruiter Mode, and (later) RAG all agree.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Contact {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub github: String,
    #[serde(default)]
    pub linkedin: String,
    #[serde(default)]
    pub website: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Person {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub contact: Contact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub architecture: String,
    #[serde(default)]
    pub technologies: Vec<String>,
    #[serde(default)]
    pub highlights: Vec<String>,
    #[serde(default)]
    pub repo: String,
    #[serde(default)]
    pub demo: String,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub organization: String,
    #[serde(default)]
    pub period: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Education {
    #[serde(default)]
    pub degree: String,
    #[serde(default)]
    pub institution: String,
    #[serde(default)]
    pub period: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certification {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub issuer: String,
    #[serde(default)]
    pub year: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    #[serde(default)]
    pub repo: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    #[serde(default)]
    pub claim: String,
    #[serde(default)]
    pub evidence: Vec<String>,
    #[serde(default)]
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub person: Person,
    #[serde(default)]
    pub highlights: Vec<String>,
    #[serde(default)]
    pub skills: Vec<Skill>,
    #[serde(default)]
    pub technologies: Vec<String>,
    #[serde(default)]
    pub projects: Vec<Project>,
    #[serde(default)]
    pub experience: Vec<Experience>,
    #[serde(default)]
    pub education: Vec<Education>,
    #[serde(default)]
    pub certifications: Vec<Certification>,
    #[serde(default)]
    pub achievements: Vec<Achievement>,
    #[serde(default)]
    pub contributions: Vec<Contribution>,
    #[serde(default)]
    pub claims: Vec<Claim>,
}

/// The knowledge service over the canonical embedded dataset.
#[derive(Debug, Clone)]
pub struct KnowledgeService {
    profile: Profile,
}

const PROFILE_JSON: &str = include_str!("../../../knowledge/data/profile.json");

impl KnowledgeService {
    /// Load and validate the canonical profile.
    pub fn load() -> Self {
        let profile: Profile = serde_json::from_str(PROFILE_JSON)
            .expect("knowledge/data/profile.json must be valid JSON matching the schema");
        Self { profile }
    }

    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    pub fn person(&self) -> &Person {
        &self.profile.person
    }

    /// The OS user name is derived from the profile's first name.
    pub fn user_name(&self) -> String {
        let name = &self.profile.person.name;
        let first = name
            .split_whitespace()
            .next()
            .unwrap_or("user")
            .to_lowercase();
        if first.is_empty() {
            "user".to_string()
        } else {
            first
        }
    }

    pub fn projects(&self) -> &[Project] {
        &self.profile.projects
    }

    pub fn project(&self, id: &str) -> Option<&Project> {
        self.profile.projects.iter().find(|p| p.id == id)
    }

    pub fn skills(&self) -> &[Skill] {
        &self.profile.skills
    }

    pub fn experience(&self) -> &[Experience] {
        &self.profile.experience
    }

    pub fn claims(&self) -> &[Claim] {
        &self.profile.claims
    }

    /// Projects in a given category.
    pub fn projects_by_category(&self, category: &str) -> Vec<&Project> {
        self.profile
            .projects
            .iter()
            .filter(|p| p.category.eq_ignore_ascii_case(category))
            .collect()
    }

    /// The skills implicated by a project's technologies.
    pub fn skills_for_project(&self, project: &Project) -> Vec<&Skill> {
        self.profile
            .skills
            .iter()
            .filter(|s| {
                project
                    .technologies
                    .iter()
                    .any(|t| t.eq_ignore_ascii_case(&s.name))
            })
            .collect()
    }

    /// Serialize the whole profile for the applications.
    pub fn profile_json(&self) -> String {
        serde_json::to_string(&self.profile).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_loads() {
        let k = KnowledgeService::load();
        assert!(!k.person().name.is_empty());
        assert!(!k.projects().is_empty());
    }

    #[test]
    fn user_name_derived_from_person() {
        let k = KnowledgeService::load();
        let name = k.user_name();
        assert!(!name.is_empty());
        assert!(name.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn project_lookup_and_categories() {
        let k = KnowledgeService::load();
        let p = k.project("nikhil-os").expect("nikhil-os project present");
        assert!(p.title.contains("NIKHIL//OS"));
        let systems = k.projects_by_category("systems");
        assert!(!systems.is_empty());
    }

    #[test]
    fn profile_serializes() {
        let k = KnowledgeService::load();
        let json = k.profile_json();
        assert!(json.contains("\"person\""));
        assert!(json.contains("\"projects\""));
    }
}
