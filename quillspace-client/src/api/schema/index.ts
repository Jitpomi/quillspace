/**
 * Comprehensive Zod Schemas for QuillSpace Database Resources
 * Generated from database migrations and backend types
 */
import {z} from "@builder.io/qwik-city";


// ============================================================================
// ENUMS & CONSTANTS
// ============================================================================

export const UserRoleSchema = z.enum(['admin', 'editor', 'viewer']);
export const ContentStatusSchema = z.enum(['draft', 'published', 'archived']);
export const BuildStatusSchema = z.enum(['draft', 'building', 'published', 'error']);
export const BuildTypeSchema = z.enum(['ssg', 'ssr', 'preview']);
export const SslStatusSchema = z.enum(['pending', 'issued', 'error', 'expired']);
export const VerificationMethodSchema = z.enum(['txt', 'cname', 'file']);
export const MetricTypeSchema = z.enum(['page_views', 'bounce_rate', 'load_time']);

// ============================================================================
// CORE TENANT & USER SCHEMAS
// ============================================================================

export const TenantSchema = z.object({
    id: z.string().uuid(),
    name: z.string().min(1).max(255),
    slug: z.string().min(1).max(100).regex(/^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$/),
    settings: z.record(z.any()).default({}),
    created_at: z.string().datetime(),
    updated_at: z.string().datetime(),
    is_active: z.boolean().default(true),
});

export const UserSchema = z.object({
    id: z.string().uuid(),
    tenant_id: z.string().uuid(),
    email: z.string().email().max(255),
    first_name: z.string().min(1).max(255),
    last_name: z.string().min(1).max(255),
    password_hash: z.string().max(255).optional(),
    role: UserRoleSchema.default('viewer'),
    created_at: z.string().datetime(),
    updated_at: z.string().datetime(),
    active: z.boolean().default(true),
});

// ============================================================================
// CONTENT & PUBLISHING SCHEMAS
// ============================================================================

export const ContentSchema = z.object({
    id: z.string().uuid(),
    tenant_id: z.string().uuid(),
    title: z.string().min(1).max(500),
    slug: z.string().min(1).max(200).regex(/^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$/),
    body: z.string(),
    status: ContentStatusSchema.default('draft'),
    author_id: z.string().uuid(),
    published_at: z.string().datetime().optional(),
    created_at: z.string().datetime(),
    updated_at: z.string().datetime(),
});

// ============================================================================
// WEB BUILDER SCHEMAS
// ============================================================================

export const TemplateSchema = z.object({
    id: z.string().uuid(),
    tenant_id: z.string().uuid().optional(),
    name: z.string().min(1).max(255),
    description: z.string().optional(),
    category: z.string().max(100).default('custom'),
    html_source: z.string(),
    default_schema: z.record(z.any()).default({}),
    preview_image_url: z.string().url().optional(),
    is_public: z.boolean().default(false),
    version: z.number().int().positive().default(1),
    created_at: z.string().datetime(),
    updated_at: z.string().datetime(),
});

export const TemplateVersionSchema = z.object({
    id: z.string().uuid(),
    template_id: z.string().uuid(),
    version: z.number().int().positive(),
    html_source: z.string(),
    schema: z.record(z.any()).default({}),
    changelog: z.string().optional(),
    created_at: z.string().datetime(),
});

export const SiteSchema = z.object({
    id: z.string().uuid(),
    tenant_id: z.string().uuid(),
    name: z.string().min(1).max(255),
    description: z.string().optional(),
    template_id: z.string().uuid().optional(),
    custom_domain: z.string().max(255).optional(),
    subdomain: z.string().max(100).regex(/^[a-z0-9][a-z0-9-]*[a-z0-9]$/).optional(),
    is_published: z.boolean().default(false),
    seo_settings: z.record(z.any()).default({}),
    build_status: BuildStatusSchema.default('draft'),
    theme_config: z.record(z.any()).default({}),
    created_at: z.string().datetime(),
    updated_at: z.string().datetime(),
});

