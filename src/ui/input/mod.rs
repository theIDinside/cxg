/// For which context we are supposed to dispatch input handling logic for
pub enum KeyboardInputContext {
    InputBox,
    TextView,
    /// This state acts also as a fall back context. 
    /// If the current keyboard input context, does not recognize an input
    /// We will try and translate that input on a "global" level, otherwise
    /// we would have to set the input context = Application, at which point we've introduced a LOT of complexity for when we edit text
    /// Doing it this way instead, we always check the translation against the current context (which is never set to application)
    /// And if it can't translate, we try the Application context as a fallback
    Application
}