# Contributing to QuillSpace

Thank you for your interest in contributing to QuillSpace! This document provides guidelines and information for contributors.

## Code of Conduct

We are committed to providing a welcoming and inclusive environment for all contributors. Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Getting Started

### Prerequisites

Before contributing, ensure you have:

- Read the [Development Guide](docs/development.md)
- Set up your local development environment
- Familiarized yourself with the [Architecture](docs/architecture.md)

### Development Workflow

1. **Fork the repository**
   ```bash
   git clone https://github.com/your-username/quillspace.git
   cd quillspace
   ```

2. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**
   - Follow the coding standards below
   - Add tests for new functionality
   - Update documentation as needed

4. **Test your changes**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

5. **Commit your changes**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

6. **Push and create a Pull Request**
   ```bash
   git push origin feature/your-feature-name
   ```

## Coding Standards

### Rust Code Style

- **Formatting**: Use `cargo fmt` for consistent formatting
- **Linting**: All code must pass `cargo clippy` without warnings
- **Documentation**: Public APIs must have rustdoc comments
- **Error Handling**: Use `Result<T, E>` for fallible operations
- **Async**: Prefer async/await for I/O operations

```rust
/// Creates a new content item for the specified tenant.
/// 
/// # Arguments
/// 
/// * `tenant_id` - The UUID of the tenant
/// * `request` - The content creation request
/// 
/// # Returns
/// 
/// Returns the created content item or an error if creation fails.
/// 
/// # Errors
/// 
/// This function will return an error if:
/// - The tenant does not exist
/// - The user lacks permission to create content
/// - Database operation fails
pub async fn create_content(
    tenant_id: Uuid,
    request: CreateContentRequest,
) -> Result<Content, ContentError> {
    // Implementation
}
```

### TypeScript/JavaScript Style

- **Formatting**: Use Prettier for consistent formatting
- **Linting**: All code must pass ESLint without warnings
- **Types**: Use TypeScript for all new code
- **Components**: Follow Qwik component patterns

```typescript
import { component$, useSignal } from '@builder.io/qwik';

interface ButtonProps {
  variant?: 'primary' | 'secondary';
  onClick$?: () => void;
}

export const Button = component$<ButtonProps>(({ 
  variant = 'primary', 
  onClick$ 
}) => {
  const isLoading = useSignal(false);

  return (
    <button
      class={`btn btn-${variant}`}
      onClick$={async () => {
        isLoading.value = true;
        await onClick$?.();
        isLoading.value = false;
      }}
    >
      <Slot />
    </button>
  );
});
```

### Database Migrations

- **Naming**: Use descriptive names with timestamps
- **Reversibility**: Always include both `up` and `down` migrations
- **Safety**: Consider impact on existing data
- **Indexing**: Add appropriate indexes for performance

```sql
-- migrations/20231201120000_add_content_categories.sql
-- Up
CREATE TABLE content_categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR NOT NULL,
    slug VARCHAR NOT NULL,
    description TEXT,
    parent_id UUID REFERENCES content_categories(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(tenant_id, slug)
);

CREATE INDEX idx_content_categories_tenant ON content_categories(tenant_id);
CREATE INDEX idx_content_categories_parent ON content_categories(parent_id);

-- Enable RLS
ALTER TABLE content_categories ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON content_categories
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

-- Down
DROP TABLE IF EXISTS content_categories;
```

## Testing Guidelines

### Unit Tests

Write unit tests for all business logic:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_content_success() {
        let tenant_id = Uuid::new_v4();
        let request = CreateContentRequest {
            title: "Test Article".to_string(),
            content: "Test content".to_string(),
            status: ContentStatus::Draft,
        };

        let result = create_content(tenant_id, request).await;
        
        assert!(result.is_ok());
        let content = result.unwrap();
        assert_eq!(content.title, "Test Article");
        assert_eq!(content.status, ContentStatus::Draft);
    }

    #[tokio::test]
    async fn test_create_content_invalid_tenant() {
        let invalid_tenant_id = Uuid::new_v4();
        let request = CreateContentRequest {
            title: "Test".to_string(),
            content: "Test".to_string(),
            status: ContentStatus::Draft,
        };

        let result = create_content(invalid_tenant_id, request).await;
        
        assert!(result.is_err());
        assert_matches!(result.unwrap_err(), ContentError::TenantNotFound);
    }
}
```

### Integration Tests

Test API endpoints and database interactions:

```rust
#[tokio::test]
async fn test_content_api_create() {
    let app = create_test_app().await;
    let tenant = create_test_tenant().await;
    let token = create_test_jwt(&tenant).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/content")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "title": "Test Article",
                    "content": "Test content",
                    "status": "draft"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}