export const PageSchema = z.object({
    id: z.string().uuid(),
    site_id: z.string().uuid(),
    slug: z.string().min(1).max(200).regex(/^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$/),
    title: z.string().min(1).max(500),
    meta_description: z.string().optional(),
    meta_keywords: z.string().optional(),
    puck_data: z.record(z.any()).default({}),
    is_published: z.boolean().default(false),
    published_html: z.string().optional(),
    published_at: z.string().datetime().optional(),
    sort_order: z.number().int().default(0),
    created_at: z.string().datetime(),
    updated_at: z.string().datetime(),
});

export const DomainSchema = z.object({
    id: z.string().uuid(),
    site_id: z.string().uuid(),
    domain: z.string().max(255).regex(/^[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9]*\.[a-zA-Z]{2,}$/),
    verification_token: z.string().max(255),
    verification_method: VerificationMethodSchema.default('txt'),
    is_verified: z.boolean().default(false),
    dns_configured: z.boolean().default(false),
    ssl_status: SslStatusSchema.default('pending'),
    ssl_expires_at: z.string().datetime().optional(),
    last_checked_at: z.string().datetime().optional(),
    created_at: z.string().datetime(),
    verified_at: z.string().datetime().optional(),
});

export const WidgetSchema = z.object({
    id: z.string().uuid(),
    name: z.string().min(1).max(255),
    display_name: z.string().min(1).max(255),
    description: z.string().optional(),
    category: z.string().max(100).default('custom'),
    component_config: z.record(z.any()),
    external_api_config: z.record(z.any()).default({}),
    is_public: z.boolean().default(true),
    is_approved: z.boolean().default(false),
    version: z.string().max(20).default('1.0.0'),
    icon_url: z.string().url().optional(),
    documentation_url: z.string().url().optional(),
    created_by: z.string().uuid().optional(),
    created_at: z.string().datetime(),
    updated_at: z.string().datetime(),
});

export const SiteBuildSchema = z.object({
    id: z.string().uuid(),
    site_id: z.string().uuid(),
    build_type: BuildTypeSchema,
    status: z.enum(['pending', 'building', 'success', 'error']).default('pending'),
    build_log: z.string().optional(),
    assets_url: z.string().url().optional(),
    error_message: z.string().optional(),
    build_duration_ms: z.number().int().positive().optional(),
    started_at: z.string().datetime(),
    completed_at: z.string().datetime().optional(),
});

export const AssetSchema = z.object({
    id: z.string().uuid(),
    tenant_id: z.string().uuid(),
    site_id: z.string().uuid(),
    filename: z.string().min(1).max(500),
    original_filename: z.string().min(1).max(500),
    mime_type: z.string().max(100),
    file_size: z.number().int().positive(),
    storage_path: z.string(),
    cdn_url: z.string().url().optional(),
    alt_text: z.string().optional(),
    is_optimized: z.boolean().default(false),
    created_at: z.string().datetime(),
    updated_at: z.string().datetime(),
});

export const PageAnalyticsSchema = z.object({
    id: z.string().uuid(),
    page_id: z.string().uuid(),
    metric_type: MetricTypeSchema,
    metric_value: z.number().min(0),
    recorded_at: z.string().datetime(),
    user_agent: z.string().optional(),
    ip_address: z.string().ip().optional(),
    referrer: z.string().url().optional(),
});

// ============================================================================
// CREATE/UPDATE INPUT SCHEMAS (without auto-generated fields)
// ============================================================================

export const CreateTenantSchema = TenantSchema.omit({
    id: true,
    created_at: true,
    updated_at: true,
});

export const UpdateTenantSchema = CreateTenantSchema.partial().omit({
    slug: true, // Slug shouldn't be changed after creation
});

export const CreateUserSchema = UserSchema.omit({
    id: true,
    created_at: true,
    updated_at: true,
});

export const UpdateUserSchema = CreateUserSchema.partial().omit({
    tenant_id: true, // Tenant shouldn't be changed
});

