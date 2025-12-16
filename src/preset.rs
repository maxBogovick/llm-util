//! LLM preset configurations for different use cases.
//!
//! This module provides pre-configured templates for common LLM tasks like
//! code review, documentation generation, refactoring, and more.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of preset for LLM tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresetKind {
    /// Comprehensive code review
    CodeReview,
    /// Documentation generation
    Documentation,
    /// Refactoring suggestions
    Refactoring,
    /// Bug detection and analysis
    BugAnalysis,
    /// Security audit
    SecurityAudit,
    /// Test generation
    TestGeneration,
    /// Architecture review
    ArchitectureReview,
    /// Performance analysis
    PerformanceAnalysis,
    /// Migration planning
    MigrationPlan,
    /// API design review
    ApiDesign,
}

impl PresetKind {
    /// Returns the ID string for this preset.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::CodeReview => "code-review",
            Self::Documentation => "documentation",
            Self::Refactoring => "refactoring",
            Self::BugAnalysis => "bug-analysis",
            Self::SecurityAudit => "security-audit",
            Self::TestGeneration => "test-generation",
            Self::ArchitectureReview => "architecture-review",
            Self::PerformanceAnalysis => "performance-analysis",
            Self::MigrationPlan => "migration-plan",
            Self::ApiDesign => "api-design",
        }
    }

    /// Returns all available preset kinds.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::CodeReview,
            Self::Documentation,
            Self::Refactoring,
            Self::BugAnalysis,
            Self::SecurityAudit,
            Self::TestGeneration,
            Self::ArchitectureReview,
            Self::PerformanceAnalysis,
            Self::MigrationPlan,
            Self::ApiDesign,
        ]
    }

    /// Parse preset kind from string ID.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "code-review" => Some(Self::CodeReview),
            "documentation" => Some(Self::Documentation),
            "refactoring" => Some(Self::Refactoring),
            "bug-analysis" => Some(Self::BugAnalysis),
            "security-audit" => Some(Self::SecurityAudit),
            "test-generation" => Some(Self::TestGeneration),
            "architecture-review" => Some(Self::ArchitectureReview),
            "performance-analysis" => Some(Self::PerformanceAnalysis),
            "migration-plan" => Some(Self::MigrationPlan),
            "api-design" => Some(Self::ApiDesign),
            _ => None,
        }
    }
}

/// Preset configuration for LLM tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMPreset {
    /// Unique preset identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of the preset
    pub description: String,
    /// System prompt for the LLM
    pub system_prompt: String,
    /// User prompt template
    pub user_prompt_template: String,
    /// Suggested model for this task
    pub suggested_model: String,
    /// Maximum tokens hint
    pub max_tokens_hint: usize,
    /// Temperature hint for generation
    pub temperature_hint: f32,
    /// Include metadata in output
    pub include_metadata: bool,
    /// Include directory structure
    pub include_structure: bool,
    /// Code block style
    pub code_block_style: CodeBlockStyle,
}

/// Code block formatting style.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CodeBlockStyle {
    /// Markdown code blocks
    Markdown,
    /// XML CDATA sections
    Xml,
    /// Inline code
    Inline,
}

impl LLMPreset {
    /// Creates a preset for the given kind.
    #[must_use]
    pub fn for_kind(kind: PresetKind) -> Self {
        match kind {
            PresetKind::CodeReview => Self::code_review(),
            PresetKind::Documentation => Self::documentation(),
            PresetKind::Refactoring => Self::refactoring(),
            PresetKind::BugAnalysis => Self::bug_analysis(),
            PresetKind::SecurityAudit => Self::security_audit(),
            PresetKind::TestGeneration => Self::test_generation(),
            PresetKind::ArchitectureReview => Self::architecture_review(),
            PresetKind::PerformanceAnalysis => Self::performance_analysis(),
            PresetKind::MigrationPlan => Self::migration_plan(),
            PresetKind::ApiDesign => Self::api_design(),
        }
    }

    /// Creates a collection of all standard presets.
    #[must_use]
    pub fn all_presets() -> HashMap<String, Self> {
        let mut presets = HashMap::new();

        for kind in PresetKind::all() {
            let preset = Self::for_kind(*kind);
            presets.insert(kind.id().to_string(), preset);
        }

        presets
    }

