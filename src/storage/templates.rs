// Operation templates - saved operation configurations

use crate::operations::BitOperation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationTemplate {
    pub template_name: String,
    pub operation: BitOperation,
}

impl OperationTemplate {
    pub fn new(template_name: String, operation: BitOperation) -> Self {
        Self {
            template_name,
            operation,
        }
    }

    /// Get the icon for this template based on operation type
    pub fn icon(&self) -> &'static str {
        match &self.operation {
            BitOperation::LoadFile { .. } => "📂",
            BitOperation::TakeSkipSequence { .. } => "🔀",
            BitOperation::InvertBits { .. } => "🔄",
            BitOperation::MultiWorksheetLoad { .. } => "📑",
            BitOperation::TruncateBits { .. } => "✂",
            BitOperation::InterleaveBits { .. } => "🧩",
        }
    }

    /// Create an operation from this template
    pub fn instantiate(&self) -> BitOperation {
        self.operation.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateLibrary {
    pub templates: Vec<OperationTemplate>,
}

impl TemplateLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_template(&mut self, template: OperationTemplate) {
        self.templates.push(template);
    }

    pub fn remove_template(&mut self, index: usize) {
        if index < self.templates.len() {
            self.templates.remove(index);
        }
    }

    pub fn get_template(&self, index: usize) -> Option<&OperationTemplate> {
        self.templates.get(index)
    }

    /// Get default templates (common presets)
    pub fn with_defaults() -> Self {
        let mut library = Self::new();

        // Manchester decode preset
        library.add_template(OperationTemplate::new(
            "Manchester Decode (Take Every Other)".to_string(),
            BitOperation::TakeSkipSequence {
                name: "Manchester Decode".to_string(),
                sequence: crate::operations::OperationSequence::from_string("T1S1").unwrap(),
                enabled: true,
            },
        ));

        // Byte swap preset (for 2-byte words)
        library.add_template(OperationTemplate::new(
            "Byte Swap (16-bit words)".to_string(),
            BitOperation::InterleaveBits {
                name: "Byte Swap".to_string(),
                interleaver_type: crate::operations::InterleaverType::Block,
                block_config: Some(crate::operations::BlockInterleaverConfig {
                    block_size: 8,
                    depth: 2,
                    direction: crate::operations::InterleaverDirection::Deinterleave,
                }),
                convolutional_config: None,
                symbol_config: None,
                enabled: true,
            },
        ));

        // Invert all bits preset
        library.add_template(OperationTemplate::new(
            "Invert All Bits".to_string(),
            BitOperation::InvertBits {
                name: "Invert All".to_string(),
                enabled: true,
            },
        ));

        library
    }
}