export const CreateContentSchema = ContentSchema.omit({
    id: true,
    created_at: true,
    updated_at: true,
    published_at: true,
});

export const UpdateContentSchema = CreateContentSchema.partial().omit({
    tenant_id: true,
    author_id: true,
});

export const CreateSiteSchema = SiteSchema.omit({
    id: true,
    created_at: true,
    updated_at: true,
});

export const UpdateSiteSchema = CreateSiteSchema.partial().omit({
    tenant_id: true,
});

export const CreatePageSchema = PageSchema.omit({
    id: true,
    created_at: true,
    updated_at: true,
    published_at: true,
});

export const UpdatePageSchema = CreatePageSchema.partial().omit({
    site_id: true,
});

export const CreateTemplateSchema = TemplateSchema.omit({
    id: true,
    created_at: true,
    updated_at: true,
});

export const UpdateTemplateSchema = CreateTemplateSchema.partial().omit({
    tenant_id: true,
});

export const CreateAssetSchema = AssetSchema.omit({
    id: true,
    created_at: true,
    updated_at: true,
});

export const CreateDomainSchema = DomainSchema.omit({
    id: true,
    created_at: true,
    verified_at: true,
    last_checked_at: true,
});

export const CreateWidgetSchema = WidgetSchema.omit({
    id: true,
    created_at: true,
    updated_at: true,
});

// ============================================================================
// API RESPONSE SCHEMAS
// ============================================================================

export const ApiResponseSchema = <T extends z.ZodTypeAny>(dataSchema: T) =>
    z.object({
        success: z.boolean(),
        data: dataSchema.optional(),
        error: z.string().optional(),
        request_id: z.string().uuid(),
    });

export const PaginatedResponseSchema = <T extends z.ZodTypeAny>(itemSchema: T) =>
    z.object({
        success: z.boolean(),
        data: z.object({
            items: z.array(itemSchema),
            total: z.number().int().min(0),
            page: z.number().int().min(1),
            per_page: z.number().int().min(1),
            total_pages: z.number().int().min(0),
        }),
        error: z.string().optional(),
        request_id: z.string().uuid(),
    });

// ============================================================================
// QUERY PARAMETER SCHEMAS
// ============================================================================

export const PaginationQuerySchema = z.object({
    page: z.coerce.number().int().min(1).default(1),
    per_page: z.coerce.number().int().min(1).max(100).default(20),
    sort_by: z.string().optional(),
    sort_order: z.enum(['asc', 'desc']).default('desc'),
});

export const ContentQuerySchema = PaginationQuerySchema.extend({
    status: ContentStatusSchema.optional(),
    author_id: z.string().uuid().optional(),
    search: z.string().optional(),
});

export const SiteQuerySchema = PaginationQuerySchema.extend({
    is_published: z.coerce.boolean().optional(),
    template_id: z.string().uuid().optional(),
});

export const PageQuerySchema = PaginationQuerySchema.extend({
    site_id: z.string().uuid().optional(),
    is_published: z.coerce.boolean().optional(),
});

export const TemplateQuerySchema = PaginationQuerySchema.extend({
    category: z.string().optional(),
    is_public: z.coerce.boolean().optional(),
});

export const AssetQuerySchema = PaginationQuerySchema.extend({
    site_id: z.string().uuid().optional(),
    mime_type: z.string().optional(),
});

// ============================================================================
// AUTHENTICATION & SECURITY SCHEMAS
// ============================================================================

export const LoginRequestSchema = z.object({
    email: z.string().email(),
    password: z.string().min(8),
});

export const LoginResponseSchema = z.object({
    token: z.string(),
    user: UserSchema,
    expires_at: z.string().datetime(),
});

export const JwtClaimsSchema = z.object({
    sub: z.string().uuid(), // user_id
    email: z.string().email(),
    name: z.string(),
    role: UserRoleSchema,
    tenant_id: z.string().uuid(),
    exp: z.number().int(),
    iat: z.number().int(),
    iss: z.string(),
});