```

### Frontend Tests

Test Qwik components:

```typescript
import { createDOM } from '@builder.io/qwik/testing';
import { test, expect } from 'vitest';
import { ContentList } from './content-list';

test('ContentList renders empty state', async () => {
  const { screen, render } = await createDOM();
  
  await render(<ContentList content={[]} />);
  
  expect(screen.querySelector('[data-testid="empty-state"]')).toBeTruthy();
});

test('ContentList renders content items', async () => {
  const { screen, render } = await createDOM();
  const mockContent = [
    { id: '1', title: 'Article 1', status: 'published' },
    { id: '2', title: 'Article 2', status: 'draft' }
  ];
  
  await render(<ContentList content={mockContent} />);
  
  expect(screen.querySelectorAll('[data-testid="content-item"]')).toHaveLength(2);
});
```

## Documentation

### API Documentation

- Update OpenAPI specifications for new endpoints
- Include request/response examples
- Document error conditions
- Add rate limiting information

### Code Documentation

- Document public APIs with rustdoc/JSDoc
- Include usage examples
- Explain complex algorithms
- Document configuration options

### User Documentation

- Update user guides for new features
- Include screenshots for UI changes
- Provide migration guides for breaking changes
- Update FAQ and troubleshooting sections

## Pull Request Process

### PR Title Format

Use conventional commit format:

- `feat: add new feature`
- `fix: resolve bug in authentication`
- `docs: update API documentation`
- `refactor: simplify content service`
- `test: add integration tests for users`
- `chore: update dependencies`

### PR Description Template

```markdown
## Description
Brief description of the changes and motivation.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed
- [ ] Performance impact assessed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Breaking changes documented
- [ ] Database migrations included (if applicable)

## Screenshots (if applicable)
Include screenshots for UI changes.

## Related Issues
Closes #123
```

### Review Process

1. **Automated Checks**: All CI checks must pass
2. **Code Review**: At least one maintainer approval required
3. **Testing**: Comprehensive test coverage for new features
4. **Documentation**: Updated documentation for user-facing changes

## Issue Reporting

### Bug Reports

Use the bug report template:

```markdown
**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

**Expected behavior**
A clear and concise description of what you expected to happen.

**Screenshots**
If applicable, add screenshots to help explain your problem.

**Environment:**
- OS: [e.g. macOS 12.0]
- Browser: [e.g. Chrome 96]
- QuillSpace Version: [e.g. 1.2.3]

**Additional context**
Add any other context about the problem here.
```

### Feature Requests

Use the feature request template:

```markdown
**Is your feature request related to a problem?**
A clear and concise description of what the problem is.

**Describe the solution you'd like**
A clear and concise description of what you want to happen.

**Describe alternatives you've considered**
A clear and concise description of any alternative solutions or features you've considered.

**Additional context**
Add any other context or screenshots about the feature request here.
```

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist

- [ ] Update version numbers in `Cargo.toml` and `package.json`
- [ ] Update `CHANGELOG.md`
- [ ] Run full test suite
- [ ] Update documentation
- [ ] Create release notes
- [ ] Tag release in Git
- [ ] Deploy to staging environment
- [ ] Deploy to production environment
- [ ] Announce release

## Community

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and community discussion
- **Discord**: Real-time chat with maintainers and community
- **Email**: security@quillspace.com for security issues

### Recognition

Contributors will be recognized in:

- Release notes for significant contributions
- Contributors section in README
- Annual contributor spotlight blog posts

## Security

### Reporting Security Issues

**Do not report security vulnerabilities through public GitHub issues.**

Instead, email security@quillspace.com with:

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will respond within 48 hours and work with you to resolve the issue.

### Security Best Practices

- Never commit secrets or API keys
- Use environment variables for configuration
- Validate all user inputs
- Follow OWASP security guidelines
- Keep dependencies updated

## License

By contributing to QuillSpace, you agree that your contributions will be licensed under the MIT License.

## Questions?

If you have questions about contributing, please:

1. Check the documentation
2. Search existing issues and discussions
3. Join our Discord community
4. Open a new discussion

Thank you for contributing to QuillSpace! 🚀
