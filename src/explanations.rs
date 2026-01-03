//! Firefox preference explanations module
//!
//! This module contains a database of explanations for Firefox preferences.
//! Explanations are stored in a static HashMap for efficient lookup.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Static lookup table for Firefox preference explanations
///
/// Add new explanations here to include them in JSON array output.
///
/// # Writing Good Explanations
/// - Describe what the preference controls
/// - Mention the effects of different values (especially for booleans and enums)
/// - Note any potential side effects or breaking changes
/// - Keep explanations concise but informative
/// - Use clear, non-technical language when possible
///
/// # Example
/// ```rust
/// use std::collections::HashMap;
///
/// let explanations = HashMap::from([
///     ("javascript.enabled", "Master switch to enable or disable JavaScript..."),
///     ("browser.startup.homepage", "The default homepage that Firefox opens with..."),
/// ]);
///
/// assert!(explanations.contains_key("javascript.enabled"));
/// ```
static PREF_EXPLANATIONS: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

/// Get explanation for a preference key (returns static string reference)
///
/// Internal function that returns `Option<&'static str>` for efficient lookup.
///
/// # Arguments
/// * `key` - The preference key to look up (e.g., "javascript.enabled")
///
/// # Returns
/// `Some(&'static str)` containing the explanation, or `None` if not found
pub(crate) fn get_preference_explanation_static(key: &str) -> Option<&'static str> {
    PREF_EXPLANATIONS
        .get_or_init(|| {
            // Add more explanations here following the pattern above:
            // ("preference.name", "Explanation of what this preference does..."),
            //
            // Example format for different preference types:
            // - Boolean: "When true, X happens. When false, Y happens."
            // - Integer: "Sets the value for X. Valid values are 0-5 where..."
            // - String: "Specifies the X. Use Y format for..."
            HashMap::from([
                (
                    "javascript.enabled",
                    "Master switch to enable or disable JavaScript execution in Firefox. \
              When set to true, JavaScript can run in web pages. When false, JavaScript is \
              completely disabled, which may break many modern websites that rely on JavaScript \
              for functionality.",
                ),
                (
                    "privacy.trackingprotection.enabled",
                    "Enables Firefox's built-in tracking protection feature to block online trackers. \
              When set to true, Firefox blocks known tracking scripts and cookies from third-party \
              trackers, enhancing privacy while browsing. When false, tracking protection is disabled \
              and trackers may monitor your browsing activity across websites.",
                ),
                // Privacy and Tracking Protection
                (
                    "privacy.trackingprotection.emailtracking.enabled",
                    "Controls Firefox's email tracking protection feature that blocks tracking pixels \
              embedded in emails. When set to true, Firefox prevents email trackers from notifying \
              senders when you view emails containing hidden tracking images. This feature helps \
              prevent marketing and analytics companies from tracking your email viewing behavior.",
                ),
                (
                    "privacy.bounceTrackingProtection.mode",
                    "Controls the bounce tracking protection feature that detects and blocks redirect \
              trackers. Mode 1 (stateful) requires trackers to access storage before being flagged. \
              Mode 2 (stateless) is more aggressive and can block trackers based purely on redirect \
              behavior without requiring storage access. This feature runs in the background when \
              Enhanced Tracking Protection is set to Strict mode.",
                ),
                (
                    "privacy.fingerprintingProtection",
                    "Enables Firefox's fingerprinting protection to prevent websites from uniquely \
              identifying your browser based on its characteristics. When set to true, Firefox \
              obscures or randomizes information about your device, screen configuration, installed \
              fonts, and other attributes that can be used to create a digital fingerprint for \
              tracking purposes across different browsing sessions.",
                ),
                (
                    "privacy.globalprivacycontrol.enabled",
                    "Enables the Global Privacy Control (GPC) signal that tells websites not to sell \
              or share your personal data. When set to true, Firefox sends a universal opt-out signal \
              to every website you visit, requesting that they not sell or share your browsing data \
              per regulations like CCPA. Websites are not legally required to honor this signal, \
              though many do respect it.",
                ),
                (
                    "privacy.trackingprotection.socialtracking.enabled",
                    "Controls Firefox's social tracking protection feature that blocks social media \
              trackers embedded in websites. When set to true, Firefox prevents social media platforms \
              like Facebook, Twitter, and LinkedIn from tracking your browsing activity across \
              third-party websites through embedded buttons, widgets, and like buttons.",
                ),
                (
                    "browser.contentblocking.category",
                    "Controls Firefox's Enhanced Tracking Protection mode. Valid values are 'standard', \
              'strict', and 'custom'. Standard mode blocks basic trackers in private windows only. \
              Strict mode blocks all known trackers everywhere and may cause some websites to break. \
              Custom mode allows granular control over which tracking protections are enabled.",
                ),
                // Network and DNS
                (
                    "network.trr.mode",
                    "Controls DNS over HTTPS (DoH) mode in Firefox. Values: 0 = DoH disabled, \
              2 = Enabled with fallback (uses DoH but falls back to standard DNS if DoH fails), \
              3 = Strict mode (only uses DoH; if it fails, DNS resolution fails). Mode 2 is \
              recommended as it provides a good balance between privacy and reliability.",
                ),
                (
                    "network.trr.uri",
                    "Specifies the URL of the DNS over HTTPS (DoH) server to use for encrypted DNS \
              resolution. By default, Firefox uses Cloudflare's DoH service. This setting works \
              in conjunction with network.trr.mode to enable encrypted DNS queries that prevent \
              ISPs and network intermediaries from seeing which websites you visit.",
                ),
                (
                    "doh-rollout.disable-heuristics",
                    "Disables Firefox's automatic network-based detection for when to enable DNS over \
              HTTPS. When set to true, Firefox will not automatically enable or disable DoH based \
              on network conditions. This gives users and administrators more control over DoH \
              behavior rather than letting Firefox automatically decide based on network heuristics.",
                ),
                (
                    "network.http.referer.disallowCrossSiteRelaxingDefault.top_navigation",
                    "Prevents websites from relaxing referrer policies for cross-site top-level \
              navigation. When set to true, Firefox ignores permissive referrer policy directives \
              like 'unsafe-url' and 'no-referrer-when-downgrade' that would otherwise expose more \
              information about which site you came from. This helps prevent referrer information \
              leakage and enhances privacy.",
                ),
                // Security and HTTPS
                (
                    "dom.security.https_only_mode",
                    "Forces all connections to websites to use secure encrypted HTTPS instead of \
              unencrypted HTTP. When set to true, Firefox automatically attempts to upgrade HTTP \
              connections to HTTPS. If the HTTPS connection fails, Firefox may fall back to HTTP \
              after a timeout and displays a warning about the insecure connection.",
                ),
                (
                    "dom.security.https_only_mode_pbm",
                    "Same as dom.security.https_only_mode but applies only to Private Browsing \
              windows. When set to true, Firefox forces HTTPS-only mode in private browsing sessions, \
              providing stronger security protection for private browsing activities.",
                ),
                (
                    "privacy.query_stripping.enabled",
                    "Enables query parameter stripping to remove known tracking parameters from URLs \
              before navigating to them. When set to true, Firefox automatically strips tracking \
              parameters like utm_source, fbclid, and gclid from URLs, preventing websites and \
              third parties from tracking user behavior through link decoration.",
                ),
                (
                    "privacy.query_stripping.enabled.pbmode",
                    "Same as privacy.query_stripping.enabled but applies specifically to Private \
              Browsing mode. When set to true, Firefox strips tracking parameters from URLs in \
              private browsing sessions, providing enhanced privacy by removing navigational \
              tracking information.",
                ),
                (
                    "privacy.annotate_channels.strict_list.enabled",
                    "Controls whether Firefox uses a strict list to annotate and handle tracking \
              channels. When set to true, Firefox applies stricter rules for identifying and \
              handling tracking network connections. This is part of the state partitioning system \
              that helps separate website data to prevent cross-site tracking.",
                ),
                // Password Manager
                (
                    "signon.rememberSignons",
                    "Controls whether Firefox's built-in password manager offers to save and remember \
              website logins and passwords. When set to true, Firefox will prompt to save your \
              usernames and passwords when you log into websites. When set to false, Firefox will \
              not offer to save any passwords, effectively disabling the password manager functionality.",
                ),
                (
                    "signon.autofillForms",
                    "Controls whether Firefox's Password Manager automatically fills in usernames and \
              passwords on login forms. When set to true, Firefox automatically populates login forms \
              with saved credentials. When set to false, Firefox requires manual intervention to fill \
              login forms (you must click to fill), which can improve security by preventing automatic \
              credential exposure.",
                ),
                // Browser Startup and Session
                (
                    "browser.startup.page",
                    "Controls what Firefox displays when it starts up. Values: 0 = blank page, \
              1 = home page, 2 = restore previous session, 3 = resume session (similar to 2). \
              This setting determines whether Firefox shows your homepage, a blank page, or restores \
              your last browsing session with all open tabs when the browser starts.",
                ),
                // UI and Sidebar
                (
                    "sidebar.verticalTabs",
                    "Enables vertical tabs layout in Firefox's sidebar. When set to true, tabs are \
              displayed vertically in the sidebar instead of horizontally at the top of the browser \
              window. This makes it easier to see many tabs at once and is part of Firefox's sidebar \
              revamp introduced in recent versions.",
                ),
                (
                    "sidebar.revamp",
                    "Enables the redesigned sidebar with enhanced features and improved integration. \
              When set to true, the new sidebar provides access to multiple tools, vertical tabs, \
              and a more modern interface. This is part of Firefox's major UI update to the sidebar \
              system.",
                ),
                // URL Bar and Firefox Suggest
                (
                    "browser.urlbar.suggest.quicksuggest.all",
                    "Controls whether Firefox displays Firefox Suggest suggestions in the address \
              bar. When set to true, Firefox shows suggested websites, sponsored suggestions, and \
              quick results as you type in the address bar. When set to false, these suggestions are \
              disabled, providing a more traditional address bar experience.",
                ),
                // Telemetry and Data Reporting
                (
                    "datareporting.healthreport.uploadEnabled",
                    "Controls whether Firefox uploads health report and telemetry data to Mozilla. \
              When set to false, it disables the upload of technical data, crash reports, and \
              usage statistics. This setting corresponds to the 'Allow Firefox to send technical \
              and interaction data' checkbox in Firefox Settings.",
                ),
                (
                    "toolkit.telemetry.reportingpolicy.firstRun",
                    "Related to Firefox's data choices notification system. This preference is not \
              present until the first run of Firefox. After the first run, it helps control whether \
              telemetry reporting policy prompts are shown. Commonly set to false in privacy-focused \
              configurations to disable telemetry prompts.",
                ),
                // Customization
                (
                    "toolkit.legacyUserProfileCustomizations.stylesheets",
                    "Enables loading of custom CSS files (userChrome.css and userContent.css) from \
              your Firefox profile folder. When set to true, Firefox loads these custom stylesheets \
              on startup, allowing advanced users to customize Firefox's appearance. Must be set to \
              true for userChrome.css customizations to work. Requires Firefox restart to take effect.",
                ),
                // Remote Settings (services.settings.*)
                (
                    "services.settings.main.*.last_check",
                    "Remote Settings is a Mozilla system that delivers configuration data, blocklists, \
              and feature updates to Firefox without requiring a full browser update. The 'last_check' \
              preferences store Unix timestamps indicating when Firefox last successfully checked for \
              updates to specific Remote Settings collections. These include tracking protection lists, \
              certificate revocation lists, search configurations, and other security-critical data. \
              Firefox polls these regularly (typically every 24 hours) to ensure you have the latest \
              protection and configuration data.",
                ),
                (
                    "services.settings.last_update_seconds",
                    "Stores the Unix timestamp of the last successful Remote Settings update check. \
              This is used by Firefox to schedule periodic updates for Remote Settings collections, \
              ensuring that blocklists, certificate revocation data, and configuration updates remain \
              current without requiring manual browser updates.",
                ),
                (
                    "services.settings.last_etag",
                    "Stores the ETag (entity tag) from the last Remote Settings server response. \
              This HTTP header is used for cache validation and efficient polling - Firefox sends this \
              value to the server to check if any data has changed since the last check. If the ETag \
              matches, the server knows Firefox already has the latest data, saving bandwidth.",
                ),
                (
                    "services.settings.clock_skew_seconds",
                    "Measures the time difference between your local system clock and Mozilla's Remote \
              Settings server clock. A large negative value indicates your clock is behind the server. \
              Firefox uses this to detect and handle clock-related issues that could affect security \
              checks, certificate validation, and update scheduling.",
                ),
                // Privacy and Tracking Protection - Additional
                (
                    "privacy.clearOnShutdown_v2.formdata",
                    "Controls whether Firefox clears form and search history when the browser closes. \
              When set to true, Firefox automatically deletes information you've entered into web forms \
              and search bars when you close Firefox. This is part of the updated privacy clearing system \
              (v2) that provides more granular control over what data gets cleared on shutdown.",
                ),
                (
                    "privacy.trackingprotection.allow_list.hasMigratedCategoryPrefs",
                    "Internal migration preference indicating whether the user's tracking protection allow \
              list preferences have been migrated to the new Enhanced Tracking Protection category system. \
              When true, the user has been transitioned from the old tracking protection system to the \
              newer 'Standard/Strict/Custom' category-based system introduced in recent Firefox versions.",
                ),
                (
                    "privacy.trackingprotection.allow_list.baseline.enabled",
                    "Controls whether the baseline tracking protection allow list is enabled. When true, \
              websites on this special list are exempt from baseline tracking protections. This is used \
              to whitelist websites that would break under standard tracking protection rules, allowing \
              their trackers to function normally. Firefox manages this list through Remote Settings.",
                ),
                (
                    "privacy.trackingprotection.allow_list.convenience.enabled",
                    "Controls whether the convenience tracking protection allow list is enabled. When \
              true, certain third-party services that provide useful features (like embedded maps, videos, \
              or comment systems) are allowed to bypass tracking protections. This helps maintain website \
              functionality while still protecting privacy where possible.",
                ),
                (
                    "privacy.trackingprotection.allow_list.hasUserInteractedWithETPSettings",
                    "Internal preference tracking whether the user has ever manually modified their Enhanced \
              Tracking Protection (ETP) settings. When true, Firefox knows the user has customized their \
              tracking protection preferences, which may affect how Firefox handles future ETP updates or \
              prompts.",
                ),
                (
                    "privacy.bounceTrackingProtection.hasMigratedUserActivationData",
                    "Migration preference indicating whether bounce tracking protection data has been \
              upgraded from the old storage format to the new system. When true, Firefox has successfully \
              migrated user activation grants and bounce tracking state to the improved tracking protection \
              system.",
                ),
                (
                    "privacy.sanitize.clearOnShutdown.hasMigratedToNewPrefs3",
                    "Internal migration flag indicating whether privacy clearing preferences have been \
              migrated to the third version of the preference system. When true, the user's settings for \
              clearing data on shutdown have been updated to the latest preference format used by Firefox.",
                ),
                (
                    "privacy.purge_trackers.date_in_cookie_database",
                    "Internal timestamp tracking when Firefox last purged tracking cookies and site data \
              as part of the tracker purging feature. This is used by the 'Delete Cookies and Site Data' \
              functionality that automatically removes known tracking cookies and their associated site \
              data at regular intervals or on shutdown.",
                ),
                (
                    "privacy.purge_trackers.last_purge",
                    "Stores the Unix timestamp of the last time Firefox purged tracking cookies and site \
              data. Firefox automatically removes known tracking cookies and their associated storage \
              periodically to prevent long-term tracking. This preference records when the last purge \
              operation completed successfully.",
                ),
                (
                    "privacy.globalprivacycontrol.was_ever_enabled",
                    "Records whether the Global Privacy Control (GPC) signal has ever been enabled in \
              Firefox. When true, this indicates that at some point the user activated GPC, which sends \
              a universal opt-out signal to websites requesting that they not sell or share personal data. \
              Firefox uses this to track historical privacy preferences.",
                ),
                // Extensions and Web Compatibility
                (
                    "extensions.webcompat.enable_shims",
                    "Enables Web Compatibility shims, which are code snippets that fix broken websites \
              by emulating deprecated or missing web APIs. When true, Firefox applies these fixes to make \
              certain websites work properly. Shims are typically used when websites rely on old browser \
              features that Firefox no longer supports. Disabling this may break some websites but \
              increases browser security and reduces complexity.",
                ),
                (
                    "extensions.webcompat.perform_injections",
                    "Controls whether the Web Compatibility add-on injects custom scripts and stylesheets \
              into web pages to fix compatibility issues. When true, Firefox applies site-specific patches \
              to make problematic websites render and function correctly. These injections are managed by \
              Mozilla's web compatibility team and updated through Remote Settings.",
                ),
                (
                    "extensions.quarantinedDomains.list",
                    "Contains a comma-separated list of domains where Firefox restricts extension access for \
              security reasons. This feature, introduced in Firefox 115, prevents extensions from running \
              on sensitive websites like banking sites, payment processors, and other security-critical \
              domains. The list is maintained by Mozilla and updated through Remote Settings to protect \
              users from potentially malicious extensions.",
                ),
                (
                    "extensions.formautofill.addresses.enabled",
                    "Controls whether Firefox's autofill feature can automatically fill in address information \
              (street, city, postal code, country, etc.) on web forms. When true, Firefox will suggest and \
              fill address details that you've previously saved. When false, address autofill is disabled, \
              requiring manual entry each time.",
                ),
                (
                    "extensions.formautofill.creditCards.enabled",
                    "Controls whether Firefox's autofill feature can automatically fill in credit card \
              information on web payment forms. When true, Firefox will offer to fill saved credit card \
              details (numbers, expiration dates, cardholder names). When false, credit card autofill is \
              disabled for enhanced security. Note: This feature may be region-restricted.",
                ),
                (
                    "extensions.pictureinpicture.enable_picture_in_picture_overrides",
                    "Enables Picture-in-Picture overrides that force specific video players to work with \
              Firefox's PiP feature. When true, Firefox applies site-specific patches to make video players \
              from websites like YouTube, Netflix, and others work in Picture-in-Picture mode even if those \
              sites try to prevent it.",
                ),
                (
                    "extensions.pendingOperations",
                    "Internal flag indicating whether there are pending extension operations that need to \
              be processed, such as installs, uninstalls, updates, or migrations. When false, no extension \
              operations are pending. When true, Firefox is actively working on extension-related tasks and \
              may need to restart to complete them.",
                ),
                (
                    "extensions.databaseSchema",
                    "Version number of the extensions database schema. This tracks the format version of \
              Firefox's internal extension storage database. Firefox uses this to determine if migration \
              is needed when upgrading to a new version that changes how extension data is stored.",
                ),
                (
                    "extensions.lastAppVersion",
                    "Stores the version of Firefox that was running when extensions were last checked or \
              updated. This is used for compatibility checking - Firefox uses it to determine if extensions \
              need to be re-validated or updated after a browser upgrade.",
                ),
                (
                    "extensions.lastPlatformVersion",
                    "Similar to extensions.lastAppVersion but specifically tracks the platform version \
              (Mozilla platform) rather than the Firefox application version. Used internally for extension \
              compatibility checks.",
                ),
                // Gecko Media Plugins (GMP)
                (
                    "media.gmp-gmpopenh264.version",
                    "Stores the version number of the installed OpenH264 plugin. OpenH264 is Cisco's \
              open-source H.264 video codec implementation that Firefox automatically downloads to enable \
              H.264 video playback on websites that require this codec. This preference tracks which \
              version is currently installed.",
                ),
                (
                    "media.gmp-gmpopenh264.lastDownload",
                    "Unix timestamp indicating when Firefox last successfully downloaded or updated the \
              OpenH264 plugin. Firefox periodically checks for plugin updates to ensure you have the latest \
              security fixes and performance improvements.",
                ),
                (
                    "media.gmp-gmpopenh264.lastUpdate",
                    "Similar to lastDownload, stores the timestamp of the last OpenH264 plugin update. \
              Used by Firefox's plugin management system to track when updates were applied.",
                ),
                (
                    "media.gmp-gmpopenh264.lastInstallStart",
                    "Unix timestamp recording when the last OpenH264 plugin installation began. Used \
              internally for troubleshooting installation issues and tracking plugin download timing.",
                ),
                (
                    "media.gmp-gmpopenh264.hashValue",
                    "Cryptographic hash (SHA-256) of the OpenH264 plugin files. Firefox uses this to \
              verify the integrity and authenticity of the downloaded plugin, ensuring it hasn't been \
              tampered with or corrupted during download.",
                ),
                (
                    "media.gmp-gmpopenh264.abi",
                    "Stores the Application Binary Interface (ABI) identifier for the OpenH264 plugin, \
              such as 'x86_64-gcc3'. This ensures the downloaded plugin matches your system architecture \
              and compiler environment. Firefox uses this to download the correct plugin variant for your \
              system.",
                ),
                (
                    "media.gmp-manager.lastCheck",
                    "Unix timestamp of the last time Firefox checked for GMP (Gecko Media Plugin) updates. \
              Firefox regularly checks Mozilla's servers for updates to OpenH264, Widevine DRM, and other \
              media plugins to ensure you have the latest versions.",
                ),
                (
                    "media.gmp-manager.lastEmptyCheck",
                    "Timestamp of the last GMP update check that found no available updates. Firefox uses \
              this to avoid checking too frequently when no updates are available, reducing unnecessary \
              network requests.",
                ),
                (
                    "media.gmp.storage.version.observed",
                    "Internal preference tracking the version of the GMP storage system that Firefox has \
              observed. Used for migrating plugin data between different storage formats when Firefox \
              updates how it stores media plugin information.",
                ),
                // Nimbus and Experiments
                (
                    "nimbus.profileId",
                    "A unique identifier (UUID) assigned to your Firefox profile for Mozilla's Nimbus \
              experiment system. This ID is used to target you for specific feature experiments, A/B tests, \
              and staged rollouts. It allows Mozilla to track which experiment variant you should receive \
              while maintaining anonymity.",
                ),
                (
                    "nimbus.migrations.init-started",
                    "Internal counter tracking how many times Nimbus has attempted to initialize \
              migrations. Used to diagnose issues with the experiment system startup and to ensure \
              experiment data is properly loaded.",
                ),
                (
                    "nimbus.migrations.after-store-initialized",
                    "Internal migration counter for operations that run after the Nimbus experiment store \
              is initialized. This tracks the number of migration cycles completed to ensure experiment \
              data is properly upgraded between Firefox versions.",
                ),
                (
                    "nimbus.migrations.after-remote-settings-update",
                    "Migration counter for updates that occur after Remote Settings data is refreshed. \
              This ensures experiment configurations are properly synchronized when Firefox receives \
              updated experiment data from Mozilla's servers.",
                ),
                (
                    "app.normandy.user_id",
                    "A unique identifier for your Firefox profile used by the legacy Normandy experiment \
              system (predecessor to Nimbus). Normandy was Mozilla's previous system for deploying \
              experiments and preference rollouts. This ID is being phased out in favor of the Nimbus \
              system.",
                ),
                (
                    "app.normandy.first_run",
                    "Indicates whether Normandy has completed its first run initialization. When false, \
              Normandy's first-run logic has not yet executed. When true, the initial setup is complete. \
              This preference is being replaced by the Nimbus experiment system.",
                ),
                (
                    "app.normandy.migrationsApplied",
                    "Counter tracking how many Normandy data migrations have been applied. This records \
              the migration version number, helping Firefox track which updates to the experiment system \
              have been successfully applied to your profile.",
                ),
                // Browser Startup and Session
                (
                    "toolkit.startup.last_success",
                    "Stores the Unix timestamp of the last successful Firefox startup. Firefox uses this \
              to detect crashes and abnormal shutdowns. If the current startup time is much later than \
              this timestamp, Firefox knows the previous session may have ended abnormally and will offer \
              to restore your tabs and windows.",
                ),
                (
                    "browser.startup.couldRestoreSession.count",
                    "Internal counter that tracks whether Firefox has a previous browsing session available \
              to restore. A value of 1 or higher indicates there are saved tabs and windows from a \
              previous session. Firefox uses this to determine if it should offer session restoration \
              after a crash or restart.",
                ),
                (
                    "browser.startup.lastColdStartupCheck",
                    "Unix timestamp of the last time Firefox performed a cold startup check. A cold \
              startup occurs when Firefox is launched without any already-running instances. This metric \
              is used for performance monitoring and optimization of browser startup times.",
                ),
                (
                    "browser.sessionstore.upgradeBackup.latestBuildID",
                    "Stores the Firefox build ID (version number with timestamp) of the last session \
              backup created during an upgrade. When Firefox updates, it backs up your current session \
              so tabs can be restored if the upgrade causes issues. This preference identifies which \
              version created that backup.",
                ),
                (
                    "browser.migration.version",
                    "Records the version of the Firefox migration system that has been applied to your \
              profile. When Firefox updates and changes how data is stored or organized, it increments \
              this number to track which migrations have been run. This ensures your profile data is \
              properly upgraded across Firefox versions.",
                ),
                // DNS and Network
                (
                    "doh-rollout.home-region",
                    "Stores your detected home region code (e.g., 'US', 'CA', 'GB') for DNS over HTTPS \
              rollout purposes. Firefox automatically determines this based on your IP address and uses \
              it to decide when and how to enable DNS over HTTPS. Different regions may have different \
              DoH providers or rollout schedules based on local regulations and infrastructure.",
                ),
                (
                    "doh-rollout.doneFirstRun",
                    "Indicates whether Firefox has completed the first-run DNS over HTTPS rollout logic. \
              When true, Firefox has evaluated your network environment and decided whether to enable \
              DoH. When false, the first-run evaluation has not yet completed.",
                ),
                (
                    "browser.region.update.updated",
                    "Unix timestamp of the last time Firefox successfully updated its detected region \
              information. Firefox periodically checks which country/region you appear to be in based \
              on your IP address to provide localized search engines, certificates, and services.",
                ),
                (
                    "browser.search.region",
                    "Stores the country/region code that Firefox uses for search localization. This affects \
              which default search engine is offered and how search results are localized. Firefox updates \
              this based on your detected location, but it can also be set manually in preferences.",
                ),
                // Telemetry and Data Collection
                (
                    "toolkit.telemetry.cachedClientID",
                    "A unique identifier (UUID) assigned to your Firefox installation for telemetry purposes. \
              This ID is used by Mozilla to count unique users and analyze usage patterns while maintaining \
              anonymity. It's generated once and persisted across Firefox restarts unless telemetry is \
              disabled or you reset it.",
                ),
                (
                    "toolkit.telemetry.cachedProfileGroupID",
                    "A UUID that groups multiple Firefox profiles together for telemetry analysis. If you \
              have multiple Firefox profiles on the same system, they share this profile group ID, allowing \
              Mozilla to understand usage patterns across profiles while still maintaining privacy.",
                ),
                (
                    "datareporting.dau.cachedUsageProfileID",
                    "Another unique identifier for the Daily Active User (DAU) telemetry system. This ID \
              helps Mozilla track daily active users and understand engagement metrics while preserving \
              user anonymity.",
                ),
                (
                    "datareporting.dau.cachedUsageProfileGroupID",
                    "Group identifier for DAU telemetry that correlates multiple Firefox profiles. Similar \
              to the profile group ID but specifically for the Daily Active User measurement system.",
                ),
                (
                    "datareporting.policy.dataSubmissionPolicyNotifiedTime",
                    "Unix timestamp of when you were last presented with Firefox's data submission policy \
              notification. This records when Firefox showed you the prompt about sending technical and \
              interaction data to Mozilla. Firefox uses this to ensure policy acceptance is properly \
              tracked and to avoid showing the prompt too frequently.",
                ),
                (
                    "datareporting.policy.dataSubmissionPolicyAcceptedVersion",
                    "Records which version of Mozilla's data submission policy you have accepted. This \
              number increments when the policy changes significantly. Firefox uses this to track whether \
              you need to be presented with an updated policy notification.",
                ),
                (
                    "datareporting.usage.uploadEnabled",
                    "Master switch for Firefox usage data upload. When false, Firefox does not upload \
              usage statistics, technical performance data, or interaction metrics to Mozilla. This \
              corresponds to unchecking the 'Allow Firefox to send technical and interaction data' \
              option in Firefox Settings.",
                ),
                // Download and File Handling
                (
                    "browser.download.viewableInternally.typeWasRegistered.webp",
                    "Indicates whether Firefox has registered the WebP image format for internal viewing. \
              When true, Firefox can display WebP images natively without downloading them first. This \
              preference is set when Firefox initializes its MIME type handlers for different file formats.",
                ),
                (
                    "browser.download.viewableInternally.typeWasRegistered.avif",
                    "Indicates whether Firefox has registered the AVIF image format for internal viewing. \
              When true, Firefox can display AVIF images natively. AVIF is a modern image format that \
              provides better compression than JPEG or WebP. This preference tracks Firefox's capability \
              to handle this format.",
                ),
                // UI and Customization
                (
                    "browser.uiCustomization.state",
                    "JSON-encoded string that stores the complete state of Firefox's toolbar and UI \
              customization. This records which buttons are in which toolbars, their order, and which \
              items are in the overflow menu. When you customize Firefox's interface (move buttons, \
              add/remove items from toolbars), this preference is updated to remember your layout.",
                ),
                (
                    "browser.uiCustomization.horizontalTabstrip",
                    "JSON array listing the items displayed in the horizontal tab strip (when vertical \
              tabs are disabled). This controls which buttons and elements appear in the tab bar area, \
              such as the new tab button, all tabs button, and tab scroll buttons.",
                ),
                (
                    "sidebar.verticalTabs.dragToPinPromo.dismissed",
                    "Indicates whether you have dismissed the promotional tooltip about dragging tabs to \
              pin them when using vertical tabs. When true, you've seen and closed this prompt, and \
              Firefox won't show it again. When false, the prompt may still appear.",
                ),
                (
                    "sidebar.backupState",
                    "JSON-encoded string storing a backup of the sidebar state including position, width, \
              expanded/collapsed state, and currently active tool. Firefox uses this to restore your \
              sidebar configuration between browser sessions or after crashes.",
                ),
                (
                    "sidebar.main.tools",
                    "Comma-separated list of tools available in the Firefox sidebar. Typical values include \
              'syncedtabs', 'history', 'bookmarks', and more. This controls which sidebar panels are \
              accessible and in what order they appear.",
                ),
                (
                    "browser.toolbarbuttons.introduced.sidebar-button",
                    "Migration flag indicating when the sidebar toggle button was introduced to the \
              toolbar. When true, Firefox has added the sidebar button to your customization options, \
              typically occurring when Firefox updates and introduces new UI elements.",
                ),
                // New Tab Page and Activity Stream
                (
                    "browser.newtabpage.activity-stream.impressionId",
                    "A unique identifier (UUID) for tracking new tab page impressions and interactions. \
              This ID is used internally by Firefox to analyze how users interact with the new tab page, \
              including which features they use and how they engage with sponsored content.",
                ),
                (
                    "browser.newtabpage.activity-stream.showSponsored",
                    "Controls whether Firefox displays sponsored (paid) shortcuts and content on the new \
              tab page. When true, Firefox may show sponsored website shortcuts as part of the top sites \
              section. When false, only organic (non-sponsored) sites appear based on your browsing history.",
                ),
                (
                    "browser.newtabpage.activity-stream.showSponsoredTopSites",
                    "Specifically controls sponsored shortcuts in the Top Sites section of the new tab page. \
              When true, some of the frequently-visited sites shown may be sponsored placements. When false, \
              all top sites are based purely on your browsing history.",
                ),
                (
                    "browser.newtabpage.activity-stream.showSponsoredCheckboxes",
                    "Controls whether checkboxes appear next to sponsored shortcuts allowing you to dismiss \
              or hide specific sponsored content. When true, you can interact with sponsored items. When \
              false, sponsored items (if shown) cannot be individually dismissed.",
                ),
                (
                    "browser.newtabpage.activity-stream.showWeather",
                    "Controls whether Firefox displays a weather widget on the new tab page. When true, \
              the new tab page shows current weather conditions and a forecast for your location. When \
              false, the weather widget is hidden.",
                ),
                (
                    "browser.newtabpage.activity-stream.newtabWallpapers.wallpaper",
                    "Stores the ID of the currently selected wallpaper for the new tab page. Firefox can \
              display background images (wallpapers) on the new tab page, and this preference remembers \
              which one you've chosen. Examples include 'dark-blue', 'mountain', etc.",
                ),
                (
                    "browser.newtabpage.activity-stream.feeds.topsites",
                    "Controls whether the Top Sites section is enabled on the new tab page. When true, \
              Firefox displays your most frequently visited websites for quick access. When false, this \
              section is hidden, potentially reducing the usefulness of the new tab page.",
                ),
                (
                    "browser.newtabpage.activity-stream.feeds.section.topstories",
                    "Controls whether Firefox shows recommended articles and stories in the new tab page \
              (also known as Pocket or Mozilla Recommended content). When true, you'll see curated \
              articles. When false, this section is disabled.",
                ),
                (
                    "browser.newtabpage.storageVersion",
                    "Version number of the new tab page storage format. When Firefox updates how it stores \
              new tab page data and preferences, this number increments to trigger migration of old data \
              to the new format.",
                ),
                // URL Bar and Search
                (
                    "browser.urlbar.placeholderName",
                    "Stores the name of the default search engine to display in the address bar placeholder \
              text. For example, 'Google' or 'Bing'. This provides visual feedback about which search \
              engine will be used when you type search terms in the address bar.",
                ),
                (
                    "browser.urlbar.placeholderName.private",
                    "Same as browser.urlbar.placeholderName but for Private Browsing mode. Firefox may use \
              a different default search engine in private windows for enhanced privacy, such as DuckDuckGo \
              instead of Google.",
                ),
                (
                    "browser.urlbar.quicksuggest.migrationVersion",
                    "Tracks the migration version of Firefox Suggest data in your profile. Firefox Suggest \
              provides search suggestions as you type in the address bar. This preference records which \
              version of the suggestion data format you have, allowing Firefox to migrate data when the \
              system updates.",
                ),
                (
                    "browser.search.serpEventTelemetryCategorization.regionEnabled",
                    "Controls whether Firefox categorizes search engine results pages (SERPs) for telemetry \
              and analysis purposes. When true, Firefox records which search engines you use and categorizes \
              your search activity. This helps Mozilla understand search usage patterns.",
                ),
                // PDF and Other Features
                (
                    "pdfjs.migrationVersion",
                    "Stores the migration version for PDF.js (Firefox's built-in PDF viewer). When Firefox \
              updates the internal storage format for PDF viewer settings, caches, or form data, this \
              number increments to trigger data migration to the new format.",
                ),
                (
                    "pdfjs.enabledCache.state",
                    "Caches the enabled state of the PDF.js viewer. When true, PDF.js is active and will \
              display PDF files within the browser tab. This preference improves performance by avoiding \
              repeated checks of whether the PDF viewer should be enabled.",
                ),
                (
                    "print_printer",
                    "Stores the name of the default printer selected in Firefox's print dialog. This \
              remembers your printer choice between print sessions. The value 'Mozilla Save to PDF' \
              indicates Firefox's built-in PDF printing is selected.",
                ),
                // Identity and Sync
                (
                    "identity.fxaccounts.toolbar.syncSetup.panelAccessed",
                    "Indicates whether you have accessed the Firefox Sync setup panel from the toolbar. \
              When true, you've clicked the Firefox Account button in the toolbar at least once, which \
              may affect how Firefox displays sync-related prompts and notifications.",
                ),
                (
                    "services.sync.engine.addresses.available",
                    "Indicates whether the Firefox Sync engine for addresses (form autofill addresses) is \
              available for your account. When true, your saved addresses will sync across devices signed \
              into your Firefox Account.",
                ),
                (
                    "signon.firefoxRelay.feature",
                    "Controls the integration with Firefox Relay, Mozilla's email aliasing service that \
              helps protect your real email address. The value 'disabled' indicates this feature is turned \
              off. Other values may indicate the feature is enabled or set to specific modes.",
                ),
                (
                    "signon.generation.enabled",
                    "Controls whether Firefox's password generator feature is enabled. When true, Firefox \
              can suggest strong, unique passwords when you create new accounts or change passwords. When \
              false, this feature is disabled and you must manually create passwords.",
                ),
                (
                    "signon.management.page.breach-alerts.enabled",
                    "Controls whether Firefox shows alerts when your saved passwords have been compromised \
              in data breaches. When true, Firefox will notify you in the password manager if any of your \
              saved logins appear in known data breaches. When false, these alerts are disabled.",
                ),
                // Safe Browsing
                (
                    "browser.safebrowsing.provider.mozilla.lastupdatetime",
                    "Unix timestamp of when Firefox last updated Mozilla's Safe Browsing blocklist. Safe \
              Browsing protects you from malicious websites, phishing, and dangerous downloads. This \
              preference records when the blocklist was last successfully downloaded.",
                ),
                (
                    "browser.safebrowsing.provider.mozilla.nextupdatetime",
                    "Unix timestamp scheduled for the next Safe Browsing blocklist update. Firefox \
              automatically checks for updates to the malware and phishing site blocklist at regular \
              intervals, typically every 30 minutes to several hours depending on the provider.",
                ),
                // Other System Preferences
                (
                    "browser.shell.didSkipDefaultBrowserCheckOnFirstRun",
                    "Indicates whether Firefox skipped the default browser check on first run. When true, \
              Firefox did not prompt you to set it as the default browser during the first launch. This \
              may happen in managed environments or if a deployment policy suppresses the prompt.",
                ),
                (
                    "browser.shell.defaultBrowserCheckCount",
                    "Counter tracking how many times Firefox has checked if it's the default browser. \
              Firefox periodically asks if you want to set it as the default browser if it's not already. \
              This number increments with each check and is used to avoid showing the prompt too frequently.",
                ),
                (
                    "browser.shell.mostRecentDefaultPromptSeen",
                    "Unix timestamp of when you last saw the prompt to set Firefox as your default browser. \
              Firefox uses this to avoid showing the default browser prompt too frequently, ensuring you \
              aren't nagged.",
                ),
                (
                    "browser.policies.applied",
                    "Indicates whether enterprise policies have been applied to Firefox. When true, Firefox \
              is running under policy control (commonly in corporate or managed environments) and certain \
              settings may be enforced or restricted by system administrators.",
                ),
                (
                    "browser.aboutwelcome.didSeeFinalScreen",
                    "Indicates whether you have completed and dismissed the Firefox welcome/onboarding \
              experience. When true, you've reached the end of the welcome tour that appears on first \
              launch. Firefox uses this to avoid showing the welcome screens again.",
                ),
                (
                    "browser.laterrun.enabled",
                    "Controls Firefox's Later Run feature, which shows optional browser tips and feature \
              tours after you've been using Firefox for a while. When true, these delayed prompts may \
              appear to highlight useful features. When false, these prompts are disabled.",
                ),
                (
                    "browser.engagement.fxa-toolbar-menu-button.has-used",
                    "Tracks whether you have interacted with the Firefox Account toolbar button. When true, \
              you've clicked or used the Firefox Account/sync button in the toolbar at least once. This \
              may affect how Firefox displays sync-related features and prompts.",
                ),
                (
                    "distribution.nixos.bookmarksProcessed",
                    "Indicates whether Firefox has processed NixOS-specific bookmarks. The NixOS Linux \
              distribution includes default bookmarks for NixOS documentation and resources. When true, \
              these distribution-specific bookmarks have been added to your bookmark library.",
                ),
                (
                    "distribution.iniFile.exists.appversion",
                    "Records the Firefox version number when distribution customization files were detected. \
              Some Linux distributions and organizations customize Firefox with distribution.ini files. This \
              preference tracks which version had these customizations.",
                ),
                (
                    "distribution.iniFile.exists.value",
                    "Indicates whether a distribution customization file exists. When true, Firefox found \
              a distribution.ini file that customizes settings, bookmarks, or preferences for a specific \
              organization or Linux distribution.",
                ),
                (
                    "storage.vacuum.last.places.sqlite",
                    "Unix timestamp of the last time Firefox performed vacuum (cleanup and optimization) \
              on the places.sqlite database. This database stores your bookmarks and history. Vacuuming \
              reclaims space and improves performance as the database grows over time.",
                ),
                (
                    "storage.vacuum.last.index",
                    "Index or counter for the last vacuum operation performed on Firefox's storage \
              databases. Firefox periodically vacuums its SQLite databases to maintain performance as \
              you add bookmarks, history, and other data.",
                ),
                (
                    "places.database.lastMaintenance",
                    "Unix timestamp of the last time Firefox performed maintenance on the Places database \
              (bookmarks and history). This includes vacuuming, reindexing, and other optimizations to \
              keep Firefox responsive as your browsing history grows.",
                ),
                (
                    "browser.bookmarks.restore_default_bookmarks",
                    "Controls whether Firefox should restore the default set of bookmarks on next startup. \
              When true, Firefox will reset your bookmarks to the default set on the next launch. When \
              false (normal), your existing bookmarks are preserved. This is typically set during \
              troubleshooting or profile reset operations.",
                ),
                (
                    "browser.pagethumbnails.storage_version",
                    "Version number of the page thumbnails storage format. Firefox captures thumbnail \
              images of websites you visit for display on the new tab page and in other features. This \
              preference tracks the storage format version for migration purposes.",
                ),
                (
                    "browser.theme.dark.activetab",
                    "When true, Firefox is using a dark theme variant where the active tab has dark \
              styling applied. This preference helps Firefox track which theme variant is active so \
              it can properly apply visual updates and theme changes.",
                ),
                (
                    "browser.proton.toolbar.version",
                    "Stores the version number of the Proton toolbar design. Firefox's Proton design \
              system updated the browser interface with a modern look. This preference tracks which \
              iteration of the Proton toolbar design your profile is using.",
                ),
                (
                    "browser.colorway-builtin-themes-cleanup",
                    "Internal preference related to Firefox Colorway themes, which were special limited-time \
              color themes. This tracks cleanup and migration of colorway theme data after these themes \
              were discontinued or moved to the standard theme system.",
                ),
                (
                    "extensions.activeThemeID",
                    "Stores the ID of the currently active Firefox theme (e.g., 'default-theme@mozilla.org'). \
              This preference tracks which theme is applied, including built-in themes like Light, Dark, \
              System, or any installed custom themes.",
                ),
                (
                    "extensions.colorway-builtin-themes-cleanup",
                    "Internal migration preference for cleaning up legacy Colorway theme data. When \
              Firefox discontinued the Colorway themes (special limited-time color variants), this \
              preference tracked removal of old theme data from your profile.",
                ),
                (
                    "extensions.signatureCheckpoint",
                    "Internal counter tracking extension signature validation checkpoints. Firefox requires \
              extensions to be digitally signed by Mozilla for security. This preference tracks which \
              signature validation checks have been completed.",
                ),
                (
                    "extensions.blocklist.pingCountVersion",
                    "Counter related to the add-on blocklist ping mechanism. Firefox periodically checks \
              if installed extensions are known to be malicious or problematic. This preference tracks \
              the version or count of blocklist checks performed.",
                ),
                (
                    "extensions.getAddons.cache.lastUpdate",
                    "Unix timestamp of when Firefox last updated the add-ons cache. Firefox caches \
              information about available extensions, themes, and plugins from addons.mozilla.org to \
              speed up the add-ons manager and provide recommendations.",
                ),
                (
                    "extensions.getAddons.databaseSchema",
                    "Version number of the add-ons database schema. This tracks the format version of \
              Firefox's internal storage for information about available and installed extensions.",
                ),
                (
                    "browser.contextual-services.contextId",
                    "A unique identifier (UUID) for contextual services telemetry and personalization. \
              Firefox uses this to track contextual features like Firefox Suggest, Pocket recommendations, \
              and other location-based or activity-based services while maintaining user anonymity.",
                ),
                (
                    "browser.contextual-services.contextId.timestamp-in-seconds",
                    "Unix timestamp of when the contextual services ID was created. This records when \
              Firefox generated the unique identifier used for contextual features, helping to track \
              the age and rotation of this identifier.",
                ),
                (
                    "captchadetection.lastSubmission",
                    "Unix timestamp of when Firefox last submitted data related to CAPTCHA detection \
              and analysis. Firefox may analyze CAPTCHA challenges to improve user experience and \
              detect suspicious CAPTCHA behavior.",
                ),
                (
                    "dom.push.userAgentID",
                    "A unique identifier (UUID) for Firefox's Push API service. Web applications can use \
              the Push API to send you notifications even when the website isn't loaded. This ID uniquely \
              identifies your browser to push notification servers.",
                ),
                (
                    "network.cookie.CHIPS.lastMigrateDatabase",
                    "Version number of the Cookies Having Independent Partitioned State (CHIPS) database \
              migration. CHIPS is a privacy feature that partitions cookies by top-level site to prevent \
              cross-site tracking. This tracks which migration version has been applied.",
                ),
                (
                    "gecko.handlerService.defaultHandlersVersion",
                    "Version number of the default protocol handlers in Firefox. Protocol handlers allow \
              web applications to register as handlers for specific URL schemes (like mailto: links). \
              This preference tracks the version of the default handler configuration.",
                ),
                // Additional Remote Settings and Update Timers
                (
                    "app.update.lastUpdateTime.*",
                    "A series of preferences storing Unix timestamps of when various automatic update \
              and maintenance tasks last ran. These timers control when Firefox checks for add-on \
              updates, signature verification, Remote Settings polling, and other periodic background \
              tasks. Each timer ensures Firefox doesn't check too frequently, balancing between \
              staying current and conserving resources.",
                ),
                (
                    "media.gmp-manager.buildID",
                    "Stores the Firefox build ID (version timestamp) that the GMP manager is associated \
              with. This helps track which version of Firefox downloaded or updated media plugins, \
              allowing Firefox to determine if plugin re-registration or updates are needed after \
              browser upgrades.",
                ),
                (
                    "toolkit.telemetry.previousBuildID",
                    "Stores the Firefox build ID from the previous browser version. After Firefox updates, \
              this preference records what version you were running before. This is used for telemetry \
              analysis to understand upgrade patterns and to identify issues that occur specifically \
              during upgrades.",
                ),
                (
                    "extensions.lastAppBuildId",
                    "Stores the specific Firefox build identifier (version + timestamp) when extensions \
              were last validated or updated. This is used to track which browser version was running \
              during extension operations, helping to diagnose compatibility issues that arise after \
              Firefox updates.",
                ),
                // Additional System Preferences
                (
                    "toolkit.profiles.storeID",
                    "A unique identifier for your Firefox profile's storage location. This ID helps \
              Firefox distinguish between different profile directories and is used internally for \
              file system operations and profile management.",
                ),
                (
                    "idle.lastDailyNotification",
                    "Unix timestamp of the last time Firefox performed daily idle maintenance tasks. \
              Firefox runs certain maintenance operations (like database vacuuming, telemetry uploads, \
              and data cleanup) when the computer has been idle for a while. This preference tracks \
              when these daily tasks were last completed.",
                ),
                (
                    "browser.tabs.groups.smart.userEnabled",
                    "Controls whether you have manually enabled smart tab grouping features. When true, \
              you've activated Firefox's tab grouping system that organizes tabs based on browsing \
              context. When false, smart tab grouping is either disabled or not yet configured.",
                ),
                (
                    "browser.pageActions.persistedActions",
                    "JSON-encoded string storing which page actions (bookmark button, screenshot, etc.) \
              are pinned to the address bar. This remembers your preferences for which quick-action \
              buttons appear in the URL bar, allowing you to customize which page actions are \
              immediately accessible.",
                ),
                (
                    "privacy.sanitize.pending",
                    "JSON-encoded string tracking pending data sanitization operations that need to \
              be executed. When you close Firefox or trigger privacy clearing, this preference may \
              hold queued operations to clear specific types of data (history, cookies, cache, etc.) \
              to be processed when the browser is in a safe state to do so.",
                ),
                (
                    "privacy.trackingprotection.consentmanager.skip.pbmode.enabled",
                    "Controls whether Firefox skips tracking protection consent dialogs in Private \
              Browsing mode. When true, Firefox won't show additional consent prompts for tracking \
              protection features when in private windows, streamlining the private browsing experience.",
                ),
                (
                    "browser.startup.homepage_override.mstone",
                    "Stores the Firefox milestone (major version number) that triggered the homepage \
              override behavior. When Firefox updates to a new major version, it may display a special \
              'What's New' page once. This preference records which version you last saw, ensuring \
              the special homepage only appears once per major version.",
                ),
                (
                    "browser.startup.homepage_override.buildID",
                    "Stores the specific Firefox build ID (timestamp-based version) of the last homepage \
              override. Similar to mstone but more granular - it tracks the exact build rather than \
              just the major version. This helps Firefox decide whether to show update-related pages.",
                ),
                (
                    "dom.forms.autocomplete.formautofill",
                    "Controls whether Firefox's form autocomplete feature includes autofill suggestions. \
              When true, Firefox will suggest saved information (names, addresses, etc.) when filling \
              out forms. This is the master switch that enables or disables the entire form autofill \
              suggestion system.",
                ),
                (
                    "dom.security.https_only_mode_ever_enabled_pbm",
                    "Records whether HTTPS-Only mode has ever been enabled in Private Browsing mode. \
              When true, you've previously activated HTTPS-Only mode while browsing privately. This \
              helps Firefox remember your privacy preferences across sessions and may affect default \
              behavior in private windows.",
                ),
                (
                    "trailhead.firstrun.didSeeAboutWelcome",
                    "Indicates whether you have completed the Firefox Trailhead onboarding experience. \
              Trailhead is Firefox's new user experience that guides first-time users through setup \
              and feature highlights. When true, you've seen and completed the about:welcome tour.",
                ),
                (
                    "browser.termsofuse.prefMigrationCheck",
                    "Internal preference tracking whether you've acknowledged Firefox's updated terms \
              of use. When true, Firefox has confirmed you've seen or acknowledged the current terms. \
              This may be checked during browser updates to determine if legal terms need to be \
              presented again.",
                ),
                (
                    "extensions.webextensions.uuids",
                    "JSON-encoded object mapping extension IDs to their unique UUIDs. Every installed \
              extension is assigned a UUID for internal identification and security purposes. This \
              preference maintains the mapping between extension IDs (like 'webcompat@mozilla.org') \
              and their corresponding UUIDs.",
                ),
                // Specific Remote Settings Collections
                (
                    "services.settings.main.cookie-banner-rules-list.last_check",
                    "Stores the timestamp of the last check for Firefox's cookie banner handling rules. \
              This collection contains rules that automatically detect and handle cookie consent banners \
              on websites, clicking 'reject all' buttons or injecting opt-out cookies. The rules are \
              site-specific and tell Firefox how to interact with different cookie banner implementations.",
                ),
                (
                    "services.settings.main.normandy-recipes-capabilities.last_check",
                    "Records the last check for legacy Normandy recipe capabilities. Normandy was \
              Mozilla's previous experiment system (predecessor to Nimbus) that delivered preference \
              rollouts, A/B tests, and feature experiments. This collection defined what capabilities \
              the Normandy client supported.",
                ),
                (
                    "services.settings.main.third-party-cookie-blocking-exempt-urls.last_check",
                    "Timestamp for the last update of the exempt URLs list for third-party cookie blocking. \
              Firefox's Enhanced Tracking Protection can block third-party cookies, but some websites \
              require certain third-party cookies to function properly. This collection contains URLs \
              and domains that are exempt from third-party cookie blocking to prevent website breakage.",
                ),
                (
                    "services.settings.main.doh-providers.last_check",
                    "Records the last update check for DNS over HTTPS (DoH) provider configurations. \
              This collection contains the list of DoH servers that Firefox can use for encrypted DNS \
              resolution, including Cloudflare, NextDNS, and other providers. It includes provider \
              settings like endpoint URLs, filtering capabilities, and regional availability.",
                ),
                (
                    "services.settings.main.webcompat-interventions.last_check",
                    "Timestamp for the last update of web compatibility interventions. This collection \
              contains site-specific fixes and patches that make broken websites work correctly in \
              Firefox. These interventions address issues with websites that detect browser features \
              incorrectly or rely on non-standard APIs.",
                ),
                (
                    "services.settings.main.anti-tracking-url-decoration.last_check",
                    "Records the last check for URL decoration parameters used for tracking protection. \
              Some websites add tracking parameters to URLs (like utm_source, fbclid) that allow \
              third parties to track users across sites. This collection contains lists of these \
              tracking parameters so Firefox can strip them for privacy protection.",
                ),
                (
                    "services.settings.main.addons-data-leak-blocker-domains.last_check",
                    "Timestamp for updates to the list of domains where Firefox blocks add-ons to prevent \
              data leakage. This security feature prevents extensions from accessing sensitive websites \
              like banking sites, payment processors, and government domains where malicious extensions \
              could steal credentials or personal data.",
                ),
                (
                    "services.settings.main.newtab-wallpapers-v2.last_check",
                    "Records the last update check for Firefox new tab page wallpapers. This collection \
              contains metadata about background images (wallpapers) that users can apply to customize \
              the new tab page. It includes wallpaper IDs, file locations, and availability information.",
                ),
                (
                    "services.settings.main.password-recipes.last_check",
                    "Timestamp for password generation recipe updates. This collection contains rules for \
              Firefox's password generator feature, which automatically suggests strong, unique passwords \
              when creating new accounts. Recipes define password requirements (length, character types) \
              for specific websites based on their password policies.",
                ),
                (
                    "services.settings.main.search-categorization.last_check",
                    "Records the last update for search categorization rules. This collection contains \
              data that helps Firefox categorize search engines and search results for telemetry and \
              analysis purposes, understanding how users interact with different search providers.",
                ),
                (
                    "services.settings.main.search-config-overrides-v2.last_check",
                    "Timestamp for search configuration override updates. This collection contains \
              region-specific or localized overrides to Firefox's default search engine configuration, \
              allowing Mozilla to customize which search engines are available as defaults in different \
              countries or regions.",
                ),
                (
                    "services.settings.main.url-classifier-skip-urls.last_check",
                    "Records the last update of URLs that should be skipped by Safe Browsing checks. \
              Firefox's URL classifier checks websites against malware and phishing blocklists. \
              This collection contains URLs known to be safe that can skip these checks to improve \
              performance and reduce server load.",
                ),
                (
                    "services.settings.main.moz-essential-domain-fallbacks.last_check",
                    "Timestamp for essential domain fallback list updates. This collection contains \
              fallback domains for critical Mozilla services. If a primary service endpoint fails, \
              Firefox can use these fallback domains to maintain essential functionality like updates, \
              Remote Settings sync, and crash reporting.",
                ),
                (
                    "services.settings.main.websites-with-shared-credential-backends.last_check",
                    "Records the last update of websites that share credential backend infrastructure. \
              This collection identifies websites that use common authentication or payment processing \
              systems, which helps Firefox's password manager and autofill features work correctly \
              across related sites.",
                ),
                (
                    "services.settings.main.devtools-compatibility-browsers.last_check",
                    "Timestamp for updates to the browser compatibility database used by Developer Tools. \
              This collection contains information about which web features are supported in different \
              browsers and versions, powering the Browser Compatibility tab in DevTools that warns \
              developers about features that may not work in other browsers.",
                ),
                (
                    "services.settings.main.nimbus-secure-experiments.last_check",
                    "Records the last check for Nimbus secure experiment configurations. Nimbus is \
              Mozilla's current experiment system for A/B testing, feature rollouts, and staged \
              releases. This collection contains secure experiment definitions that are validated \
              and signed before delivery to ensure they haven't been tampered with.",
                ),
                (
                    "services.settings.main.tracking-protection-lists.last_check",
                    "Timestamp for tracking protection list updates. This collection contains the \
              blocklists used by Firefox's Enhanced Tracking Protection, including lists of known \
              tracking scripts, fingerprinting scripts, social media trackers, and cryptominers. \
              Firefox downloads these lists regularly to maintain up-to-date protection.",
                ),
                (
                    "services.settings.main.translations-models.last_check",
                    "Records the last update check for Firefox Translations models. Firefox's local \
              translation feature uses machine learning models to translate web pages without sending \
              data to external servers. This collection contains metadata about available translation \
              models (language pairs like en→es, fr→de) including file sizes, version numbers, and \
              download locations for the Bergamot translation engine models.",
                ),
                (
                    "services.settings.main.message-groups.last_check",
                    "Timestamp for Firefox Messaging System message group updates. This collection \
              contains configuration for groups of related messages, controlling frequency capping \
              (how often messages can show), priority ordering, and targeting rules. It ensures users \
              aren't overwhelmed by too many in-product messages.",
                ),
                (
                    "services.settings.main.partitioning-exempt-urls.last_check",
                    "Records the last update of URLs exempt from state partitioning. Firefox partitions \
              browser state (cookies, cache, storage) by top-level site to prevent cross-site tracking. \
              This collection contains sites that are exempt from this partitioning because they \
              require third-party context to function (e.g., embedded payment widgets, login systems).",
                ),
                (
                    "services.settings.main.cfr.last_check",
                    "Timestamp for Contextual Feature Recommendation (CFR) message updates. CFR is a \
              messaging system that suggests Firefox features and extensions at relevant moments, like \
              recommending a password manager when you visit a signup page. This collection contains the \
              CFR messages that are shown to users.",
                ),
                (
                    "services.settings.main.url-parser-default-unknown-schemes-interventions.last_check",
                    "Records the last update for URL parser interventions for unknown schemes. Some \
              websites use non-standard or malformed URL schemes that can break or cause unexpected \
              behavior. This collection contains site-specific patches to handle these unusual URL \
              patterns correctly.",
                ),
                (
                    "services.settings.main.query-stripping.last_check",
                    "Timestamp for query parameter stripping list updates. This collection contains \
              tracking parameters that Firefox should strip from URLs to enhance privacy. When you \
              click a link with tracking parameters (like ?utm_source=email, ?fbclid=123), Firefox \
              removes these parameters before navigating to the site, preventing the destination site \
              from tracking how you arrived there.",
                ),
                (
                    "services.settings.main.search-config-v2.last_check",
                    "Records the last update for the search engine configuration. This collection \
              contains comprehensive data about search engines available in Firefox, including their \
              names, URLs, parameter formats, icon URLs, and regional availability. It's the master \
              list that determines which search engines appear in Firefox's search options.",
                ),
                (
                    "services.settings.main.nimbus-desktop-experiments.last_check",
                    "Timestamp for desktop-specific Nimbus experiment updates. This is the main collection \
              for Firefox desktop experiments, containing A/B test configurations, feature rollouts, \
              and staged feature releases. It defines which users see which experiment variants based \
              on targeting criteria.",
                ),
                (
                    "services.settings.main.urlbar-persisted-search-terms.last_check",
                    "Records the last update for URL bar search term persistence configuration. This \
              collection contains settings for whether and how Firefox remembers search terms in the \
              address bar, allowing users to see and reuse their previous searches.",
                ),
                (
                    "services.settings.main.search-config-icons.last_check",
                    "Timestamp for search engine icon updates. This collection contains the icon images \
              and metadata for search engines that appear in Firefox's search selector and address bar. \
              Icons are periodically updated to reflect current search engine branding.",
                ),
                (
                    "services.settings.main.devtools-devices.last_check",
                    "Records the last update for the Developer Tools device list. This collection contains \
              information about mobile devices, tablets, and other viewports available in DevTools' \
              Responsive Design Mode. It includes device names, screen sizes, user agent strings, and \
              touch capabilities for testing responsive designs.",
                ),
                (
                    "services.settings.main.url-classifier-exceptions.last_check",
                    "Timestamp for Safe Browsing exception list updates. This collection contains URLs \
              and domains that should be exempt from Safe Browsing checks. Sites may be on this list \
              if they were incorrectly flagged as malicious or if they are known-safe internal services \
              that trigger false positives.",
                ),
                (
                    "services.settings.main.sites-classification.last_check",
                    "Records the last update for website classification data. This collection contains \
              categories and classifications for websites, which Firefox may use for various purposes \
              including telemetry analysis, feature targeting, and understanding user interaction \
              patterns with different types of sites.",
                ),
                (
                    "services.settings.main.translations-wasm.last_check",
                    "Timestamp for Firefox Translations WebAssembly module updates. The translation \
              feature uses a WASM-compiled version of the Bergamot translation engine. This collection \
              contains metadata about the WASM binaries, including version numbers, file sizes, and \
              download locations for the translation runtime environment.",
                ),
                (
                    "services.settings.main.bounce-tracking-protection-exceptions.last_check",
                    "Records the last update for bounce tracking protection exception list. Bounce \
              trackers redirect users through intermediate sites to track their behavior. Firefox's \
              bounce tracking protection blocks this technique, but this collection contains sites \
              that are exempt because they use redirects for legitimate purposes (like login flows).",
                ),
                (
                    "services.settings.main.hijack-blocklists.last_check",
                    "Timestamp for hijack blocklist updates. This collection contains domains and \
              patterns associated with browser hijacking attempts, such as sites that try to modify \
              browser settings, hijack the new tab page, or inject unwanted search engines. Firefox \
              blocks these to protect user control of the browser.",
                ),
                (
                    "services.settings.main.remote-permissions.last_check",
                    "Records the last update for remotely configured permission policies. This collection \
              contains permission settings that can be updated without a browser update, such as which \
              websites can access sensitive APIs or how certain permission prompts should behave \
              in specific contexts.",
                ),
                (
                    "services.settings.main.fingerprinting-protection-overrides.last_check",
                    "Timestamp for fingerprinting protection override list updates. Firefox's \
              fingerprinting protection blocks techniques that uniquely identify your browser. This \
              collection contains site-specific overrides for when stricter protection would break \
              legitimate functionality.",
                ),
                (
                    "services.settings.main.password-rules.last_check",
                    "Records the last update for password generation rules. This collection contains \
              detailed password requirements for websites, such as minimum length, required character \
              types, and forbidden characters. Firefox's password generator uses these rules to create \
              passwords that will be accepted by each website's signup form.",
                ),
                (
                    "services.settings.main.search-default-override-allowlist.last_check",
                    "Timestamp for search engine override allowlist updates. This collection contains \
              search engines that are permitted to override the default search engine in specific \
              regions or contexts. It ensures that only approved search engines can be set as defaults \
              through certain installation or configuration methods.",
                ),
                (
                    "services.settings.main.addons-manager-settings.last_check",
                    "Records the last update for add-ons manager configuration. This collection contains \
              settings and policies for Firefox's extension management system, including blocklist \
              data, recommended extensions, and administrative policies for enterprise deployments.",
                ),
                (
                    "services.settings.main.doh-config.last_check",
                    "Timestamp for DNS over HTTPS configuration updates. This collection contains \
              DoH deployment settings including which providers to use in different regions, automatic \
              enablement rules, and network-based heuristics for when to enable or disable DoH based \
              on the detected network environment.",
                ),
                (
                    "services.settings.main.search-telemetry-v2.last_check",
                    "Records the last update for search telemetry configuration. This collection contains \
              rules for categorizing and analyzing search engine usage patterns. It helps Mozilla \
              understand which search engines users prefer, how search behavior varies by region, and \
              how search features are being used.",
                ),
                (
                    "services.settings.main.top-sites.last_check",
                    "Timestamp for Top Sites configuration updates. This collection contains default \
              sites and configuration for Firefox's new tab page Top Sites section, which shows \
              frequently-visited websites for quick access. It includes sponsored site configurations \
              and regional site recommendations.",
                ),
                (
                    "services.settings.main.language-dictionaries.last_check",
                    "Records the last update for spell-check dictionary metadata. This collection contains \
              information about available spell-check dictionaries for different languages, including \
              download locations, version numbers, and file sizes. Firefox uses this to provide spell \
              checking in text fields and text areas.",
                ),
                // Update Timer Preferences
                (
                    "app.update.lastUpdateTime.browser-cleanup-thumbnails",
                    "Timestamp of the last time Firefox ran the browser thumbnail cleanup task. Firefox \
              captures page thumbnails for the new tab page and session restore. This maintenance task \
              removes old or unused thumbnails to free up disk space and keep the thumbnail cache \
              manageable.",
                ),
                (
                    "app.update.lastUpdateTime.recipe-client-addon-run",
                    "Timestamp of the last execution of the Normandy recipe client add-on. Normandy \
              was Mozilla's legacy system for delivering experiments, preference rollouts, and feature \
              configurations. This timer controlled when Firefox checked for and applied new recipes. \
              (Note: Normandy has been replaced by the Nimbus system.)",
                ),
                (
                    "app.update.lastUpdateTime.xpi-signature-verification",
                    "Timestamp of the last time Firefox verified extension signatures. All Firefox \
              extensions must be digitally signed by Mozilla to ensure they haven't been tampered \
              with. This periodic task re-validates installed extensions to maintain security and \
              detect any compromised or modified add-ons.",
                ),
                (
                    "app.update.lastUpdateTime.services-settings-poll-changes",
                    "Timestamp of the last time Firefox polled for Remote Settings changes. This timer \
              controls how often Firefox checks Mozilla's servers for updates to Remote Settings \
              collections, ensuring blocklists, experiment configurations, and other remotely-delivered \
              data stay current.",
                ),
                (
                    "app.update.lastUpdateTime.region-update-timer",
                    "Timestamp of the last time Firefox updated its detected home region. Firefox \
              determines your geographic region based on your IP address to provide localized search \
              engines, region-specific content, and compliance with local regulations. This timer \
              controls how often this region detection is updated.",
                ),
                (
                    "app.update.lastUpdateTime.rs-experiment-loader-timer",
                    "Timestamp of the last time Firefox loaded experiment configurations from Remote \
              Settings. This timer controls when Firefox checks for new Nimbus experiments and feature \
              rollouts, enabling staged rollouts and A/B tests that can be deployed without requiring \
              a browser update.",
                ),
                (
                    "app.update.lastUpdateTime.suggest-ingest",
                    "Timestamp of the last time Firefox ingested Firefox Suggest data. Firefox Suggest \
              provides search suggestions, sponsored shortcuts, and quick results as you type in the \
              address bar. This timer controls how often this suggestion data is refreshed from \
              Mozilla's servers.",
                ),
                (
                    "app.update.lastUpdateTime.addon-background-update-timer",
                    "Timestamp of the last time Firefox checked for add-on updates in the background. \
              This timer controls how frequently Firefox automatically checks for updates to installed \
              extensions and themes, ensuring you have the latest versions with security fixes and \
              new features.",
                ),
                // Additional Security-related Remote Settings
                (
                    "services.settings.security-state.onecrl.last_check",
                    "Timestamp for the last update of the OneCRL (One CRL Revocation List) certificate \
              blocklist. This collection contains intermediate certificates that have been revoked \
              or are known to be compromised. Firefox blocks websites using these certificates to \
              protect against man-in-the-middle attacks and fraudulent websites.",
                ),
                (
                    "services.settings.blocklists.addons-bloomfilters.last_check",
                    "Timestamp for the last update of the add-on blocklist bloom filters. This collection \
              contains compact data structures for efficiently checking if extensions are on the \
              blocklist. Bloom filters allow Firefox to quickly determine if an extension ID is known \
              to be malicious or problematic without downloading the full blocklist.",
                ),
                (
                    "services.settings.security-state.intermediates.last_check",
                    "Timestamp for the last update of intermediate certificate data. This collection \
              contains intermediate certificates that are trusted for TLS/SSL connections. Firefox \
              uses this to properly validate certificate chains and establish secure HTTPS connections.",
                ),
                (
                    "services.settings.security-state.cert-revocations.last_check",
                    "Timestamp for the last update of the certificate revocation list. This collection \
              contains information about certificates that have been revoked before their expiration \
              date. Firefox checks this list to prevent connections to sites using compromised or \
              fraudulent certificates.",
                ),
                (
                    "services.settings.blocklists.gfx.last_check",
                    "Timestamp for the last update of the graphics driver blocklist. This collection \
              contains graphics drivers and hardware configurations known to cause stability or \
              security issues. Firefox blocks or restricts certain GPU features (like hardware acceleration \
              or WebGL) on these problematic configurations to prevent crashes and vulnerabilities.",
                ),
            ])
        })
        .get(key)
        .copied()
}
