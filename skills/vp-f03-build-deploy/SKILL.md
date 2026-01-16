---
name: vp-f03-build-deploy
version: 1.0
dependencies: [vp-f01-tech-stack, vp-f02-structure]
mcp_servers: [mental-model, methodology-kb]
---

# VP-F03: Build & Deployment Analysis

## Purpose
Analyze build tooling, CI/CD pipelines, containerization, and deployment configuration to understand the project's operational aspects.

## Prerequisites
- VP-F01 and VP-F02 completed
- Verify by calling `mental-model/get_model` and confirming tech_stack and structure are populated

## Instructions

### Step 1: Verify Prerequisites
Call `mental-model/get_model` and confirm:
- `tech_stack` section is populated
- `structure` section is populated
- `completed_viewpoints` includes "VP-F01" and "VP-F02"

### Step 2: Identify Build Tool

#### Python
- **Poetry**: `poetry.lock`, `pyproject.toml` with `[tool.poetry]`
- **Pip**: `requirements.txt`, `setup.py`
- **Pipenv**: `Pipfile`, `Pipfile.lock`
- **PDM**: `pdm.lock`, `pyproject.toml` with `[tool.pdm]`
- **Hatch**: `pyproject.toml` with `[tool.hatch]`

#### JavaScript/TypeScript
- **npm**: `package-lock.json`
- **yarn**: `yarn.lock`
- **pnpm**: `pnpm-lock.yaml`
- **bun**: `bun.lockb`

#### Rust
- **Cargo**: `Cargo.toml`, `Cargo.lock`

#### Go
- **Go modules**: `go.mod`, `go.sum`

#### General
- **Make**: `Makefile`
- **Just**: `justfile`
- **Task**: `Taskfile.yml`

### Step 3: Identify Package Manager

Note which package manager is used:
- Check for lock files
- Check for workspace configuration
- Note any package manager specific settings

### Step 4: Identify CI/CD Configuration

Look for CI/CD files:

#### GitHub Actions
- `.github/workflows/*.yml`
- Analyze workflow triggers, jobs, steps

#### GitLab CI
- `.gitlab-ci.yml`
- Analyze stages, jobs, rules

#### Jenkins
- `Jenkinsfile`
- `jenkins/` directory

#### CircleCI
- `.circleci/config.yml`

#### Other
- `azure-pipelines.yml` (Azure DevOps)
- `bitbucket-pipelines.yml` (Bitbucket)
- `.travis.yml` (Travis CI)

#### CI/CD Analysis Points
For each CI/CD config, identify:
- Build steps
- Test execution
- Linting/formatting checks
- Security scanning
- Deployment stages
- Environment configurations

### Step 5: Identify Containerization

#### Docker
- `Dockerfile` - Single container
- `docker-compose.yml` - Multi-container setup
- `.dockerignore` - Build context exclusions

Analyze Dockerfile for:
- Base image security
- Multi-stage builds
- Layer optimization
- Non-root user usage
- Health checks

#### Kubernetes
- `k8s/`, `kubernetes/`, `helm/` directories
- `*.yaml` with `apiVersion: apps/v1`
- `Chart.yaml` (Helm)

### Step 6: Identify Deployment Targets

Look for deployment configuration:

#### Cloud Platforms
- **AWS**: `serverless.yml`, `template.yaml` (SAM), `cdk.json`
- **GCP**: `app.yaml`, `cloudbuild.yaml`
- **Azure**: `azure-pipelines.yml`, ARM templates
- **Vercel**: `vercel.json`
- **Netlify**: `netlify.toml`
- **Heroku**: `Procfile`, `app.json`

#### Infrastructure as Code
- **Terraform**: `*.tf` files
- **Pulumi**: `Pulumi.yaml`
- **CloudFormation**: `template.yaml`

### Step 7: Security Analysis of Build/Deploy

Check for security considerations:
- Secrets management (environment variables, vaults)
- Dependency scanning in CI
- Container image scanning
- SAST/DAST integration
- Deployment approvals/gates

### Step 8: Update Mental Model

Call `mental-model/update_viewpoint` with:
```yaml
viewpoint: "VP-F03"
data:
  build_tool: "poetry|npm|cargo|go"
  package_manager: "pip|yarn|pnpm|cargo"
  ci_cd:
    - "github-actions"
    - "gitlab-ci"
  deployment_targets:
    - "kubernetes"
    - "aws-lambda"
    - "docker"
  containerization: "docker|podman|none"
```

## Output Schema

```yaml
build_deploy:
  build_tool: string            # Primary build tool
  package_manager: string       # Package manager used
  ci_cd:                        # CI/CD platforms
    - string
  deployment_targets:           # Where app is deployed
    - string
  containerization: string      # Container technology
```

## Quality Indicators

- ✅ **Healthy**: Automated CI/CD, security scanning, containerized, clear deployment
- ⚠️ **Warning**: Partial automation, missing security scans, manual steps
- ❌ **Critical**: No CI/CD, no containerization for production, manual deployments

## Build/Deploy Best Practices Checklist

- [ ] Automated builds on PR/push
- [ ] Tests run in CI
- [ ] Linting/formatting enforced
- [ ] Dependency vulnerability scanning
- [ ] Docker image scanning
- [ ] Multi-stage Docker builds
- [ ] Non-root container user
- [ ] Environment-specific configs
- [ ] Deployment rollback capability
- [ ] Health checks configured

## Common Patterns

### Python FastAPI + Docker + GitHub Actions
```yaml
build_deploy:
  build_tool: poetry
  package_manager: pip
  ci_cd:
    - github-actions
  deployment_targets:
    - kubernetes
    - docker
  containerization: docker
```

### TypeScript NestJS + Docker + GitLab
```yaml
build_deploy:
  build_tool: npm
  package_manager: npm
  ci_cd:
    - gitlab-ci
  deployment_targets:
    - kubernetes
    - docker
  containerization: docker
```

## Foundation Phase Complete

After VP-F03, the Foundation phase is complete. The mental model now contains:
- Tech stack information (VP-F01)
- Project structure (VP-F02)
- Build and deployment setup (VP-F03)

Proceed to **Structure Phase** starting with **VP-S01: Module Hierarchy**.
