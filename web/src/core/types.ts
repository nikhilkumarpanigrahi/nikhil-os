// TypeScript mirrors of the JSON the Rust core emits. These shapes come
// straight from the crate's serde definitions — keep them in sync with
// crates/core/src/*.rs.

export type ProcessState =
  | "NEW"
  | "READY"
  | "RUNNING"
  | "WAITING"
  | "TERMINATED";

export interface Process {
  pid: number;
  parent_pid: number;
  name: string;
  user: string;
  state: ProcessState;
  priority: number;
  cpu_usage: number;
  memory_kb: number;
  capabilities: string[];
  start_tick: number;
  cpu_ticks: number;
}

export interface MemorySnapshot {
  total_kb: number;
  used_kb: number;
  free_kb: number;
  used_percent: number;
}

export type ServiceState =
  | "stopped"
  | "starting"
  | "running"
  | "restarting"
  | "failed";

export interface Service {
  name: string;
  description: string;
  dependencies: string[];
  state: ServiceState;
  pid: number | null;
  start_tick: number;
  restarts: number;
  restart_policy: "always" | "on_failure" | "never";
  uptime_ticks: number;
}

export interface SchedulerStats {
  algorithm: string;
  context_switches: number;
  current_pid: number;
  time_slice_ticks: number;
}

export interface Snapshot {
  tick: number;
  cpu: number;
  processes: Process[];
  memory: MemorySnapshot;
  services: Service[];
  scheduler: SchedulerStats;
}

export type FileType = "file" | "directory";

export interface DirEntry {
  name: string;
  file_type: FileType;
  size: number;
  perms: string;
  owner: string;
  group: string;
}

export interface FileStat {
  path: string;
  file_type: FileType;
  size: number;
  perms: string;
  owner: string;
  group: string;
  links: number;
  mtime: number;
}

// ---- knowledge / profile ------------------------------------------------

export interface Contact {
  email: string;
  github: string;
  linkedin: string;
  website: string;
}

export interface Person {
  name: string;
  role: string;
  location: string;
  summary: string;
  contact: Contact;
}

export interface Skill {
  name: string;
  category: string;
  level: number;
}

export interface Evidence {
  title: string;
  url: string;
}

export interface Project {
  id: string;
  title: string;
  category: string;
  summary: string;
  description: string;
  architecture: string;
  technologies: string[];
  highlights: string[];
  repo: string;
  demo: string;
  evidence: Evidence[];
}

export interface Experience {
  role: string;
  organization: string;
  period: string;
  summary: string;
  highlights: string[];
}

export interface Education {
  degree: string;
  institution: string;
  period: string;
}

export interface Certification {
  name: string;
  issuer: string;
  year: string;
}

export interface Achievement {
  title: string;
  description: string;
  evidence: string[];
}

export interface Contribution {
  repo: string;
  description: string;
}

export interface Claim {
  claim: string;
  evidence: string[];
  confidence: number;
}

export interface Profile {
  person: Person;
  highlights: string[];
  skills: Skill[];
  technologies: string[];
  projects: Project[];
  experience: Experience[];
  education: Education[];
  certifications: Certification[];
  achievements: Achievement[];
  contributions: Contribution[];
  claims: Claim[];
}

export interface BootStatus {
  title: string;
  detail: string;
  ok: boolean;
}
