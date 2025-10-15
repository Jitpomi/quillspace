use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::database::Database;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectKickoff {
    pub id: Uuid,
    pub consultation_booking_id: Uuid,
    pub proposal_id: Uuid,
    pub project_name: String,
    pub client_user_id: Uuid,
    pub assigned_designer_id: Option<Uuid>,
    pub assigned_developer_id: Option<Uuid>,
    pub project_status: ProjectStatus,
    pub kickoff_date: DateTime<Utc>,
    pub estimated_completion: DateTime<Utc>,
    pub project_phases: Vec<ProjectPhase>,
    pub deliverables: Vec<Deliverable>,
    pub client_assets: Vec<ClientAsset>,
    pub communication_preferences: CommunicationPreferences,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    KickoffScheduled,
    InProgress,
    DesignReview,
    DevelopmentPhase,
    ClientReview,
    Revisions,
    FinalApproval,
    Completed,
    OnHold,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectPhase {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub estimated_duration_days: i32,
    pub start_date: Option<DateTime<Utc>>,
    pub completion_date: Option<DateTime<Utc>>,
    pub status: PhaseStatus,
    pub deliverables: Vec<String>,
    pub dependencies: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
    NotStarted,
    InProgress,
    Completed,
    Blocked,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Deliverable {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub deliverable_type: DeliverableType,
    pub due_date: DateTime<Utc>,
    pub status: DeliverableStatus,
    pub file_url: Option<String>,
    pub approval_required: bool,
    pub approved_at: Option<DateTime<Utc>>,
    pub feedback: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliverableType {
    Wireframes,
    DesignMockups,
    ContentStrategy,
    DevelopmentMilestone,
    TestingSite,
    FinalWebsite,
    TrainingMaterials,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliverableStatus {
    NotStarted,
    InProgress,
    ReadyForReview,
    UnderReview,
    Approved,
    NeedsRevision,
    Completed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientAsset {
    pub id: Uuid,
    pub name: String,
    pub asset_type: AssetType,
    pub file_url: Option<String>,
    pub description: Option<String>,
    pub status: AssetStatus,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    AuthorPhoto,
    BookCover,
    Logo,
    BrandColors,
    ExistingContent,
    InspirationImages,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetStatus {
    Pending,
    Received,
    Approved,
    NeedsReplacement,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommunicationPreferences {
    pub primary_contact_email: String,
    pub preferred_meeting_times: Vec<String>,
    pub update_frequency: UpdateFrequency,
    pub slack_channel: Option<String>,
    pub phone_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateFrequency {
    Daily,
    Weekly,
    BiWeekly,
    AsNeeded,
}

pub struct ProjectKickoffService {
    db: Database,
}

impl ProjectKickoffService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Initialize project kickoff after proposal acceptance
    pub async fn initialize_project(&self, proposal_id: Uuid) -> Result<ProjectKickoff> {
        // Get proposal and consultation details
        let (proposal, consultation) = self.get_proposal_details(proposal_id).await?;
        
        // Create project phases based on proposal scope
        let phases = self.create_project_phases(&proposal).await?;
        
        // Create initial deliverables
        let deliverables = self.create_initial_deliverables(&proposal, &phases).await?;
        
        let project = ProjectKickoff {
            id: Uuid::new_v4(),
            consultation_booking_id: consultation.id,
            proposal_id,
            project_name: proposal.title.clone(),
            client_user_id: consultation.user_id,
            assigned_designer_id: None, // Will be assigned later
            assigned_developer_id: None,
            project_status: ProjectStatus::KickoffScheduled,
            kickoff_date: Utc::now() + chrono::Duration::days(2), // Schedule kickoff in 2 days
            estimated_completion: Utc::now() + chrono::Duration::weeks(proposal.timeline_weeks as i64),
            project_phases: phases,
            deliverables,
            client_assets: Vec::new(),
            communication_preferences: CommunicationPreferences {
                primary_contact_email: consultation.guest_email.clone(),
                preferred_meeting_times: vec!["Morning".to_string(), "Afternoon".to_string()],
                update_frequency: UpdateFrequency::Weekly,
                slack_channel: None,
                phone_number: None,
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Save to database
        self.create_project_record(&project).await?;
        
        // Trigger kickoff workflow
        self.trigger_kickoff_workflow(&project).await?;
        
        Ok(project)
    }

    /// Create project phases based on proposal scope
    async fn create_project_phases(&self, proposal: &ProposalDetails) -> Result<Vec<ProjectPhase>> {
        let mut phases = Vec::new();
        
        // Phase 1: Discovery & Planning
        phases.push(ProjectPhase {
            id: Uuid::new_v4(),
            name: "Discovery & Planning".to_string(),
            description: "Gather requirements, create project plan, and set up infrastructure".to_string(),
            estimated_duration_days: 3,
            start_date: None,
            completion_date: None,
            status: PhaseStatus::NotStarted,
            deliverables: vec![
                "Project kickoff meeting".to_string(),
                "Content strategy document".to_string(),
                "Technical specification".to_string(),
            ],
            dependencies: Vec::new(),
        });

        // Phase 2: Design
        phases.push(ProjectPhase {
            id: Uuid::new_v4(),
            name: "Design & Wireframing".to_string(),
            description: "Create wireframes, design mockups, and establish visual identity".to_string(),
            estimated_duration_days: 7,
            start_date: None,
            completion_date: None,
            status: PhaseStatus::NotStarted,
            deliverables: vec![
                "Site wireframes".to_string(),
                "Visual design mockups".to_string(),
                "Style guide".to_string(),
            ],
            dependencies: vec![phases[0].id],
        });

        // Phase 3: Development
        phases.push(ProjectPhase {
            id: Uuid::new_v4(),
            name: "Development".to_string(),
            description: "Build the website using QuillSpace's platform".to_string(),
            estimated_duration_days: 10,
            start_date: None,
            completion_date: None,
            status: PhaseStatus::NotStarted,
            deliverables: vec![
                "Development environment setup".to_string(),
                "Core pages implementation".to_string(),
                "Content integration".to_string(),
                "Testing and optimization".to_string(),
            ],
            dependencies: vec![phases[1].id],
        });

        // Phase 4: Review & Launch
        phases.push(ProjectPhase {
            id: Uuid::new_v4(),
            name: "Review & Launch".to_string(),
            description: "Client review, revisions, and website launch".to_string(),
            estimated_duration_days: 5,
            start_date: None,
            completion_date: None,
            status: PhaseStatus::NotStarted,
            deliverables: vec![
                "Client review session".to_string(),
                "Revisions implementation".to_string(),
                "Website launch".to_string(),
                "Training materials".to_string(),
            ],
            dependencies: vec![phases[2].id],
        });

        Ok(phases)
    }

    /// Create initial deliverables
    async fn create_initial_deliverables(&self, proposal: &ProposalDetails, phases: &[ProjectPhase]) -> Result<Vec<Deliverable>> {
        let mut deliverables = Vec::new();
        let base_date = Utc::now();

        // Discovery deliverables
        deliverables.push(Deliverable {
            id: Uuid::new_v4(),
            name: "Project Kickoff Meeting".to_string(),
            description: "Initial project kickoff call to align on goals and timeline".to_string(),
            deliverable_type: DeliverableType::ContentStrategy,
            due_date: base_date + chrono::Duration::days(2),
            status: DeliverableStatus::NotStarted,
            file_url: None,
            approval_required: false,
            approved_at: None,
            feedback: None,
        });

        // Design deliverables
        deliverables.push(Deliverable {
            id: Uuid::new_v4(),
            name: "Website Wireframes".to_string(),
            description: "Structural layout of all website pages".to_string(),
            deliverable_type: DeliverableType::Wireframes,
            due_date: base_date + chrono::Duration::days(7),
            status: DeliverableStatus::NotStarted,
            file_url: None,
            approval_required: true,
            approved_at: None,
            feedback: None,
        });

        deliverables.push(Deliverable {
            id: Uuid::new_v4(),
            name: "Visual Design Mockups".to_string(),
            description: "High-fidelity design mockups of key pages".to_string(),
            deliverable_type: DeliverableType::DesignMockups,
            due_date: base_date + chrono::Duration::days(12),
            status: DeliverableStatus::NotStarted,
            file_url: None,
            approval_required: true,
            approved_at: None,
            feedback: None,
        });

        // Development deliverables
        deliverables.push(Deliverable {
            id: Uuid::new_v4(),
            name: "Testing Website".to_string(),
            description: "Fully functional website on staging environment".to_string(),
            deliverable_type: DeliverableType::TestingSite,
            due_date: base_date + chrono::Duration::days(20),
            status: DeliverableStatus::NotStarted,
            file_url: None,
            approval_required: true,
            approved_at: None,
            feedback: None,
        });

        // Final deliverable
        deliverables.push(Deliverable {
            id: Uuid::new_v4(),
            name: "Live Website".to_string(),
            description: "Final website launched and live".to_string(),
            deliverable_type: DeliverableType::FinalWebsite,
            due_date: base_date + chrono::Duration::days(25),
            status: DeliverableStatus::NotStarted,
            file_url: None,
            approval_required: true,
            approved_at: None,
            feedback: None,
        });

        Ok(deliverables)
    }

    /// Trigger kickoff workflow
    async fn trigger_kickoff_workflow(&self, project: &ProjectKickoff) -> Result<()> {
        // 1. Send kickoff email to client
        self.send_kickoff_email(project).await?;
        
        // 2. Create project in QuillSpace (using existing site creation)
        self.create_quillspace_site(project).await?;
        
        // 3. Set up project management workspace
        self.setup_project_workspace(project).await?;
        
        // 4. Assign team members
        self.assign_team_members(project.id).await?;
        
        // 5. Schedule kickoff meeting
        self.schedule_kickoff_meeting(project).await?;
        
        Ok(())
    }

    /// Create QuillSpace site for the project
    async fn create_quillspace_site(&self, project: &ProjectKickoff) -> Result<()> {
        // This would integrate with the existing site creation system
        // Create a new site in the client's tenant with project-specific settings
        
        let site_data = serde_json::json!({
            "name": project.project_name,
            "description": format!("Author website project started {}", project.kickoff_date),
            "project_id": project.id,
            "template_type": "author_website",
            "status": "in_development"
        });

        // TODO: Call existing site creation service
        tracing::info!("Creating QuillSpace site for project: {}", project.id);
        
        Ok(())
    }

    /// Set up project workspace
    async fn setup_project_workspace(&self, project: &ProjectKickoff) -> Result<()> {
        // Create project dashboard, file sharing, communication channels
        tracing::info!("Setting up project workspace for: {}", project.id);
        Ok(())
    }

    /// Assign team members based on project requirements
    async fn assign_team_members(&self, project_id: Uuid) -> Result<()> {
        // Auto-assign available team members or create assignment tasks
        tracing::info!("Assigning team members for project: {}", project_id);
        Ok(())
    }

    /// Schedule kickoff meeting
    async fn schedule_kickoff_meeting(&self, project: &ProjectKickoff) -> Result<()> {
        // Integrate with calendar system to schedule kickoff meeting
        tracing::info!("Scheduling kickoff meeting for project: {}", project.id);
        Ok(())
    }

    /// Send kickoff email to client
    async fn send_kickoff_email(&self, project: &ProjectKickoff) -> Result<()> {
        // Send welcome email with project details, timeline, and next steps
        tracing::info!("Sending kickoff email for project: {}", project.id);
        Ok(())
    }

    /// Get project status dashboard
    pub async fn get_project_dashboard(&self, project_id: Uuid) -> Result<ProjectDashboard> {
        let project = self.get_project_details(project_id).await?;
        
        Ok(ProjectDashboard {
            project,
            progress_percentage: self.calculate_progress_percentage(project_id).await?,
            upcoming_deliverables: self.get_upcoming_deliverables(project_id).await?,
            recent_updates: self.get_recent_updates(project_id).await?,
            team_members: self.get_assigned_team_members(project_id).await?,
        })
    }

    // Helper methods for database operations
    async fn create_project_record(&self, project: &ProjectKickoff) -> Result<()> {
        // Implementation for saving project to database
        Ok(())
    }

    async fn get_proposal_details(&self, proposal_id: Uuid) -> Result<(ProposalDetails, ConsultationDetails)> {
        // Implementation for fetching proposal and consultation details
        Ok((
            ProposalDetails {
                title: "Author Website Project".to_string(),
                timeline_weeks: 4,
            },
            ConsultationDetails {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                guest_email: "client@example.com".to_string(),
            }
        ))
    }

    async fn get_project_details(&self, project_id: Uuid) -> Result<ProjectKickoff> {
        // Implementation for fetching project details
        todo!()
    }

    async fn calculate_progress_percentage(&self, project_id: Uuid) -> Result<f32> {
        // Calculate completion percentage based on deliverables
        Ok(25.0)
    }

    async fn get_upcoming_deliverables(&self, project_id: Uuid) -> Result<Vec<Deliverable>> {
        Ok(Vec::new())
    }

    async fn get_recent_updates(&self, project_id: Uuid) -> Result<Vec<ProjectUpdate>> {
        Ok(Vec::new())
    }

    async fn get_assigned_team_members(&self, project_id: Uuid) -> Result<Vec<TeamMember>> {
        Ok(Vec::new())
    }
}

// Supporting structs
#[derive(Debug)]
struct ProposalDetails {
    title: String,
    timeline_weeks: i32,
}

#[derive(Debug)]
struct ConsultationDetails {
    id: Uuid,
    user_id: Uuid,
    guest_email: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectDashboard {
    pub project: ProjectKickoff,
    pub progress_percentage: f32,
    pub upcoming_deliverables: Vec<Deliverable>,
    pub recent_updates: Vec<ProjectUpdate>,
    pub team_members: Vec<TeamMember>,
}

#[derive(Debug, Serialize)]
pub struct ProjectUpdate {
    pub id: Uuid,
    pub message: String,
    pub update_type: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Serialize)]
pub struct TeamMember {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub avatar_url: Option<String>,
}
