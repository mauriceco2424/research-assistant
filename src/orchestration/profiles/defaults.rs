use chrono::Utc;

use super::model::{
    KnowledgeEntry, KnowledgeProfile, ProfileMetadata, ProfileScopeMode, RemoteInferenceMetadata,
    UserProfile, UserProfileFields, WorkProfile, WorkProfileFields, WritingProfile,
    WritingProfileFields,
};

pub fn default_user_profile() -> UserProfile {
    UserProfile {
        metadata: default_metadata("user-profile"),
        summary: Vec::new(),
        fields: UserProfileFields {
            name: String::new(),
            affiliations: Vec::new(),
            communication_style: Vec::new(),
            availability: None,
        },
        history: Vec::new(),
    }
}

pub fn default_work_profile() -> WorkProfile {
    WorkProfile {
        metadata: default_metadata("work-profile"),
        summary: Vec::new(),
        fields: WorkProfileFields {
            active_projects: Vec::new(),
            milestones: Vec::new(),
            preferred_tools: Vec::new(),
            focus_statement: None,
            risks: Vec::new(),
        },
        history: Vec::new(),
    }
}

pub fn default_writing_profile() -> WritingProfile {
    WritingProfile {
        metadata: default_metadata("writing-profile"),
        summary: Vec::new(),
        fields: WritingProfileFields {
            tone_descriptors: Vec::new(),
            structure_preferences: Vec::new(),
            style_examples: Vec::new(),
            remote_inference_metadata: RemoteInferenceMetadata::default(),
        },
        history: Vec::new(),
    }
}

pub fn default_knowledge_profile() -> KnowledgeProfile {
    KnowledgeProfile {
        metadata: default_metadata("knowledge-profile"),
        summary: Vec::new(),
        entries: Vec::<KnowledgeEntry>::new(),
        history: Vec::new(),
    }
}

fn default_metadata(id: &str) -> ProfileMetadata {
    let mut metadata = ProfileMetadata::new(id, ProfileScopeMode::ThisBase);
    metadata.last_updated = Utc::now();
    metadata
}
