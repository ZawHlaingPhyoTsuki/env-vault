export interface Project {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface ProjectSummary extends Project {
  environment_count: number;
  variable_count: number;
}

export interface Environment {
  id: string;
  project_id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface EnvironmentSummary extends Environment {
  variable_count: number;
}

export interface Variable {
  id: string;
  environment_id: string;
  key: string;
  comment: string | null;
  position: number;
  created_at: string;
  updated_at: string;
}

export interface VariableRow {
  id: string;
  environment_id: string;
  key: string;
  masked_value: string;
  has_value: boolean;
  comment: string | null;
  position: number;
  created_at: string;
  updated_at: string;
}

export interface RevealedValue {
  id: string;
  value: string;
}
