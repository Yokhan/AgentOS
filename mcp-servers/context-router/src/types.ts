export interface Route {
  keywords: RegExp;
  files: string[];
  agent: string;
  codexSkills: string[];
  codexSubagents: string[];
  pipeline: string;
  risk: "LOW" | "MEDIUM" | "HIGH" | "CRITICAL";
  needsFreshDocs?: boolean;
}

export interface RouteResult {
  modes: string[];
  agent: string;
  files: string[];
  codexSkills: string[];
  codexSubagents: string[];
  pipeline: string;
  risk: "LOW" | "MEDIUM" | "HIGH" | "CRITICAL";
  codeIntelligence: {
    id: string;
    tools: string[];
    reason: string;
    guards: string[];
  };
  needsFreshDocs: boolean;
  targetLanguage: string | null;
  languageResolution: string | null;
  writingProfiles: string[];
  writingLanguageProfiles: string[];
  writingProcessProfiles: string[];
  writingDomainProfiles: string[];
  writingTechnicalProfiles: string[];
  writingEditors: string[];
  writingGates: string[];
  writingExternalTools: Array<{
    id: string;
    access: string;
    execution: string;
    paid: boolean;
  }>;
  writingRejectedProfiles: Array<{ id: string; reason: string }>;
}

export interface ServerState {
  currentModes: string[];
  activeRules: string[];
  lastRouteTime: string;
  taskDescription: string;
}

export interface ProjectContext {
  lessons: string;
  research: string;
  gitLog: string;
  currentTask: string;
  toolRegistry: string;
  ecosystem: string;
}
