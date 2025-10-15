-- Consultation Bookings System
-- Tracks Calendly bookings and manages the consultation workflow

-- Booking status enum
CREATE TYPE booking_status AS ENUM (
    'scheduled',
    'completed', 
    'cancelled',
    'no_show',
    'rescheduled'
);

-- Main consultation bookings table
CREATE TABLE consultation_bookings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Calendly integration
    calendly_event_uuid TEXT NOT NULL UNIQUE,
    event_name TEXT NOT NULL,
    scheduled_at TIMESTAMPTZ NOT NULL,
    status booking_status NOT NULL DEFAULT 'scheduled',
    guest_email TEXT NOT NULL,
    
    -- Project information
    project_brief JSONB,
    consultation_notes TEXT,
    proposal_sent BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_consultation_bookings_tenant_id ON consultation_bookings(tenant_id);
CREATE INDEX idx_consultation_bookings_user_id ON consultation_bookings(user_id);
CREATE INDEX idx_consultation_bookings_calendly_uuid ON consultation_bookings(calendly_event_uuid);
CREATE INDEX idx_consultation_bookings_scheduled_at ON consultation_bookings(scheduled_at);
CREATE INDEX idx_consultation_bookings_status ON consultation_bookings(status);

-- Project brief forms (for collecting detailed requirements)
CREATE TABLE project_brief_forms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    booking_id UUID NOT NULL REFERENCES consultation_bookings(id) ON DELETE CASCADE,
    
    -- Basic project info
    project_name TEXT,
    project_type TEXT, -- 'new_website', 'redesign', 'maintenance'
    genre TEXT, -- Author's genre
    target_audience TEXT,
    
    -- Website requirements
    pages_needed TEXT[], -- Array of page types needed
    features_required TEXT[], -- Array of features
    design_preferences JSONB, -- Colors, style, inspiration
    content_status TEXT, -- 'ready', 'partial', 'needs_creation'
    
    -- Business details
    timeline TEXT, -- 'asap', '1_month', '3_months', 'flexible'
    budget_range TEXT, -- 'under_5k', '5k_10k', '10k_plus', 'discuss'
    existing_website TEXT, -- URL if exists
    
    -- Additional info
    special_requirements TEXT,
    questions_for_team TEXT,
    
    -- Completion tracking
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    completed_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_project_brief_forms_booking_id ON project_brief_forms(booking_id);
CREATE INDEX idx_project_brief_forms_completed ON project_brief_forms(completed);

-- Consultation preparation materials
CREATE TABLE consultation_materials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    booking_id UUID NOT NULL REFERENCES consultation_bookings(id) ON DELETE CASCADE,
    
    material_type TEXT NOT NULL, -- 'portfolio', 'case_study', 'pricing', 'checklist'
    title TEXT NOT NULL,
    content TEXT,
    file_url TEXT,
    
    -- Tracking
    viewed BOOLEAN NOT NULL DEFAULT FALSE,
    viewed_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_consultation_materials_booking_id ON consultation_materials(booking_id);
CREATE INDEX idx_consultation_materials_type ON consultation_materials(material_type);

-- Proposal tracking
CREATE TABLE consultation_proposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    booking_id UUID NOT NULL REFERENCES consultation_bookings(id) ON DELETE CASCADE,
    
    -- Proposal details
    title TEXT NOT NULL,
    description TEXT,
    scope_of_work JSONB, -- Detailed breakdown
    timeline_weeks INTEGER,
    total_cost DECIMAL(10,2),
    
    -- Package options
    packages JSONB, -- Array of different package options
    
    -- Status tracking
    status TEXT NOT NULL DEFAULT 'draft', -- 'draft', 'sent', 'accepted', 'rejected', 'negotiating'
    sent_at TIMESTAMPTZ,
    responded_at TIMESTAMPTZ,
    
    -- Contract info
    contract_signed BOOLEAN NOT NULL DEFAULT FALSE,
    contract_signed_at TIMESTAMPTZ,
    deposit_paid BOOLEAN NOT NULL DEFAULT FALSE,
    deposit_amount DECIMAL(10,2),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_consultation_proposals_booking_id ON consultation_proposals(booking_id);
CREATE INDEX idx_consultation_proposals_status ON consultation_proposals(status);

-- Row Level Security
ALTER TABLE consultation_bookings ENABLE ROW LEVEL SECURITY;
ALTER TABLE project_brief_forms ENABLE ROW LEVEL SECURITY;
ALTER TABLE consultation_materials ENABLE ROW LEVEL SECURITY;
ALTER TABLE consultation_proposals ENABLE ROW LEVEL SECURITY;

-- RLS Policies for consultation_bookings
CREATE POLICY consultation_bookings_tenant_isolation ON consultation_bookings
    FOR ALL USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

CREATE POLICY consultation_bookings_user_access ON consultation_bookings
    FOR ALL USING (
        user_id = current_setting('app.current_user_id')::UUID 
        OR current_setting('app.user_role') = 'admin'
    );

-- RLS Policies for project_brief_forms
CREATE POLICY project_brief_forms_tenant_isolation ON project_brief_forms
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM consultation_bookings cb 
            WHERE cb.id = booking_id 
            AND cb.tenant_id = current_setting('app.current_tenant_id')::UUID
        )
    );

CREATE POLICY project_brief_forms_user_access ON project_brief_forms
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM consultation_bookings cb 
            WHERE cb.id = booking_id 
            AND (cb.user_id = current_setting('app.current_user_id')::UUID 
                 OR current_setting('app.user_role') = 'admin')
        )
    );

-- Similar RLS policies for other tables
CREATE POLICY consultation_materials_tenant_isolation ON consultation_materials
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM consultation_bookings cb 
            WHERE cb.id = booking_id 
            AND cb.tenant_id = current_setting('app.current_tenant_id')::UUID
        )
    );

CREATE POLICY consultation_proposals_tenant_isolation ON consultation_proposals
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM consultation_bookings cb 
            WHERE cb.id = booking_id 
            AND cb.tenant_id = current_setting('app.current_tenant_id')::UUID
        )
    );

-- Update trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_consultation_bookings_updated_at 
    BEFORE UPDATE ON consultation_bookings 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_project_brief_forms_updated_at 
    BEFORE UPDATE ON project_brief_forms 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_consultation_proposals_updated_at 
    BEFORE UPDATE ON consultation_proposals 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