export const SecurityStatusSchema = z.object({
    tenant_id: z.string().uuid(),
    tenant_name: z.string(),
    user_isolation_mode: z.enum(['collaborative', 'isolated', 'role_based']),
    total_users: z.number().int().min(0),
    admin_users: z.number().int().min(0),
    active_users: z.number().int().min(0),
    last_security_change: z.string().datetime(),
});

export const PermissionSchema = z.object({
    resource: z.string(),
    action: z.string(),
    tenant: z.string(),
});

export const UserPermissionsSchema = z.object({
    user_role: z.string(),
    tenant_id: z.string().uuid(),
    permissions: z.array(PermissionSchema),
});

// ============================================================================
// TYPE EXPORTS (inferred from schemas)
// ============================================================================

export type Tenant = z.infer<typeof TenantSchema>;
export type User = z.infer<typeof UserSchema>;
export type Content = z.infer<typeof ContentSchema>;
export type Template = z.infer<typeof TemplateSchema>;
export type TemplateVersion = z.infer<typeof TemplateVersionSchema>;
export type Site = z.infer<typeof SiteSchema>;
export type Page = z.infer<typeof PageSchema>;
export type Domain = z.infer<typeof DomainSchema>;
export type Widget = z.infer<typeof WidgetSchema>;
export type SiteBuild = z.infer<typeof SiteBuildSchema>;
export type Asset = z.infer<typeof AssetSchema>;
export type PageAnalytics = z.infer<typeof PageAnalyticsSchema>;

export type CreateTenant = z.infer<typeof CreateTenantSchema>;
export type UpdateTenant = z.infer<typeof UpdateTenantSchema>;
export type CreateUser = z.infer<typeof CreateUserSchema>;
export type UpdateUser = z.infer<typeof UpdateUserSchema>;
export type CreateContent = z.infer<typeof CreateContentSchema>;
export type UpdateContent = z.infer<typeof UpdateContentSchema>;
export type CreateSite = z.infer<typeof CreateSiteSchema>;
export type UpdateSite = z.infer<typeof UpdateSiteSchema>;
export type CreatePage = z.infer<typeof CreatePageSchema>;
export type UpdatePage = z.infer<typeof UpdatePageSchema>;
export type CreateTemplate = z.infer<typeof CreateTemplateSchema>;
export type UpdateTemplate = z.infer<typeof UpdateTemplateSchema>;
export type CreateAsset = z.infer<typeof CreateAssetSchema>;
export type CreateDomain = z.infer<typeof CreateDomainSchema>;
export type CreateWidget = z.infer<typeof CreateWidgetSchema>;

export type LoginRequest = z.infer<typeof LoginRequestSchema>;
export type LoginResponse = z.infer<typeof LoginResponseSchema>;
export type JwtClaims = z.infer<typeof JwtClaimsSchema>;
export type SecurityStatus = z.infer<typeof SecurityStatusSchema>;
export type Permission = z.infer<typeof PermissionSchema>;
export type UserPermissions = z.infer<typeof UserPermissionsSchema>;

export type UserRole = z.infer<typeof UserRoleSchema>;
export type ContentStatus = z.infer<typeof ContentStatusSchema>;
export type BuildStatus = z.infer<typeof BuildStatusSchema>;
export type BuildType = z.infer<typeof BuildTypeSchema>;
export type SslStatus = z.infer<typeof SslStatusSchema>;
export type VerificationMethod = z.infer<typeof VerificationMethodSchema>;
export type MetricType = z.infer<typeof MetricTypeSchema>;

// ============================================================================
// VALIDATION HELPERS
// ============================================================================

export const validateUuid = (value: string): boolean => {
    return z.string().uuid().safeParse(value).success;
};

export const validateEmail = (value: string): boolean => {
    return z.string().email().safeParse(value).success;
};

export const validateSlug = (value: string): boolean => {
    return z.string().regex(/^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$/).safeParse(value).success;
};

export const validateDomain = (value: string): boolean => {
    return z.string().regex(/^[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9]*\.[a-zA-Z]{2,}$/).safeParse(value).success;
};
