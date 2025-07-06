export interface LanguageStatistics {
  name: string;
  files: number;
  lines: number;
  code: number;
  comments: number;
  blanks: number;
  share: number;
}

export interface Analysis {
  project_name: string;
  summary: {
    total_files: number;
    total_lines: number;
    code_lines: number;
    comment_lines: number;
    blank_lines: number;
    languages: number;
    primary_language: string;
    code_ratio: number;
    documentation_ratio: number;
  };
  language_statistics: LanguageStatistics[];
}

export interface ComparisonResult {
  repositories: Analysis[];
  comparison: {
    total_files_diff: number;
    total_lines_diff: number;
    code_lines_diff: number;
    comment_lines_diff: number;
    blank_lines_diff: number;
    common_languages: string[];
    unique_languages: {
      [repo: string]: string[];
    };
  };
} 