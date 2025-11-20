//! Capability registry scaffolding for the intent router.

use super::payload::IntentPayload;
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Callback executed before dispatching an intent.
pub type ValidationCallback = Arc<dyn Fn(&IntentPayload) -> Result<()> + Send + Sync>;

/// Declarative descriptor for router capabilities registered by modules.
#[derive(Clone)]
pub struct CapabilityDescriptor {
    pub descriptor_id: String,
    pub actions: Vec<String>,
    pub keywords: Vec<String>,
    pub required_params: Vec<String>,
    pub validation_callback: Option<ValidationCallback>,
    pub default_confirmation: ConfirmationRule,
    pub version: String,
}

impl CapabilityDescriptor {
    pub fn new(descriptor_id: impl Into<String>) -> Self {
        Self {
            descriptor_id: descriptor_id.into(),
            actions: Vec::new(),
            keywords: Vec::new(),
            required_params: Vec::new(),
            validation_callback: None,
            default_confirmation: ConfirmationRule::None,
            version: "1.0.0".into(),
        }
    }
}

/// Default confirmation policy for a capability.
#[derive(Clone)]
pub enum ConfirmationRule {
    None,
    ConfirmPhrase(String),
    Manifest,
}

/// Registry storing capability descriptors for intent routing.
pub struct CapabilityRegistry {
    descriptors: HashMap<String, CapabilityDescriptor>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            descriptors: HashMap::new(),
        }
    }

    pub fn register(&mut self, descriptor: CapabilityDescriptor) -> Result<()> {
        let key = descriptor.descriptor_id.clone();
        if self.descriptors.contains_key(&key) {
            bail!("Capability descriptor {} already exists", key);
        }
        self.descriptors.insert(key, descriptor);
        Ok(())
    }

    pub fn get(&self, descriptor_id: &str) -> Option<&CapabilityDescriptor> {
        self.descriptors.get(descriptor_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &CapabilityDescriptor> {
        self.descriptors.values()
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}
