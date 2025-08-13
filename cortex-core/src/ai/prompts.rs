use super::AIContext;

pub struct PromptBuilder {
    system_prompt: String,
    user_prompt: String,
    context: Option<AIContext>,
}

impl PromptBuilder {
    pub fn new() -> Self {
        Self {
            system_prompt: Self::default_system_prompt(),
            user_prompt: String::new(),
            context: None,
        }
    }
    
    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = prompt;
        self
    }
    
    pub fn with_user_prompt(mut self, prompt: String) -> Self {
        self.user_prompt = prompt;
        self
    }
    
    pub fn with_context(mut self, context: AIContext) -> Self {
        self.context = Some(context);
        self
    }
    
    pub fn build(&self) -> String {
        let mut full_prompt = self.system_prompt.clone();
        
        if let Some(ctx) = &self.context {
            full_prompt.push_str("\n\n");
            full_prompt.push_str(&ctx.to_prompt_context());
        }
        
        full_prompt.push_str("\n\nUser request: ");
        full_prompt.push_str(&self.user_prompt);
        
        full_prompt
    }
    
    fn default_system_prompt() -> String {
        r#"You are an AI assistant integrated into Cortex, a terminal file manager.
Your role is to help users with file management tasks through natural language.

Guidelines:
1. Provide clear, actionable file operations
2. Always confirm before suggesting destructive operations
3. Use appropriate shell commands for the user's platform
4. Explain what each operation will do
5. Consider the current directory and selected files in your responses

When suggesting file operations, format them as clear commands that can be executed."#.to_string()
    }
    
    pub fn file_organization_prompt() -> String {
        r#"Analyze the files in the current directory and suggest an organized structure.
Consider file types, dates, and logical groupings.
Provide specific move/rename commands to reorganize the files."#.to_string()
    }
    
    pub fn bulk_rename_prompt(pattern: &str) -> String {
        format!(
            r#"Generate meaningful names for the selected files based on: {}
Provide a list of rename operations in the format:
old_name -> new_name"#,
            pattern
        )
    }
    
    pub fn search_prompt(query: &str) -> String {
        format!(
            r#"Search for files matching this description: {}
Consider file names, types, and potential content.
Return a list of relevant files with brief explanations."#,
            query
        )
    }
}