    fn code_review() -> Self {
        Self {
            id: "code-review".to_string(),
            name: "Code Review".to_string(),
            description: "Comprehensive code review with best practices".to_string(),
            system_prompt: r#"You are an expert code reviewer with deep knowledge across multiple programming languages and paradigms.
Your task is to perform a thorough code review focusing on:
- Code quality and maintainability
- Performance issues and optimizations
- Security vulnerabilities
- Best practices and design patterns
- Potential bugs and edge cases
- Documentation completeness
- Test coverage gaps

Provide actionable feedback with specific examples and suggestions."#.to_string(),
            user_prompt_template: r#"Please review this codebase and provide detailed feedback.

**Project Overview:**
- Total Files: {file_count}
- Total Lines: {total_lines}
- Languages: {languages}
- Estimated Tokens: {total_tokens}

**Review Focus Areas:**
1. Architecture and design patterns
2. Code quality and maintainability
3. Performance bottlenecks
4. Security concerns
5. Error handling
6. Testing strategy

**Codebase:**
{code_content}

Please structure your review with:
1. Executive Summary
2. Critical Issues (🔴)
3. Important Issues (🟠)
4. Suggestions (🟡)
5. Positive Aspects (✅)
6. Recommendations"#.to_string(),
            suggested_model: "claude-sonnet-4".to_string(),
            max_tokens_hint: 150_000,
            temperature_hint: 0.3,
            include_metadata: true,
            include_structure: true,
            code_block_style: CodeBlockStyle::Markdown,
        }
    }

    fn documentation() -> Self {
        Self {
            id: "documentation".to_string(),
            name: "Documentation Generation".to_string(),
            description: "Generate comprehensive project documentation".to_string(),
            system_prompt: r#"You are a technical documentation expert. Generate clear, comprehensive documentation that includes:
- Project overview and purpose
- Architecture explanation
- API documentation
- Usage examples
- Setup instructions
- Contributing guidelines

Write in a clear, professional style suitable for both beginners and experienced developers."#.to_string(),
            user_prompt_template: r#"Generate comprehensive documentation for this project.

**Project Information:**
- Files: {file_count}
- Languages: {languages}
- Total Code: {total_lines} lines

**Documentation Requirements:**
1. README.md with:
   - Project description
   - Features
   - Installation
   - Quick start
   - Usage examples
2. API Documentation
3. Architecture overview
4. Development guide

**Codebase:**
{code_content}

Generate structured markdown documentation ready to use."#.to_string(),
            suggested_model: "claude-sonnet-4".to_string(),
            max_tokens_hint: 100_000,
            temperature_hint: 0.5,
            include_metadata: true,
            include_structure: true,
            code_block_style: CodeBlockStyle::Markdown,
        }
    }

    fn refactoring() -> Self {
        Self {
            id: "refactoring".to_string(),
            name: "Refactoring Suggestions".to_string(),
            description: "Get refactoring recommendations to improve code quality".to_string(),
            system_prompt: r#"You are a code refactoring expert. Analyze the code and suggest:
- Code duplication removal
- Design pattern applications
- Improved abstractions
- Better naming conventions
- Simplified complex logic
- Enhanced modularity

Provide concrete before/after examples for each suggestion."#.to_string(),
            user_prompt_template: r#"Analyze this codebase and provide refactoring recommendations.

**Codebase Stats:**
- Files: {file_count}
- Total Lines: {total_lines}
- Languages: {languages}

**Refactoring Goals:**
1. Reduce code duplication
2. Improve readability
3. Enhance maintainability
4. Apply design patterns where appropriate
5. Simplify complex functions

**Code:**
{code_content}

For each refactoring suggestion, provide:
- Current issue
- Proposed solution with code example
- Benefits
- Implementation priority"#.to_string(),
            suggested_model: "claude-sonnet-4".to_string(),
            max_tokens_hint: 120_000,
            temperature_hint: 0.4,
            include_metadata: true,
            include_structure: true,
            code_block_style: CodeBlockStyle::Markdown,
        }
    }

