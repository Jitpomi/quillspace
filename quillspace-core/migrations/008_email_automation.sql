-- Email Automation System
-- Handles automated email sequences for consultation workflow

-- Email types enum
CREATE TYPE email_type AS ENUM (
    'booking_confirmation',
    'pre_consultation_reminder', 
    'post_consultation_followup',
    'project_brief_reminder',
    'proposal_sent',
    'proposal_accepted',
    'project_kickoff',
    'project_update',
    'project_completion'
);

-- Email status enum
CREATE TYPE email_status AS ENUM (
    'pending',
    'sent',
    'failed',
    'cancelled'
);

-- Email jobs queue
CREATE TABLE email_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    booking_id UUID NOT NULL REFERENCES consultation_bookings(id) ON DELETE CASCADE,
    email_type email_type NOT NULL,
    recipient_email TEXT NOT NULL,
    template_variables JSONB NOT NULL DEFAULT '{}',
    scheduled_for TIMESTAMPTZ NOT NULL,
    sent_at TIMESTAMPTZ,
    status email_status NOT NULL DEFAULT 'pending',
    retry_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Email templates (for customizable templates)
CREATE TABLE email_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    email_type email_type NOT NULL,
    subject TEXT NOT NULL,
    html_content TEXT NOT NULL,
    text_content TEXT NOT NULL,
    variables TEXT[] NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Project management tables
CREATE TYPE project_status AS ENUM (
    'kickoff_scheduled',
    'in_progress',
    'design_review',
    'development_phase',
    'client_review',
    'revisions',
    'final_approval',
    'completed',
    'on_hold',
    'cancelled'
);

CREATE TYPE phase_status AS ENUM (
    'not_started',
    'in_progress', 
    'completed',
    'blocked'
);

CREATE TYPE deliverable_status AS ENUM (
    'not_started',
    'in_progress',
    'ready_for_review',
    'under_review',
    'approved',
    'needs_revision',
    'completed'
);

-- Main project table
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    consultation_booking_id UUID NOT NULL REFERENCES consultation_bookings(id) ON DELETE CASCADE,
    proposal_id UUID, -- References consultation_proposals(id)
    project_name TEXT NOT NULL,
    client_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    assigned_designer_id UUID REFERENCES users(id),
    assigned_developer_id UUID REFERENCES users(id),
    project_status project_status NOT NULL DEFAULT 'kickoff_scheduled',
    kickoff_date TIMESTAMPTZ NOT NULL,
    estimated_completion TIMESTAMPTZ NOT NULL,
    actual_completion TIMESTAMPTZ,
    
    -- Communication preferences
    primary_contact_email TEXT NOT NULL,
    preferred_meeting_times TEXT[] DEFAULT '{}',
    update_frequency TEXT DEFAULT 'weekly',
    slack_channel TEXT,
    phone_number TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Project phases
CREATE TABLE project_phases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    estimated_duration_days INTEGER NOT NULL,
    start_date TIMESTAMPTZ,
    completion_date TIMESTAMPTZ,
    status phase_status NOT NULL DEFAULT 'not_started',
    deliverables TEXT[] DEFAULT '{}',
    dependencies UUID[] DEFAULT '{}', -- Array of phase IDs
    phase_order INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Project deliverables
CREATE TABLE project_deliverables (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    phase_id UUID REFERENCES project_phases(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    description TEXT,
    deliverable_type TEXT NOT NULL,
    due_date TIMESTAMPTZ NOT NULL,
    status deliverable_status NOT NULL DEFAULT 'not_started',
    file_url TEXT,
    approval_required BOOLEAN NOT NULL DEFAULT FALSE,
    approved_at TIMESTAMPTZ,
    approved_by UUID REFERENCES users(id),
    feedback TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Client assets
CREATE TABLE client_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    asset_type TEXT NOT NULL,
    file_url TEXT,
    file_size BIGINT,
    mime_type TEXT,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    uploaded_by UUID REFERENCES users(id),
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Project updates/activity log
CREATE TABLE project_updates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    update_type TEXT NOT NULL, -- 'phase_completed', 'deliverable_submitted', 'feedback_received', etc.
    title TEXT NOT NULL,
    message TEXT,
    metadata JSONB DEFAULT '{}',
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_email_jobs_status_scheduled ON email_jobs(status, scheduled_for) WHERE status = 'pending';
CREATE INDEX idx_email_jobs_booking_id ON email_jobs(booking_id);
CREATE INDEX idx_email_jobs_retry_count ON email_jobs(retry_count) WHERE status = 'pending';

CREATE INDEX idx_email_templates_type_active ON email_templates(email_type, is_active) WHERE is_active = TRUE;

CREATE INDEX idx_projects_client_user ON projects(client_user_id);
CREATE INDEX idx_projects_status ON projects(project_status);
CREATE INDEX idx_projects_completion ON projects(estimated_completion);

CREATE INDEX idx_project_phases_project_order ON project_phases(project_id, phase_order);
CREATE INDEX idx_project_phases_status ON project_phases(status);

CREATE INDEX idx_project_deliverables_project ON project_deliverables(project_id);
CREATE INDEX idx_project_deliverables_due_date ON project_deliverables(due_date);
CREATE INDEX idx_project_deliverables_status ON project_deliverables(status);

CREATE INDEX idx_client_assets_project ON client_assets(project_id);
CREATE INDEX idx_project_updates_project_created ON project_updates(project_id, created_at);

-- Row Level Security
ALTER TABLE email_jobs ENABLE ROW LEVEL SECURITY;
ALTER TABLE email_templates ENABLE ROW LEVEL SECURITY;
ALTER TABLE projects ENABLE ROW LEVEL SECURITY;
ALTER TABLE project_phases ENABLE ROW LEVEL SECURITY;
ALTER TABLE project_deliverables ENABLE ROW LEVEL SECURITY;
ALTER TABLE client_assets ENABLE ROW LEVEL SECURITY;
ALTER TABLE project_updates ENABLE ROW LEVEL SECURITY;

-- RLS Policies for email_jobs
CREATE POLICY email_jobs_tenant_isolation ON email_jobs
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM consultation_bookings cb 
            WHERE cb.id = booking_id 
            AND cb.tenant_id = current_setting('app.current_tenant_id')::UUID
        )
    );

