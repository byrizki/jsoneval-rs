namespace JsonEvalRs
{
    /// <summary>
    /// Return format for path-based methods
    /// </summary>
    public enum ReturnFormat
    {
        /// <summary>
        /// Nested object preserving the path hierarchy (default)
        /// Example: { "user": { "profile": { "name": "John" } } }
        /// </summary>
        Nested = 0,
        
        /// <summary>
        /// Flat object with dotted keys
        /// Example: { "user.profile.name": "John" }
        /// </summary>
        Flat = 1,
        
        /// <summary>
        /// Array of values in the order of requested paths
        /// Example: ["John"]
        /// </summary>
        Array = 2
    }
}