    fn bug_analysis() -> Self {
        Self {
            id: "bug-analysis".to_string(),
            name: "Bug Detection & Analysis".to_string(),
            description: "Identify potential bugs and edge cases".to_string(),
            system_prompt: r#"You are a bug detection expert. Analyze code for:
- Null pointer/undefined access
- Race conditions
- Memory leaks
- Off-by-one errors
- Unhandled edge cases
- Resource leaks
- Logic errors
- Type safety issues

Rate each finding by severity: Critical, High, Medium, Low."#.to_string(),
            user_prompt_template: r#"Analyze this codebase for potential bugs and issues.

**Project Info:**
- Files: {file_count}
- Languages: {languages}
- Total Lines: {total_lines}

**Analysis Focus:**
1. Runtime errors
2. Logic errors
3. Edge cases
4. Resource management
5. Concurrency issues

**Codebase:**
{code_content}

For each bug, provide:
- Severity level
- Location (file:line)
- Description
- Reproduction scenario
- Fix suggestion"#.to_string(),
            suggested_model: "claude-sonnet-4".to_string(),
            max_tokens_hint: 100_000,
            temperature_hint: 0.2,
            include_metadata: true,
            include_structure: false,
            code_block_style: CodeBlockStyle::Markdown,
        }
    }

    fn security_audit() -> Self {
        Self {
            id: "security-audit".to_string(),
            name: "Security Audit".to_string(),
            description: "Comprehensive security vulnerability assessment".to_string(),
            system_prompt: r#"You are a security expert. Audit the code for:
- SQL injection vulnerabilities
- XSS vulnerabilities
- Authentication/authorization flaws
- Insecure data storage
- Cryptographic weaknesses
- Input validation issues
- Secrets in code
- Dependency vulnerabilities

Use OWASP Top 10 as a reference framework."#.to_string(),
            user_prompt_template: r#"Perform a security audit of this codebase.

**Project Details:**
- Files: {file_count}
- Languages: {languages}

**Security Checklist:**
1. Authentication & Authorization
2. Input validation
3. Data encryption
4. Secret management
5. Dependencies security
6. API security
7. Error handling

**Code to Audit:**
{code_content}

For each security issue:
- Severity: Critical/High/Medium/Low
- CWE ID (if applicable)
- Location
- Vulnerability description
- Exploit scenario
- Remediation steps"#.to_string(),
            suggested_model: "claude-sonnet-4".to_string(),
            max_tokens_hint: 120_000,
            temperature_hint: 0.2,
            include_metadata: true,
            include_structure: true,
            code_block_style: CodeBlockStyle::Markdown,
        }
    }

    fn test_generation() -> Self {
        Self {
            id: "test-generation".to_string(),
            name: "Test Suite Generation".to_string(),
            description: "Generate comprehensive test cases".to_string(),
            system_prompt: r#"You are a test automation expert. Generate:
- Unit tests for all functions
- Integration tests for modules
- Edge case tests
- Property-based tests where applicable
- Mock/stub suggestions
- Test data examples

Use the project's testing framework and conventions."#.to_string(),
            user_prompt_template: r#"Generate comprehensive tests for this codebase.

**Project Stats:**
- Files: {file_count}
- Languages: {languages}

**Test Requirements:**
1. Unit tests with >80% coverage
2. Integration tests
3. Edge case coverage
4. Mock/stub strategies
5. Test documentation

**Code:**
{code_content}

Generate tests with:
- Clear test names
- Arrange-Act-Assert pattern
- Edge cases
- Error scenarios
- Documentation"#.to_string(),
            suggested_model: "claude-sonnet-4".to_string(),
            max_tokens_hint: 150_000,
            temperature_hint: 0.4,
            include_metadata: true,
            include_structure: true,
            code_block_style: CodeBlockStyle::Markdown,
        }
    }

    fn architecture_review() -> Self {
        Self {
            id: "architecture-review".to_string(),
            name: "Architecture Review".to_string(),
            description: "Evaluate system architecture and design".to_string(),
            system_prompt: r#"You are a software architect. Review:
- System architecture
- Component relationships
- Design patterns usage
- Separation of concerns
- Scalability considerations
- Maintainability
- Technology choices

Provide architectural diagrams and improvement suggestions."#.to_string(),
            user_prompt_template: r#"Review the architecture of this system.

**Project Overview:**
- Files: {file_count}
- Languages: {languages}
- Structure: {directory_structure}

**Architecture Review Points:**
1. Overall architecture pattern
2. Module organization
3. Dependencies and coupling
4. Scalability design
5. Error handling strategy
6. Data flow

**Codebase:**
{code_content}

Provide:
1. Current architecture assessment
2. Strengths and weaknesses
3. Recommended improvements
4. Migration strategy (if needed)
5. Architecture diagram (mermaid)"#.to_string(),
            suggested_model: "claude-opus-4".to_string(),
            max_tokens_hint: 100_000,
            temperature_hint: 0.4,
            include_metadata: true,
            include_structure: true,
            code_block_style: CodeBlockStyle::Markdown,
        }
    }