-- RLS Policies for email_templates (admin only)
CREATE POLICY email_templates_admin_only ON email_templates
    FOR ALL USING (current_setting('app.user_role') = 'admin');

-- RLS Policies for projects
CREATE POLICY projects_tenant_isolation ON projects
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM consultation_bookings cb 
            WHERE cb.id = consultation_booking_id 
            AND cb.tenant_id = current_setting('app.current_tenant_id')::UUID
        )
    );

CREATE POLICY projects_user_access ON projects
    FOR ALL USING (
        client_user_id = current_setting('app.current_user_id')::UUID 
        OR assigned_designer_id = current_setting('app.current_user_id')::UUID
        OR assigned_developer_id = current_setting('app.current_user_id')::UUID
        OR current_setting('app.user_role') = 'admin'
    );

-- Similar RLS policies for related tables
CREATE POLICY project_phases_access ON project_phases
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM projects p 
            WHERE p.id = project_id 
            AND (p.client_user_id = current_setting('app.current_user_id')::UUID 
                 OR p.assigned_designer_id = current_setting('app.current_user_id')::UUID
                 OR p.assigned_developer_id = current_setting('app.current_user_id')::UUID
                 OR current_setting('app.user_role') = 'admin')
        )
    );

CREATE POLICY project_deliverables_access ON project_deliverables
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM projects p 
            WHERE p.id = project_id 
            AND (p.client_user_id = current_setting('app.current_user_id')::UUID 
                 OR p.assigned_designer_id = current_setting('app.current_user_id')::UUID
                 OR p.assigned_developer_id = current_setting('app.current_user_id')::UUID
                 OR current_setting('app.user_role') = 'admin')
        )
    );

CREATE POLICY client_assets_access ON client_assets
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM projects p 
            WHERE p.id = project_id 
            AND (p.client_user_id = current_setting('app.current_user_id')::UUID 
                 OR p.assigned_designer_id = current_setting('app.current_user_id')::UUID
                 OR p.assigned_developer_id = current_setting('app.current_user_id')::UUID
                 OR current_setting('app.user_role') = 'admin')
        )
    );

CREATE POLICY project_updates_access ON project_updates
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM projects p 
            WHERE p.id = project_id 
            AND (p.client_user_id = current_setting('app.current_user_id')::UUID 
                 OR p.assigned_designer_id = current_setting('app.current_user_id')::UUID
                 OR p.assigned_developer_id = current_setting('app.current_user_id')::UUID
                 OR current_setting('app.user_role') = 'admin')
        )
    );

-- Update triggers
CREATE TRIGGER update_email_jobs_updated_at 
    BEFORE UPDATE ON email_jobs 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_email_templates_updated_at 
    BEFORE UPDATE ON email_templates 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_projects_updated_at 
    BEFORE UPDATE ON projects 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_project_phases_updated_at 
    BEFORE UPDATE ON project_phases 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_project_deliverables_updated_at 
    BEFORE UPDATE ON project_deliverables 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert default email templates
INSERT INTO email_templates (name, email_type, subject, html_content, text_content, variables) VALUES
(
    'Booking Confirmation',
    'booking_confirmation',
    '🎉 Your QuillSpace consultation is confirmed!',
    '<div>Your consultation has been confirmed. <a href="{{brief_url}}">Complete your project brief</a></div>',
    'Your consultation has been confirmed. Complete your project brief at {{brief_url}}',
    ARRAY['event_name', 'scheduled_at', 'brief_url', 'consultation_url']
),
(
    'Project Brief Reminder', 
    'project_brief_reminder',
    '📝 Complete your project brief for maximum consultation value',
    '<div>Please complete your project brief at <a href="{{brief_url}}">{{brief_url}}</a></div>',
    'Please complete your project brief at {{brief_url}}',
    ARRAY['event_name', 'scheduled_at', 'brief_url']
),
(
    'Pre-consultation Reminder',
    'pre_consultation_reminder', 
    '⏰ Your QuillSpace consultation is tomorrow!',
    '<div>Your consultation {{event_name}} is tomorrow. Please review the preparation checklist.</div>',
    'Your consultation {{event_name}} is tomorrow. Please review the preparation checklist.',
    ARRAY['event_name', 'scheduled_at', 'preparation_checklist', 'zoom_link']
);
