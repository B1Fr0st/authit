General structure:

Types all should live in their reasonably respective files:
-   Request/Response types in /types/requests.rs
-   Monitoring/Data types in /types/data.rs
-   Core types in /types/core.rs (License, Product, internal types)

Handlers are broken up according to DESIGN_DOC.md, and can be read such as the following:
Assuming that /api/v1/ is the root, any given endpoint can be found in its respective mod.rs/NAME.rs file.
-   /private/license/generator would be   in /handlers/private/license.rs.
-   /public/auth would be in /handlers/public/mod.rs.