    fn performance_analysis() -> Self {
        Self {
            id: "performance-analysis".to_string(),
            name: "Performance Analysis".to_string(),
            description: "Identify performance bottlenecks and optimization opportunities".to_string(),
            system_prompt: r#"You are a performance optimization expert. Analyze:
- Algorithm complexity (Big O)
- Memory usage patterns
- I/O operations
- Database query optimization
- Caching opportunities
- Parallelization potential
- Resource management

Prioritize optimizations by impact."#.to_string(),
            user_prompt_template: r#"Analyze performance characteristics of this codebase.

**Project Info:**
- Files: {file_count}
- Languages: {languages}
- Total Lines: {total_lines}

**Performance Focus:**
1. Algorithmic complexity
2. Memory efficiency
3. I/O optimization
4. Caching strategies
5. Concurrency utilization

**Code:**
{code_content}

For each optimization:
- Current bottleneck
- Impact level (High/Medium/Low)
- Optimization strategy
- Expected improvement
- Implementation complexity"#.to_string(),
            suggested_model: "claude-sonnet-4".to_string(),
            max_tokens_hint: 120_000,
            temperature_hint: 0.3,
            include_metadata: true,
            include_structure: false,
            code_block_style: CodeBlockStyle::Markdown,
        }
    }

    fn migration_plan() -> Self {
        Self {
            id: "migration-plan".to_string(),
            name: "Migration Planning".to_string(),
            description: "Create a plan for technology migration or upgrade".to_string(),
            system_prompt: r#"You are a migration specialist. Create detailed migration plans covering:
- Current state analysis
- Target state definition
- Step-by-step migration path
- Risk assessment
- Rollback strategy
- Testing approach
- Timeline estimation

Consider backward compatibility and minimal disruption."#.to_string(),
            user_prompt_template: r#"Create a migration plan for this project.

**Current Project:**
- Files: {file_count}
- Languages: {languages}
- Dependencies: {dependencies}

**Migration Goal:** [User to specify: e.g., "Migrate from Python 2 to Python 3"]

**Code:**
{code_content}

Provide:
1. Current state analysis
2. Migration challenges
3. Step-by-step plan
4. Code changes needed
5. Testing strategy
6. Risk mitigation
7. Timeline estimate"#.to_string(),
            suggested_model: "claude-opus-4".to_string(),
            max_tokens_hint: 100_000,
            temperature_hint: 0.5,
            include_metadata: true,
            include_structure: true,
            code_block_style: CodeBlockStyle::Markdown,
        }
    }

    fn api_design() -> Self {
        Self {
            id: "api-design".to_string(),
            name: "API Design Review".to_string(),
            description: "Review and improve API design".to_string(),
            system_prompt: r#"You are an API design expert. Review APIs for:
- RESTful principles
- Consistency
- Documentation
- Error handling
- Versioning strategy
- Security
- Performance
- Developer experience

Suggest improvements following industry best practices."#.to_string(),
            user_prompt_template: r#"Review the API design in this codebase.

**Project Info:**
- Files: {file_count}
- Languages: {languages}

**API Review Areas:**
1. Endpoint design
2. Request/response formats
3. Error handling
4. Authentication/authorization
5. Rate limiting
6. Documentation
7. Versioning

**Code:**
{code_content}

Provide:
- API inventory
- Design issues
- Improvement suggestions
- OpenAPI/Swagger spec (if applicable)
- Best practice recommendations"#.to_string(),
            suggested_model: "claude-sonnet-4".to_string(),
            max_tokens_hint: 100_000,
            temperature_hint: 0.4,
            include_metadata: true,
            include_structure: true,
            code_block_style: CodeBlockStyle::Markdown,
        }
    }
}