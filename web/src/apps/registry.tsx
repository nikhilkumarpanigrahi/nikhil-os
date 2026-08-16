import type { ComponentType, ReactNode } from "react";
import { Files } from "./Files";
import { Projects } from "./Projects";
import { Recruiter } from "./Recruiter";
import { Resume } from "./Resume";
import { SystemMonitor } from "./SystemMonitor";
import { Terminal } from "./Terminal";
import { Welcome } from "./Welcome";

export type AppId =
  | "terminal"
  | "files"
  | "projects"
  | "resume"
  | "recruiter"
  | "system-monitor"
  | "welcome";

export interface AppDef {
  id: AppId;
  title: string;
  icon: ReactNode;
  component: ComponentType;
  defaultSize: { w: number; h: number };
}

const Icon = ({ children }: { children: ReactNode }) => (
  <svg
    width="18"
    height="18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.8"
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden
  >
    {children}
  </svg>
);

export const APPS: Record<AppId, AppDef> = {
  terminal: {
    id: "terminal",
    title: "Terminal",
    icon: (
      <Icon>
        <path d="M4 17l6-5-6-5" />
        <path d="M12 19h8" />
      </Icon>
    ),
    component: Terminal,
    defaultSize: { w: 680, h: 440 },
  },
  files: {
    id: "files",
    title: "Files",
    icon: (
      <Icon>
        <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
      </Icon>
    ),
    component: Files,
    defaultSize: { w: 720, h: 480 },
  },
  projects: {
    id: "projects",
    title: "Projects",
    icon: (
      <Icon>
        <rect x="3" y="3" width="7" height="7" rx="1.5" />
        <rect x="14" y="3" width="7" height="7" rx="1.5" />
        <rect x="3" y="14" width="7" height="7" rx="1.5" />
        <rect x="14" y="14" width="7" height="7" rx="1.5" />
      </Icon>
    ),
    component: Projects,
    defaultSize: { w: 780, h: 540 },
  },
  resume: {
    id: "resume",
    title: "Resume",
    icon: (
      <Icon>
        <path d="M6 2h9l5 5v15a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V3a1 1 0 0 1 1-1z" />
        <path d="M14 2v6h6" />
        <path d="M8 13h8M8 17h8M8 9h3" />
      </Icon>
    ),
    component: Resume,
    defaultSize: { w: 740, h: 560 },
  },
  recruiter: {
    id: "recruiter",
    title: "Recruiter",
    icon: (
      <Icon>
        <circle cx="9" cy="8" r="3.2" />
        <path d="M3.5 20a5.5 5.5 0 0 1 11 0" />
        <path d="M17 7.5v5M14.5 10h5" />
      </Icon>
    ),
    component: Recruiter,
    defaultSize: { w: 640, h: 500 },
  },
  "system-monitor": {
    id: "system-monitor",
    title: "System Monitor",
    icon: (
      <Icon>
        <path d="M3 3v18h18" />
        <path d="M7 14l3-4 3 2 4-6" />
      </Icon>
    ),
    component: SystemMonitor,
    defaultSize: { w: 760, h: 520 },
  },
  welcome: {
    id: "welcome",
    title: "Welcome",
    icon: (
      <Icon>
        <path d="M12 3l8 4v5c0 5-3.5 8-8 9-4.5-1-8-4-8-9V7z" />
        <path d="M9 12l2 2 4-4" />
      </Icon>
    ),
    component: Welcome,
    defaultSize: { w: 560, h: 400 },
  },
};

export const APP_ORDER: AppId[] = [
  "terminal",
  "files",
  "projects",
  "resume",
  "recruiter",
  "system-monitor",
  "welcome",
];
