using System;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.DependencyInjection.Extensions;

namespace JsonEvalRs
{
    /// <summary>
    /// Extension methods for registering JsonEvalRs services in dependency injection
    /// </summary>
    public static class JsonEvalRsServiceCollectionExtensions
    {
        /// <summary>
        /// Adds the global ParsedSchemaCache as a singleton service
        /// </summary>
        /// <param name="services">The service collection</param>
        /// <returns>The service collection for chaining</returns>
        public static IServiceCollection AddJsonEvalRsCache(this IServiceCollection services)
        {
            services.TryAddSingleton<ParsedSchemaCache>(sp => ParsedSchemaCache.Global);
            return services;
        }

        /// <summary>
        /// Adds a new local ParsedSchemaCache instance as a singleton service
        /// </summary>
        /// <param name="services">The service collection</param>
        /// <returns>The service collection for chaining</returns>
        public static IServiceCollection AddJsonEvalRsLocalCache(this IServiceCollection services)
        {
            services.TryAddSingleton<ParsedSchemaCache>(sp => new ParsedSchemaCache());
            return services;
        }

        /// <summary>
        /// Adds the global ParsedSchemaCache and pre-loads schemas on startup
        /// </summary>
        /// <param name="services">The service collection</param>
        /// <param name="configure">Configuration action to pre-load schemas</param>
        /// <returns>The service collection for chaining</returns>
        public static IServiceCollection AddJsonEvalRsCache(
            this IServiceCollection services,
            Action<ParsedSchemaCache> configure)
        {
            if (configure == null)
                throw new ArgumentNullException(nameof(configure));

            services.AddJsonEvalRsCache();
            
            // Configure the cache on startup
            services.AddSingleton<IStartupCacheInitializer>(sp =>
            {
                var cache = sp.GetRequiredService<ParsedSchemaCache>();
                return new StartupCacheInitializer(cache, configure);
            });

            return services;
        }

        /// <summary>
        /// Adds a local ParsedSchemaCache and pre-loads schemas on startup
        /// </summary>
        /// <param name="services">The service collection</param>
        /// <param name="configure">Configuration action to pre-load schemas</param>
        /// <returns>The service collection for chaining</returns>
        public static IServiceCollection AddJsonEvalRsLocalCache(
            this IServiceCollection services,
            Action<ParsedSchemaCache> configure)
        {
            if (configure == null)
                throw new ArgumentNullException(nameof(configure));

            services.AddJsonEvalRsLocalCache();
            
            // Configure the cache on startup
            services.AddSingleton<IStartupCacheInitializer>(sp =>
            {
                var cache = sp.GetRequiredService<ParsedSchemaCache>();
                return new StartupCacheInitializer(cache, configure);
            });

            return services;
        }
    }

    /// <summary>
    /// Interface for cache initialization on startup
    /// </summary>
    public interface IStartupCacheInitializer
    {
        /// <summary>
        /// Initialize the cache with pre-loaded schemas
        /// </summary>
        void Initialize();
    }

    /// <summary>
    /// Internal implementation of cache initializer
    /// </summary>
    internal class StartupCacheInitializer : IStartupCacheInitializer
    {
        private readonly ParsedSchemaCache _cache;
        private readonly Action<ParsedSchemaCache> _configure;
        private bool _initialized;

        public StartupCacheInitializer(ParsedSchemaCache cache, Action<ParsedSchemaCache> configure)
        {
            _cache = cache ?? throw new ArgumentNullException(nameof(cache));
            _configure = configure ?? throw new ArgumentNullException(nameof(configure));
        }

        public void Initialize()
        {
            if (_initialized)
                return;

            _configure(_cache);
            _initialized = true;
        }
    }

#if NET6_0_OR_GREATER
    /// <summary>
    /// Extension methods for IApplicationBuilder to initialize the cache on startup
    /// Available only for .NET 6.0 and later with ASP.NET Core
    /// </summary>
    public static class JsonEvalRsApplicationBuilderExtensions
    {
        /// <summary>
        /// Initialize the ParsedSchemaCache with pre-loaded schemas
        /// Call this in Configure() method after services are built
        /// </summary>
        /// <param name="app">The application builder</param>
        /// <returns>The application builder for chaining</returns>
        public static Microsoft.AspNetCore.Builder.IApplicationBuilder UseJsonEvalRsCache(
            this Microsoft.AspNetCore.Builder.IApplicationBuilder app)
        {
            var initializer = app.ApplicationServices.GetService<IStartupCacheInitializer>();
            initializer?.Initialize();
            return app;
        }
    }
#endif
}
